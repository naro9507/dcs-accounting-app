//! 帳簿の取込（仕訳の CSV・Excel 帳簿）。
//!
//! 取込は 2 段で行う。
//!
//! 1. 読む: ファイルを「下書きの取引」（`Draft`）に直す（`journal_csv`・`excel_book`）
//! 2. 確かめる・書き込む（このモジュール）: 科目と取引先を当てはめ、取引の検証
//!    （`domain::journal::validate`）と重複の判定をする。プレビューでは何も書き込まない。
//!    直すところ（`Issue`）が 1 つでも残っていれば、取り込まない。
//!
//! 同じ取引（日付と明細行が同じ）がすでに帳簿にあれば、重複として飛ばす。
//! 同じファイルをもう一度取り込んでも、二重にならない。

pub mod excel_book;
pub mod journal_csv;
pub mod statement_file;

use crate::db::accounts::{self, Account};
use crate::db::partners::{self, Partner};
use crate::db::transactions::{self, DbMasters, Transaction};
use crate::db::Database;
use crate::domain::journal::{
    self, AllowedInactive, EntryMode, LineInput, MasterLookup, Side, TransactionInput,
};
use crate::domain::string_enum;
use crate::error::{AppError, AppResult, ErrorCode};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 科目の指定。CSV は科目コード、Excel 帳簿は帳簿の中の番号。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountRef {
    Code(String),
    BookNumber(i64),
}

/// 取引先の指定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartnerRef {
    Code(String),
    BookNumber(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraftLine {
    pub side: Side,
    pub account: AccountRef,
    pub amount: i64,
    pub partner: Option<PartnerRef>,
    pub memo: String,
    /// 補助科目のコード（科目の下で探す）
    pub sub_account: Option<String>,
    pub tax_category: Option<crate::domain::master::TaxCategory>,
    pub is_qualified_invoice: bool,
}

/// 読んだだけの取引。まだ検証していない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    pub sheet: Option<String>,
    /// ファイルの中の行（1 から）
    pub row: usize,
    pub date: String,
    pub memo: String,
    pub lines: Vec<DraftLine>,
}

string_enum! {
    /// 取込で直すところの種類。画面はこれを見て文言を選ぶ。
    pub enum IssueCode {
        /// 見出しの行が見つからない・必要な列がない
        MissingColumns => "missingColumns",
        DuplicateColumns => "duplicateColumns",
        MissingTransactionNo => "missingTransactionNo",
        /// 日付が読めない・暦にない
        InvalidDate => "invalidDate",
        /// 取り込む年度の外の日付
        OutsideYear => "outsideYear",
        /// 金額が 1 円以上の整数でない
        InvalidAmount => "invalidAmount",
        /// 借方・貸方の書き方が違う
        InvalidSide => "invalidSide",
        /// 科目コードが帳簿にない
        UnknownAccount => "unknownAccount",
        /// Excel 帳簿の科目の番号に、このアプリの科目を当てていない
        UnmappedAccount => "unmappedAccount",
        /// 使わない設定にした科目・取引先
        InactiveMaster => "inactiveMaster",
        /// 取引先コードが帳簿にない
        UnknownPartner => "unknownPartner",
        /// 補助科目コードが、その科目の下にない
        UnknownSubAccount => "unknownSubAccount",
        /// 税区分が読めない
        InvalidTaxCategory => "invalidTaxCategory",
        /// 借方合計と貸方合計が合わない
        Unbalanced => "unbalanced",
        /// 同じ科目が借方と貸方の両方にある
        SameAccount => "sameAccount",
        /// 借方か貸方の片方しかない
        MissingSide => "missingSide",
        /// メモが長すぎる・明細行が多すぎる
        TooLong => "tooLong",
        InconsistentDate => "inconsistentDate",
        MonthLocked => "monthLocked",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub sheet: Option<String>,
    pub row: usize,
    pub code: IssueCode,
    /// 問題の値（科目コードなど）。個人名や金額は入れない
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl Issue {
    pub fn new(sheet: Option<&str>, row: usize, code: IssueCode) -> Self {
        Self {
            sheet: sheet.map(str::to_string),
            row,
            code,
            value: None,
        }
    }

    pub fn with(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

/// Excel 帳簿の科目（帳簿の中の番号）。画面でこのアプリの科目を当てる。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookAccount {
    pub number: i64,
    pub name_ja: String,
    pub name_en: String,
    /// 名前から選んだ候補
    pub suggested_account_id: Option<i64>,
    pub debit: i64,
    pub credit: i64,
}

/// Excel 帳簿の取引先（生徒・店・会社）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookPartner {
    pub number: i64,
    pub name: String,
}

/// ファイルを読んだ結果。
#[derive(Debug, Clone, Default)]
pub struct Parsed {
    pub drafts: Vec<Draft>,
    pub issues: Vec<Issue>,
    /// Excel 帳簿だけ
    pub book_accounts: Vec<BookAccount>,
    pub book_partners: Vec<BookPartner>,
}

/// Excel 帳簿の科目の番号と、このアプリの科目の対応。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMapping {
    pub number: i64,
    pub account_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountTotal {
    pub account_id: i64,
    pub debit: i64,
    pub credit: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPartner {
    pub code: String,
    pub name: String,
}

/// 取込の前に見せる内容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub fiscal_year: Option<i64>,
    /// 年度がまだなく、取り込むと作る
    pub creates_fiscal_year: bool,
    /// 取り込む取引の数
    pub ready: usize,
    /// すでに帳簿にあるので飛ばす取引の数
    pub duplicates: usize,
    /// 直すところ（最初の `ISSUES_SHOWN` 件）
    pub issues: Vec<Issue>,
    pub issue_count: usize,
    pub total_debit: i64,
    pub total_credit: i64,
    /// 取り込む取引の、科目ごとの合計（年次集計の確かめに使う）
    pub account_totals: Vec<AccountTotal>,
    pub book_accounts: Vec<BookAccount>,
    pub new_partners: Vec<NewPartner>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub fiscal_year: i64,
    pub imported: usize,
    pub duplicates: usize,
}

const ISSUES_SHOWN: usize = 200;

/// 取引先をどうするか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartnerPlan {
    Existing(i64),
    /// `Plan::new_partners` の番号
    New(usize),
}

/// 当てはめと検証を済ませた取引。
struct Ready {
    input: TransactionInput,
    /// 明細行ごとの取引先（作る前の取引先もある）
    partners: Vec<Option<PartnerPlan>>,
}

struct Plan {
    fiscal_year: Option<i64>,
    creates_fiscal_year: bool,
    ready: Vec<Ready>,
    duplicates: usize,
    issues: Vec<Issue>,
    new_partners: Vec<NewPartner>,
    book_accounts: Vec<BookAccount>,
}

struct Masters {
    accounts: Vec<Account>,
    partners: Vec<Partner>,
    sub_accounts: Vec<crate::db::sub_accounts::SubAccount>,
}

impl MasterLookup for Masters {
    fn account_active(&self, id: i64) -> Option<bool> {
        self.accounts
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.is_active)
    }
    fn partner_active(&self, id: i64) -> Option<bool> {
        self.partners
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.is_active)
    }
    fn sub_account(&self, id: i64) -> Option<(i64, bool)> {
        self.sub_accounts
            .iter()
            .find(|s| s.id == id)
            .map(|s| (s.account_id, s.is_active))
    }
}

// 重複判定は取引先・補助科目・税区分・摘要も含む。明細の並び順は問わない。
type SignatureLine = (
    &'static str,
    i64,
    i64,
    Option<i64>,
    Option<i64>,
    &'static str,
    bool,
    String,
);
type Signature = (String, String, Vec<SignatureLine>);

fn signature(date: &str, memo: &str, lines: impl Iterator<Item = SignatureLine>) -> Signature {
    let mut lines: Vec<_> = lines.collect();
    lines.sort_unstable();
    (date.to_string(), memo.to_string(), lines)
}

fn existing_signatures(
    conn: &Connection,
    year: i64,
    masters: &DbMasters,
) -> AppResult<HashMap<Signature, usize>> {
    let mut counts = HashMap::new();
    let all: Vec<Transaction> = transactions::list_between(
        conn,
        year,
        &format!("{year:04}-01-01"),
        &format!("{year:04}-12-31"),
    )?;
    for t in all {
        let key = signature(
            &t.date,
            &t.memo,
            t.lines.iter().map(|l| {
                let tax = l.tax_category.unwrap_or_else(|| {
                    journal::resolve_tax(
                        None,
                        masters.account_tax(l.account_id),
                        masters.tax_exempt(&t.date),
                    )
                });
                (
                    l.side.as_str(),
                    l.account_id,
                    l.amount,
                    l.partner_id,
                    l.sub_account_id,
                    tax.as_str(),
                    l.is_qualified_invoice,
                    l.memo.clone(),
                )
            }),
        );
        *counts.entry(key).or_insert(0) += 1;
    }
    Ok(counts)
}

fn fiscal_year_state(conn: &Connection, year: i64) -> AppResult<Option<bool>> {
    Ok(conn
        .query_row(
            "SELECT status = 'closed' FROM fiscal_years WHERE year = ?1",
            [year],
            |row| row.get(0),
        )
        .optional()?)
}

/// `journal::validate` のエラーを、取込で直すところに言い換える。
fn issue_of(error: &AppError, draft: &Draft) -> Issue {
    let code = match (error.code, error.field) {
        (ErrorCode::InvalidFormat, Some("date")) => IssueCode::InvalidDate,
        (ErrorCode::OutOfRange, Some("date")) => IssueCode::OutsideYear,
        (ErrorCode::OutOfRange, Some("amount")) => IssueCode::InvalidAmount,
        (ErrorCode::OutOfRange, _) | (ErrorCode::TooLong, _) => IssueCode::TooLong,
        (ErrorCode::NotFound, Some("partnerId")) => IssueCode::UnknownPartner,
        (ErrorCode::NotFound, _) => IssueCode::UnknownAccount,
        (ErrorCode::Inactive, _) => IssueCode::InactiveMaster,
        (ErrorCode::SameAccount, _) => IssueCode::SameAccount,
        (ErrorCode::Required, _) => IssueCode::MissingSide,
        _ => IssueCode::Unbalanced,
    };
    let issue = Issue::new(draft.sheet.as_deref(), draft.row, code);
    if code == IssueCode::OutsideYear || code == IssueCode::InvalidDate {
        issue.with(draft.date.clone())
    } else {
        issue
    }
}

/// 取引先の候補の名前で、帳簿にある取引先を探す。なければ作る（コードは帳簿の番号）。
fn plan_book_partner(
    number: i64,
    book_partners: &[BookPartner],
    masters: &Masters,
    new_partners: &mut Vec<NewPartner>,
    cache: &mut HashMap<i64, PartnerPlan>,
) -> PartnerPlan {
    if let Some(plan) = cache.get(&number) {
        return *plan;
    }
    let name = book_partners
        .iter()
        .find(|p| p.number == number)
        .map(|p| p.name.clone())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| number.to_string());
    let plan = match masters.partners.iter().find(|p| p.name_ja == name) {
        Some(p) => PartnerPlan::Existing(p.id),
        None => {
            let taken = |code: &str| {
                masters.partners.iter().any(|p| p.code == code)
                    || new_partners.iter().any(|p| p.code == code)
            };
            let base = number.to_string();
            let code = std::iter::once(base.clone())
                .chain((2..).map(|n| format!("{base}-{n}")))
                .find(|c| !taken(c))
                .unwrap_or(base);
            new_partners.push(NewPartner { code, name });
            PartnerPlan::New(new_partners.len() - 1)
        }
    };
    cache.insert(number, plan);
    plan
}

/// 読んだ取引に科目・取引先を当てはめ、検証し、重複を除く。DB には書き込まない。
fn plan(
    conn: &Connection,
    parsed: &Parsed,
    year: Option<i64>,
    mapping: &[AccountMapping],
) -> AppResult<Plan> {
    let masters = Masters {
        accounts: accounts::list(conn)?,
        partners: partners::list(conn)?,
        sub_accounts: crate::db::sub_accounts::list(conn)?,
    };
    let mut issues = parsed.issues.clone();
    // 年度: 指定がなければ、最初の取引の日付の年
    let fiscal_year = year.or_else(|| {
        parsed
            .drafts
            .iter()
            .find_map(|d| journal::parse_date(d.date.trim()).map(|(y, _, _)| y))
    });
    let creates_fiscal_year = match fiscal_year {
        Some(y) => match fiscal_year_state(conn, y)? {
            None => true,
            Some(true) => return Err(AppError::new(ErrorCode::YearClosed)),
            Some(false) => false,
        },
        None => false,
    };
    if let Some(y) = fiscal_year {
        if !(2000..=2100).contains(&y) {
            return Err(AppError::field(ErrorCode::OutOfRange, "fiscalYear"));
        }
    }

    let current = fiscal_year.filter(|_| !creates_fiscal_year);
    let db_masters = current
        .map(|year| DbMasters::load(conn, year))
        .transpose()?;
    let mut existing = match (current, &db_masters) {
        (Some(y), Some(masters)) => existing_signatures(conn, y, masters)?,
        _ => HashMap::new(),
    };
    let locked_months = current
        .map(|year| crate::db::locks::status(conn, year))
        .transpose()?
        .map(|status| status.locked_months)
        .unwrap_or_default();
    let validator: &dyn MasterLookup = db_masters
        .as_ref()
        .map_or(&masters as &dyn MasterLookup, |m| m);
    let mapping: HashMap<i64, i64> = mapping.iter().map(|m| (m.number, m.account_id)).collect();
    let mut new_partners = Vec::new();
    let mut partner_cache = HashMap::new();
    let mut ready = Vec::new();
    let mut duplicates = 0;

    for draft in &parsed.drafts {
        let mut lines = Vec::with_capacity(draft.lines.len());
        let mut partner_plans = Vec::with_capacity(draft.lines.len());
        let mut problem = None;
        for line in &draft.lines {
            let account_id = match &line.account {
                AccountRef::Code(code) => masters
                    .accounts
                    .iter()
                    .find(|a| &a.code == code)
                    .map(|a| a.id)
                    .ok_or_else(|| {
                        Issue::new(draft.sheet.as_deref(), draft.row, IssueCode::UnknownAccount)
                            .with(code.clone())
                    }),
                AccountRef::BookNumber(n) => mapping.get(n).copied().ok_or_else(|| {
                    Issue::new(
                        draft.sheet.as_deref(),
                        draft.row,
                        IssueCode::UnmappedAccount,
                    )
                    .with(n.to_string())
                }),
            };
            let partner = match &line.partner {
                None => Ok(None),
                Some(PartnerRef::Code(code)) => masters
                    .partners
                    .iter()
                    .find(|p| &p.code == code)
                    .map(|p| Some(PartnerPlan::Existing(p.id)))
                    .ok_or_else(|| {
                        Issue::new(draft.sheet.as_deref(), draft.row, IssueCode::UnknownPartner)
                            .with(code.clone())
                    }),
                Some(PartnerRef::BookNumber(n)) => Ok(Some(plan_book_partner(
                    *n,
                    &parsed.book_partners,
                    &masters,
                    &mut new_partners,
                    &mut partner_cache,
                ))),
            };
            // 補助科目は、その科目の下でコードを探す
            let sub_account = match (&account_id, &line.sub_account) {
                (Ok(account_id), Some(code)) => masters
                    .sub_accounts
                    .iter()
                    .find(|s| s.account_id == *account_id && &s.code == code)
                    .map(|s| Some(s.id))
                    .ok_or_else(|| {
                        Issue::new(
                            draft.sheet.as_deref(),
                            draft.row,
                            IssueCode::UnknownSubAccount,
                        )
                        .with(code.clone())
                    }),
                _ => Ok(None),
            };
            match (account_id, partner, sub_account) {
                (Ok(account_id), Ok(partner), Ok(sub_account_id)) => {
                    lines.push(LineInput {
                        sub_account_id,
                        tax_category: line.tax_category,
                        is_qualified_invoice: line.is_qualified_invoice,
                        side: line.side,
                        account_id,
                        amount: line.amount,
                        partner_id: match partner {
                            Some(PartnerPlan::Existing(id)) => Some(id),
                            _ => None,
                        },
                        memo: line.memo.clone(),
                    });
                    partner_plans.push(partner);
                }
                (Err(issue), _, _) | (_, Err(issue), _) | (_, _, Err(issue)) => {
                    problem.get_or_insert(issue);
                }
            }
        }
        if let Some(issue) = problem {
            issues.push(issue);
            continue;
        }
        let input = TransactionInput {
            date: draft.date.clone(),
            entry_mode: EntryMode::Import,
            memo: draft.memo.clone(),
            lines,
        };
        let Some(year) = fiscal_year else { continue };
        let valid = match journal::validate(&input, year, validator, &AllowedInactive::default()) {
            Ok(valid) => valid,
            Err(error) => {
                issues.push(issue_of(&error, draft));
                continue;
            }
        };
        let key = signature(
            &valid.date,
            &valid.memo,
            valid
                .lines
                .iter()
                .zip(&partner_plans)
                .map(|(line, partner)| {
                    let partner = match partner {
                        Some(PartnerPlan::Existing(id)) => Some(*id),
                        Some(PartnerPlan::New(index)) => Some(-1 - *index as i64),
                        None => None,
                    };
                    (
                        line.side.as_str(),
                        line.account_id,
                        line.amount,
                        partner,
                        line.sub_account_id,
                        line.tax_category.map_or("", |tax| tax.as_str()),
                        line.is_qualified_invoice,
                        line.memo.clone(),
                    )
                }),
        );
        if let Some(count) = existing.get_mut(&key).filter(|c| **c > 0) {
            *count -= 1;
            duplicates += 1;
            continue;
        }
        if journal::parse_date(&valid.date)
            .is_some_and(|(_, month, _)| locked_months.contains(&month))
        {
            issues.push(
                Issue::new(draft.sheet.as_deref(), draft.row, IssueCode::MonthLocked)
                    .with(valid.date),
            );
            continue;
        }
        ready.push(Ready {
            input,
            partners: partner_plans,
        });
    }

    // 候補の科目を当てる（取引を当てはめるときには使わない。画面で確かめてから使う）
    let book_accounts = parsed
        .book_accounts
        .iter()
        .map(|b| BookAccount {
            suggested_account_id: excel_book::suggest_account(
                &b.name_ja,
                &b.name_en,
                &masters.accounts,
            ),
            ..b.clone()
        })
        .collect();

    // 作る取引先は、取り込む取引で使うものだけにする
    let used: Vec<usize> = {
        let mut used: Vec<usize> = ready
            .iter()
            .flat_map(|r| r.partners.iter())
            .filter_map(|p| match p {
                Some(PartnerPlan::New(i)) => Some(*i),
                _ => None,
            })
            .collect();
        used.sort_unstable();
        used.dedup();
        used
    };
    let renumber: HashMap<usize, usize> = used
        .iter()
        .enumerate()
        .map(|(new, old)| (*old, new))
        .collect();
    let new_partners = used.iter().map(|i| new_partners[*i].clone()).collect();
    for r in &mut ready {
        for p in r.partners.iter_mut() {
            if let Some(PartnerPlan::New(i)) = p {
                *i = renumber[i];
            }
        }
    }

    Ok(Plan {
        fiscal_year,
        creates_fiscal_year,
        ready,
        duplicates,
        issues,
        new_partners,
        book_accounts,
    })
}

/// 取り込む前に、内容と直すところを見る。何も書き込まない。
pub fn preview(
    conn: &Connection,
    parsed: &Parsed,
    year: Option<i64>,
    mapping: &[AccountMapping],
) -> AppResult<ImportPreview> {
    let plan = plan(conn, parsed, year, mapping)?;
    let mut totals: HashMap<i64, AccountTotal> = HashMap::new();
    let (mut total_debit, mut total_credit) = (0, 0);
    for line in plan.ready.iter().flat_map(|r| &r.input.lines) {
        let t = totals.entry(line.account_id).or_insert(AccountTotal {
            account_id: line.account_id,
            debit: 0,
            credit: 0,
        });
        match line.side {
            Side::Debit => {
                t.debit += line.amount;
                total_debit += line.amount;
            }
            Side::Credit => {
                t.credit += line.amount;
                total_credit += line.amount;
            }
        }
    }
    let mut account_totals: Vec<AccountTotal> = totals.into_values().collect();
    account_totals.sort_by_key(|t| t.account_id);
    Ok(ImportPreview {
        fiscal_year: plan.fiscal_year,
        creates_fiscal_year: plan.creates_fiscal_year,
        ready: plan.ready.len(),
        duplicates: plan.duplicates,
        issue_count: plan.issues.len(),
        issues: plan.issues.into_iter().take(ISSUES_SHOWN).collect(),
        total_debit,
        total_credit,
        account_totals,
        book_accounts: plan.book_accounts,
        new_partners: plan.new_partners,
    })
}

/// 取り込む。直すところが残っていれば何もしない。全部を 1 つの DB トランザクションで書く。
pub fn commit(
    db: &Database,
    parsed: &Parsed,
    year: Option<i64>,
    mapping: &[AccountMapping],
) -> AppResult<ImportResult> {
    db.transaction(|tx| {
        let plan = plan(tx, parsed, year, mapping)?;
        if !plan.issues.is_empty() {
            return Err(AppError::new(ErrorCode::ImportBlocked));
        }
        let Some(fiscal_year) = plan.fiscal_year else {
            return Ok(ImportResult {
                fiscal_year: 0,
                imported: 0,
                duplicates: plan.duplicates,
            });
        };
        if plan.creates_fiscal_year {
            create_fiscal_year(tx, fiscal_year)?;
        }
        let mut partner_ids = Vec::with_capacity(plan.new_partners.len());
        for p in &plan.new_partners {
            tx.execute(
                "INSERT INTO partners (code, name_ja, is_customer, is_vendor, sort)
                 VALUES (?1, ?2, ?3, ?4, (SELECT coalesce(max(sort), 0) + 10 FROM partners))",
                params![
                    p.code,
                    p.name,
                    !is_vendor_code(&p.code),
                    is_vendor_code(&p.code)
                ],
            )?;
            partner_ids.push(tx.last_insert_rowid());
        }
        let inputs: Vec<TransactionInput> = plan
            .ready
            .into_iter()
            .map(|mut r| {
                for (line, partner) in r.input.lines.iter_mut().zip(&r.partners) {
                    if let Some(PartnerPlan::New(i)) = partner {
                        line.partner_id = Some(partner_ids[*i]);
                    }
                }
                r.input
            })
            .collect();
        let imported = transactions::insert_many(tx, fiscal_year, &inputs)?;
        Ok(ImportResult {
            fiscal_year,
            imported,
            duplicates: plan.duplicates,
        })
    })
}

/// Excel 帳簿では、501 番からが店・会社（仕入先・支払先）、それより前が生徒（顧客）。
fn is_vendor_code(code: &str) -> bool {
    code.split('-')
        .next()
        .and_then(|n| n.parse::<i64>().ok())
        .is_some_and(|n| n >= 501)
}

/// 取り込む年度を作る。申告区分と消費税の扱いは、いちばん近い年度に合わせる。
fn create_fiscal_year(conn: &Connection, year: i64) -> AppResult<()> {
    let changed = conn.execute(
        "INSERT INTO fiscal_years (year, filing_type, tax_status)
         SELECT ?1, filing_type, tax_status FROM fiscal_years ORDER BY abs(year - ?1), year DESC LIMIT 1",
        [year],
    )?;
    if changed == 0 {
        return Err(AppError::new(ErrorCode::NotSetUp));
    }
    Ok(())
}

/// 金額を読む。「1,200」「¥1,200」「１２００」「1200.0」を受け付ける。1 円以上の整数だけ。
pub fn parse_amount(text: &str) -> Option<i64> {
    let normalized: String = text
        .trim()
        .chars()
        .filter(|c| !matches!(c, ',' | '，' | '¥' | '￥' | ' ' | '円'))
        .map(|c| match c {
            '０'..='９' => char::from_u32(c as u32 - '０' as u32 + '0' as u32).unwrap_or(c),
            _ => c,
        })
        .collect();
    let normalized = normalized.strip_suffix(".0").unwrap_or(&normalized);
    let value: i64 = normalized.parse().ok()?;
    (1..=journal::AMOUNT_MAX).contains(&value).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_amounts_written_in_various_ways() {
        assert_eq!(parse_amount("1,200"), Some(1200));
        assert_eq!(parse_amount(" ¥1,200 "), Some(1200));
        assert_eq!(parse_amount("１２００"), Some(1200));
        assert_eq!(parse_amount("1200.0"), Some(1200));
        assert_eq!(parse_amount("1200.5"), None);
        assert_eq!(parse_amount("0"), None);
        assert_eq!(parse_amount("-100"), None);
        assert_eq!(parse_amount(""), None);
    }

    #[test]
    fn signature_ignores_line_order_but_keeps_the_memo() {
        let line = |side: Side, account| {
            (
                side.as_str(),
                account,
                100,
                None,
                None,
                "",
                false,
                String::new(),
            )
        };
        let debit = line(Side::Debit, 1);
        let credit = line(Side::Credit, 2);
        let a = signature(
            "2026-01-01",
            "A",
            [debit.clone(), credit.clone()].into_iter(),
        );
        let b = signature(
            "2026-01-01",
            "A",
            [credit.clone(), debit.clone()].into_iter(),
        );
        assert_eq!(a, b);
        assert_ne!(a, signature("2026-01-01", "B", [debit, credit].into_iter()));
    }

    #[test]
    fn vendor_codes_start_at_501() {
        assert!(is_vendor_code("501"));
        assert!(is_vendor_code("502-2"));
        assert!(!is_vendor_code("12"));
    }
}
