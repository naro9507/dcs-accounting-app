//! カード・銀行・電子マネーの明細（CSV・Excel）の読み方。
//!
//! カード会社ごとの違いは「取込形式」（`FormatDefinition`）というデータで表し、コードに
//! 会社ごとの分岐を書かない。同梱のプリセットも、利用者が作る形式と同じ形のデータにする。
//!
//! ファイルは呼び出し側で「文字列の表」（行 × 列）に直してから渡す。ここでは
//! 表から明細の行（利用日・金額・店名…）を読み、読めない行とその理由を返す。

use super::journal::{days_in_month, AMOUNT_MAX};
use super::string_enum;
use serde::{Deserialize, Serialize};

string_enum! {
    /// 文字コード。
    pub enum TextEncoding {
        Utf8 => "utf8",
        ShiftJis => "shiftJis",
    }
}

string_enum! {
    /// 区切り文字。
    pub enum Delimiter {
        Comma => "comma",
        Tab => "tab",
        Semicolon => "semicolon",
    }
}

impl Delimiter {
    pub fn byte(self) -> u8 {
        match self {
            Delimiter::Comma => b',',
            Delimiter::Tab => b'\t',
            Delimiter::Semicolon => b';',
        }
    }
}

string_enum! {
    /// 列の役割。
    pub enum ColumnRole {
        /// 読まない
        Ignore => "ignore",
        /// 利用日
        Date => "date",
        /// 金額（利用と返金が 1 列）
        Amount => "amount",
        /// 出金額（銀行の明細など、出金と入金が 2 列に分かれているとき）
        Withdrawal => "withdrawal",
        /// 入金額
        Deposit => "deposit",
        /// 店名・内容
        Merchant => "merchant",
        /// メモ
        Memo => "memo",
        /// 外貨額（海外利用）。メモに残す
        ForeignAmount => "foreignAmount",
        /// 利用者（家族カードなど）
        User => "user",
    }
}

string_enum! {
    /// 金額の符号の向き。
    pub enum AmountSign {
        /// 利用（支払い）がプラス、返金がマイナス。カードの明細に多い
        SpendPositive => "spendPositive",
        /// 利用がマイナス、入金がプラス。口座の明細に多い
        SpendNegative => "spendNegative",
    }
}

string_enum! {
    /// 明細の種類。
    pub enum SourceKind {
        Card => "card",
        Bank => "bank",
        EMoney => "eMoney",
    }
}

fn default_true() -> bool {
    true
}

/// 取込形式の中身（ファイルの読み方）。取込先の口座は含めない（他の人と共有できるように）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatDefinition {
    pub encoding: TextEncoding,
    pub delimiter: Delimiter,
    /// 見出しの行（1 から）。0 なら見出しがない
    pub header_row: usize,
    /// 見出しを照合せずに先頭から読み飛ばす行数。
    #[serde(default, skip_serializing_if = "is_zero")]
    pub skip_rows: usize,
    /// 列ごとの役割。列の数より短ければ、残りは読まない
    pub columns: Vec<ColumnRole>,
    pub sign: AmountSign,
    /// 日付として読めない行（合計の行・注記など）は、黙って飛ばす
    #[serde(default = "default_true")]
    pub skip_unreadable_dates: bool,
    /// この形式の目印。保存したときの見出しの行（見出しがなければ空）。
    /// 読まない列は `{month}月` で始まる月別見出しのひな型も使える。
    #[serde(default)]
    pub header: Vec<String>,
}

fn is_zero(value: &usize) -> bool {
    *value == 0
}

impl FormatDefinition {
    fn data_start(&self) -> usize {
        self.header_row.max(self.skip_rows)
    }

    fn column(&self, role: ColumnRole) -> Option<usize> {
        self.columns.iter().position(|r| *r == role)
    }

    /// 必須の役割（利用日・金額・店名）がそろっているか。足りない役割を返す。
    pub fn missing_roles(&self) -> Vec<ColumnRole> {
        let mut missing = Vec::new();
        if self.column(ColumnRole::Date).is_none() {
            missing.push(ColumnRole::Date);
        }
        let has_amount = self.column(ColumnRole::Amount).is_some()
            || self.column(ColumnRole::Withdrawal).is_some()
            || self.column(ColumnRole::Deposit).is_some();
        if !has_amount {
            missing.push(ColumnRole::Amount);
        }
        if self.column(ColumnRole::Merchant).is_none() {
            missing.push(ColumnRole::Merchant);
        }
        missing
    }
}

/// 全角の英数字・記号を半角にする。
pub fn to_half_width(c: char) -> char {
    match c {
        '\u{FF01}'..='\u{FF5E}' => char::from_u32(c as u32 - 0xFEE0).unwrap_or(c),
        '\u{3000}' => ' ',
        '￥' => '¥',
        _ => c,
    }
}

/// 日付を `YYYY-MM-DD` にする。読めなければ `None`。
///
/// 「2026/10/05」「2026-10-5」「20261005」「2026年10月5日」「令和8年10月5日」「R8.10.5」
/// 「平成26年1月5日」「H26/1/5」を読む。後ろに時刻があってもよい。
/// レシートに多い「2026年11月 3日(火)」「令和 8年」のような、桁をそろえる空白も読む。
pub fn parse_date(text: &str) -> Option<String> {
    let text: String = text.trim().chars().map(to_half_width).collect();
    let text = join_date_parts(&text);
    let text = text.split_whitespace().next().unwrap_or("");
    let (base, rest) = era(text);
    let numbers: Vec<&str> = rest
        .split(|c: char| !c.is_ascii_digit() && c != '元')
        .filter(|s| !s.is_empty())
        .collect();
    let (year, month, day): (i64, u32, u32) = match numbers[..] {
        [ymd] if base.is_none() && ymd.len() == 8 && ymd.bytes().all(|b| b.is_ascii_digit()) => (
            ymd[..4].parse().ok()?,
            ymd[4..6].parse().ok()?,
            ymd[6..].parse().ok()?,
        ),
        [y, m, d] => {
            let y: i64 = if y == "元" { 1 } else { y.parse().ok()? };
            (y, m.parse().ok()?, d.parse().ok()?)
        }
        _ => return None,
    };
    let year = match base {
        Some(base) if (1..=9999).contains(&year) => base + year - 1,
        Some(_) => return None,
        None => year,
    };
    if !(1900..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
    {
        return None;
    }
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// 「年」「月」の後ろと、元号の後ろの空白を詰める（「2026年11月 3日」→「2026年11月3日」）。
/// 時刻の前の空白は残す（「2026/10/05 12:34」の時刻を日付に混ぜない）。
fn join_date_parts(text: &str) -> String {
    let mut joined = String::with_capacity(text.len());
    for c in text.chars() {
        let after_part = joined.ends_with(['年', '月'])
            || joined.ends_with("令和")
            || joined.ends_with("平成")
            || joined.ends_with("昭和");
        if c.is_whitespace() && after_part {
            continue;
        }
        joined.push(c);
    }
    joined
}

/// 和暦の元号を読む。元号があれば「その元年の西暦」と残りの文字列を返す。
fn era(text: &str) -> (Option<i64>, &str) {
    const ERAS: [(&str, i64); 6] = [
        ("令和", 2019),
        ("平成", 1989),
        ("昭和", 1926),
        ("R", 2019),
        ("H", 1989),
        ("S", 1926),
    ];
    for (name, base) in ERAS {
        if let Some(rest) = text.strip_prefix(name) {
            return (Some(base), rest);
        }
    }
    (None, text)
}

/// 金額を読む。「1,234」「¥1,234」「-1,234」「(1,234)」「▲1,234」「1234円」「1,234.00」。
/// 円未満のある金額や、空の欄は `None`。
pub fn parse_amount(text: &str) -> Option<i64> {
    let text: String = text
        .trim()
        .chars()
        .map(to_half_width)
        .filter(|c| !matches!(c, ',' | '¥' | '\\' | '円' | ' '))
        .collect();
    if text.is_empty() {
        return None;
    }
    let (negative, body) =
        if let Some(inner) = text.strip_prefix('(').and_then(|t| t.strip_suffix(')')) {
            (true, inner)
        } else if let Some(rest) = text
            .strip_prefix('-')
            .or_else(|| text.strip_prefix('▲'))
            .or_else(|| text.strip_prefix('△'))
            .or_else(|| text.strip_prefix('−'))
        {
            (true, rest)
        } else {
            (false, text.strip_prefix('+').unwrap_or(&text))
        };
    let body = match body.split_once('.') {
        Some((whole, fraction)) if fraction.bytes().all(|b| b == b'0') => whole,
        Some(_) => return None,
        None => body,
    };
    if body.is_empty() || !body.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let value: i64 = body.parse().ok()?;
    if value > AMOUNT_MAX {
        return None;
    }
    Some(if negative { -value } else { value })
}

/// 明細の 1 行。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatementRow {
    /// ファイルの中の行（1 から）
    pub row: usize,
    pub date: String,
    /// 出ていったお金。プラスは利用（支払い）、マイナスは返金・入金
    pub spend: i64,
    pub merchant: String,
    /// メモ。外貨額と利用者もここに書き足す
    pub memo: String,
}

string_enum! {
    /// 読まなかった行・読めなかった行の理由。
    pub enum RowProblem {
        /// 日付として読めない（合計の行・注記など）
        UnreadableDate => "unreadableDate",
        /// 金額が読めない
        UnreadableAmount => "unreadableAmount",
        /// 金額が 0
        ZeroAmount => "zeroAmount",
        /// 空の行
        Blank => "blank",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowNote {
    pub row: usize,
    pub problem: RowProblem,
}

/// 明細を読んだ結果。
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Statement {
    pub rows: Vec<StatementRow>,
    /// 読まずに飛ばした行（空の行、日付でない行）
    pub ignored: Vec<RowNote>,
    /// 読めなかった行。1 行でもあれば取り込まない
    pub errors: Vec<RowNote>,
}

impl Statement {
    pub fn first_date(&self) -> Option<&str> {
        self.rows.iter().map(|r| r.date.as_str()).min()
    }
    pub fn last_date(&self) -> Option<&str> {
        self.rows.iter().map(|r| r.date.as_str()).max()
    }
    /// 利用の合計（プラス）と、返金・入金の合計（プラスで返す）。
    pub fn totals(&self) -> (i64, i64) {
        self.rows.iter().fold((0, 0), |(spend, back), r| {
            if r.spend > 0 {
                (spend + r.spend, back)
            } else {
                (spend, back - r.spend)
            }
        })
    }
}

fn cell(row: &[String], index: Option<usize>) -> &str {
    index.and_then(|i| row.get(i)).map_or("", |s| s.trim())
}

/// 表から明細を読む。`table` はファイルの全行（見出しより前の行も含む）。
pub fn read(table: &[Vec<String>], def: &FormatDefinition) -> Statement {
    let mut statement = Statement::default();
    let date = def.column(ColumnRole::Date);
    let amount = def.column(ColumnRole::Amount);
    let withdrawal = def.column(ColumnRole::Withdrawal);
    let deposit = def.column(ColumnRole::Deposit);
    let merchant = def.column(ColumnRole::Merchant);
    let memo = def.column(ColumnRole::Memo);
    let foreign = def.column(ColumnRole::ForeignAmount);
    let user = def.column(ColumnRole::User);

    for (index, cells) in table.iter().enumerate().skip(def.data_start()) {
        let row = index + 1;
        let note = |problem| RowNote { row, problem };
        if cells.iter().all(|c| c.trim().is_empty()) {
            statement.ignored.push(note(RowProblem::Blank));
            continue;
        }
        let Some(parsed_date) = parse_date(cell(cells, date)) else {
            if def.skip_unreadable_dates {
                statement.ignored.push(note(RowProblem::UnreadableDate));
            } else {
                statement.errors.push(note(RowProblem::UnreadableDate));
            }
            continue;
        };
        // 金額: 1 列なら符号の向きで、2 列なら 出金 − 入金
        let value = if amount.is_some() {
            parse_amount(cell(cells, amount)).map(|v| match def.sign {
                AmountSign::SpendPositive => v,
                AmountSign::SpendNegative => -v,
            })
        } else {
            let out = cell(cells, withdrawal);
            let back = cell(cells, deposit);
            let read = |text: &str| {
                if text.is_empty() {
                    Some(0)
                } else {
                    parse_amount(text)
                }
            };
            match (read(out), read(back)) {
                (Some(o), Some(b)) if !(out.is_empty() && back.is_empty()) => {
                    Some(o.abs() - b.abs())
                }
                _ => None,
            }
        };
        let Some(spend) = value else {
            statement.errors.push(note(RowProblem::UnreadableAmount));
            continue;
        };
        if spend == 0 {
            statement.ignored.push(note(RowProblem::ZeroAmount));
            continue;
        }
        let mut memo_parts: Vec<String> = Vec::new();
        for part in [cell(cells, memo), cell(cells, foreign), cell(cells, user)] {
            if !part.is_empty() {
                memo_parts.push(part.to_string());
            }
        }
        statement.rows.push(StatementRow {
            row,
            date: parsed_date,
            spend,
            merchant: cell(cells, merchant).to_string(),
            memo: memo_parts.join(" / "),
        });
    }
    statement
}

/// 見出しの行の文字列（目印に使う）。前後の空白は除き、末尾の空の列は落とす。
pub fn header_of(table: &[Vec<String>], header_row: usize) -> Vec<String> {
    if header_row == 0 {
        return Vec::new();
    }
    let mut header: Vec<String> = table
        .get(header_row - 1)
        .map(|r| r.iter().map(|c| c.trim().to_string()).collect())
        .unwrap_or_default();
    while header.last().is_some_and(|c| c.is_empty()) {
        header.pop();
    }
    header
}

/// 保存した形式の目印と、ファイルの見出しの違い。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaderDiff {
    pub expected: Vec<String>,
    pub found: Vec<String>,
    /// ファイルにあって、形式にない列
    pub added: Vec<String>,
    /// 形式にあって、ファイルにない列
    pub missing: Vec<String>,
}

/// 読まない列だけ、見出し先頭の `{month}月` を 1〜12 月の数字として扱う。
fn header_matches(expected: &str, actual: &str, ignored: bool) -> bool {
    if expected == actual {
        return true;
    }
    if !ignored {
        return false;
    }
    let Some(suffix) = expected.strip_prefix("{month}月") else {
        return false;
    };
    actual.split_once('月').is_some_and(|(month, rest)| {
        rest == suffix
            && matches!(
                month,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "12"
            )
    })
}

fn same_header_cell(def: &FormatDefinition, index: usize, expected: &str, actual: &str) -> bool {
    let ignored = def
        .columns
        .get(index)
        .is_none_or(|role| *role == ColumnRole::Ignore);
    header_matches(expected, actual, ignored)
}

/// 試し読みに使った見出しを保存する。合っている月のひな型は残す。
pub fn header_for_save(table: &[Vec<String>], def: &FormatDefinition) -> Vec<String> {
    match check_header(table, def) {
        Ok(()) if def.header_row > 0 => def.header.clone(),
        Ok(()) => Vec::new(),
        Err(diff) => diff.found,
    }
}

/// ファイルが保存した形式と合うか。見出しが同じ（並びも同じ）なら合う。
/// 見出しのない形式は、列の数で比べる。
pub fn check_header(table: &[Vec<String>], def: &FormatDefinition) -> Result<(), HeaderDiff> {
    let found = header_of(table, def.header_row);
    let same = if def.header_row == 0 {
        let width = table
            .iter()
            .skip(def.data_start())
            .find(|r| r.iter().any(|c| !c.trim().is_empty()))
            .map_or(0, Vec::len);
        width >= def.columns.len()
    } else {
        found.len() == def.header.len()
            && def
                .header
                .iter()
                .zip(&found)
                .enumerate()
                .all(|(index, (expected, actual))| same_header_cell(def, index, expected, actual))
    };
    if same {
        return Ok(());
    }
    Err(HeaderDiff {
        added: found
            .iter()
            .filter(|actual| {
                !def.header
                    .iter()
                    .enumerate()
                    .any(|(index, expected)| same_header_cell(def, index, expected, actual))
            })
            .cloned()
            .collect(),
        missing: def
            .header
            .iter()
            .enumerate()
            .filter(|(index, expected)| {
                !found
                    .iter()
                    .any(|actual| same_header_cell(def, *index, expected, actual))
            })
            .map(|(_, expected)| expected)
            .cloned()
            .collect(),
        expected: def.header.clone(),
        found,
    })
}

/// 見出しの名前から役割を推し量る。
fn role_from_header(header: &str) -> ColumnRole {
    let h: String = header
        .chars()
        .map(to_half_width)
        .collect::<String>()
        .to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| h.contains(w));
    if has(&["現地", "外貨", "foreign"]) {
        ColumnRole::ForeignAmount
    } else if has(&["出金", "お引出", "お支払金額", "withdraw", "debit"]) {
        ColumnRole::Withdrawal
    } else if has(&["入金", "お預入", "deposit", "credit"]) {
        ColumnRole::Deposit
    } else if has(&["処理日", "計上日", "支払日", "引落日"]) {
        // 利用日ではない日付
        ColumnRole::Ignore
    } else if has(&["利用日", "日付", "取引日", "年月日", "date"]) {
        ColumnRole::Date
    } else if has(&["金額", "利用額", "amount", "(円)"]) {
        ColumnRole::Amount
    } else if has(&["利用者", "会員", "カード名義", "user"]) {
        ColumnRole::User
    } else if has(&[
        "内容",
        "店名",
        "利用先",
        "加盟店",
        "摘要",
        "description",
        "merchant",
        "payee",
    ]) {
        ColumnRole::Merchant
    } else if has(&["備考", "メモ", "memo", "note"]) {
        ColumnRole::Memo
    } else {
        ColumnRole::Ignore
    }
}

/// 区切り文字を推し量る。最初の行で、タブ → セミコロン → カンマの順に多く出るもの。
pub fn guess_delimiter(text: &str) -> Delimiter {
    let lines: Vec<&str> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(20)
        .collect();
    let count = |c: char| lines.iter().map(|l| l.matches(c).count()).sum::<usize>();
    let (tabs, semis, commas) = (count('\t'), count(';'), count(','));
    if tabs > commas && tabs >= semis {
        Delimiter::Tab
    } else if semis > commas {
        Delimiter::Semicolon
    } else {
        Delimiter::Comma
    }
}

/// 表を見て、形式の見当を付ける（見出しの行、列の役割）。文字コードと区切り文字は呼び出し側で決める。
pub fn guess(
    table: &[Vec<String>],
    encoding: TextEncoding,
    delimiter: Delimiter,
) -> FormatDefinition {
    let first_data = table
        .iter()
        .take(50)
        .position(|r| r.iter().any(|c| parse_date(c).is_some()));
    // 見出し: 役割の分かる列が 2 つ以上ある行。なければ最初のデータ行の 1 つ前
    let header_row = table
        .iter()
        .take(30)
        .position(|r| {
            r.iter()
                .filter(|c| parse_date(c).is_none() && role_from_header(c) != ColumnRole::Ignore)
                .count()
                >= 2
        })
        .map(|i| i + 1)
        .or(first_data)
        .unwrap_or(0);

    let width = table.iter().map(Vec::len).max().unwrap_or(0);
    let mut columns: Vec<ColumnRole> = if header_row > 0 {
        let header = &table[header_row - 1];
        (0..width)
            .map(|i| {
                header
                    .get(i)
                    .map_or(ColumnRole::Ignore, |h| role_from_header(h))
            })
            .collect()
    } else {
        vec![ColumnRole::Ignore; width]
    };
    // 同じ役割は最初の列だけにする
    for i in 0..columns.len() {
        if columns[i] != ColumnRole::Ignore && columns[..i].contains(&columns[i]) {
            columns[i] = ColumnRole::Ignore;
        }
    }
    // 見出しから決まらない役割は、データから決める（最初の日付の列・数の列・文字の列）
    if let Some(sample) = first_data.and_then(|i| table.get(i)) {
        if !columns.contains(&ColumnRole::Date) {
            if let Some(i) = sample.iter().position(|c| parse_date(c).is_some()) {
                columns[i] = ColumnRole::Date;
            }
        }
        let has_amount = columns.iter().any(|r| {
            matches!(
                r,
                ColumnRole::Amount | ColumnRole::Withdrawal | ColumnRole::Deposit
            )
        });
        if !has_amount {
            if let Some(i) = (0..width).find(|&i| {
                columns[i] == ColumnRole::Ignore
                    && sample.get(i).is_some_and(|c| parse_amount(c).is_some())
            }) {
                columns[i] = ColumnRole::Amount;
            }
        }
        if !columns.contains(&ColumnRole::Merchant) {
            if let Some(i) = (0..width).find(|&i| {
                columns[i] == ColumnRole::Ignore
                    && sample.get(i).is_some_and(|c| {
                        !c.trim().is_empty() && parse_amount(c).is_none() && parse_date(c).is_none()
                    })
            }) {
                columns[i] = ColumnRole::Merchant;
            }
        }
    }
    while columns.last() == Some(&ColumnRole::Ignore) {
        columns.pop();
    }
    FormatDefinition {
        encoding,
        delimiter,
        header_row,
        skip_rows: 0,
        header: header_of(table, header_row),
        columns,
        sign: AmountSign::SpendPositive,
        skip_unreadable_dates: true,
    }
}

/// 重複を防ぐための行の鍵。口座・利用日・金額・店名（正規化したもの）と、ファイルの中で
/// 同じ内容の行が何回目か（同じ日に同じ店で同じ額を 2 回使ったときも 2 件にする）。
/// 期間の重なるファイルを読んでも、同じ行は同じ鍵になる。
pub fn row_keys(
    rows: &[StatementRow],
    account_key: &str,
    normalize: impl Fn(&str) -> String,
) -> Vec<String> {
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    rows.iter()
        .map(|r| {
            let base = format!(
                "{account_key}|{}|{}|{}",
                r.date,
                r.spend,
                normalize(&r.merchant)
            );
            let n = seen.entry(base.clone()).or_insert(0);
            *n += 1;
            format!("{base}|{n}")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(text: &str) -> Vec<Vec<String>> {
        text.lines()
            .map(|l| l.split(',').map(str::to_string).collect())
            .collect()
    }

    #[test]
    fn reads_dates_in_many_forms() {
        for (text, expected) in [
            ("2026/10/05", "2026-10-05"),
            ("2026-10-5", "2026-10-05"),
            ("20261005", "2026-10-05"),
            ("2026年10月5日", "2026-10-05"),
            ("２０２６／１０／０５", "2026-10-05"),
            ("令和8年10月5日", "2026-10-05"),
            ("R8.10.5", "2026-10-05"),
            ("平成26年1月5日", "2014-01-05"),
            ("H26/1/5", "2014-01-05"),
            ("令和元年5月1日", "2019-05-01"),
            ("2026/10/05 12:34", "2026-10-05"),
            // レシートの、桁をそろえる空白
            ("2026年11月 3日(火) 12:34", "2026-11-03"),
            ("2026年 7月 7日", "2026-07-07"),
            ("令和 8年10月15日", "2026-10-15"),
        ] {
            assert_eq!(parse_date(text).as_deref(), Some(expected), "{text}");
        }
        for text in [
            "",
            "合計",
            "2026/02/30",
            "10/05",
            "1234",
            "令和0年1月1日",
            "R9223372036854775807.1.1",
            "10000/01/01",
        ] {
            assert_eq!(parse_date(text), None, "{text}");
        }
    }

    #[test]
    fn reads_amounts_in_many_forms() {
        for (text, expected) in [
            ("1,234", 1234),
            ("¥1,234", 1234),
            ("￥１，２３４", 1234),
            ("-1,234", -1234),
            ("(1,234)", -1234),
            ("▲500", -500),
            ("1234円", 1234),
            ("1,234.00", 1234),
            ("+80", 80),
        ] {
            assert_eq!(parse_amount(text), Some(expected), "{text}");
        }
        for text in ["", "abc", "12.50", "1,2a"] {
            assert_eq!(parse_amount(text), None, "{text}");
        }
    }

    const CARD: &str = "ご利用明細\n\
        ご利用日,データ処理日,ご利用内容,ご利用金額（円）,現地通貨額\n\
        2026/10/01,2026/10/03,文具店　新宿,1080,\n\
        2026/10/02,2026/10/04,AMAZON.CO.JP,\"2,200\",\n\
        2026/10/05,2026/10/06,返品 文具店,-1080,\n\
        2026/10/07,2026/10/08,HOTEL PARIS,15000,USD 100.00\n\
        ,,合計,17200,\n";

    fn card_table() -> Vec<Vec<String>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(CARD.as_bytes());
        reader
            .records()
            .map(|r| r.unwrap().iter().map(str::to_string).collect())
            .collect()
    }

    #[test]
    fn positional_format_can_include_or_skip_the_first_data_row() {
        let rows = table("2026/10/01,文具店,1080\n2026/10/02,交通費,500");
        let mut def = guess(&rows, TextEncoding::Utf8, Delimiter::Comma);
        assert_eq!(def.header_row, 0);
        assert_eq!(read(&rows, &def).rows[0].row, 1);

        def.skip_rows = 1;
        assert!(check_header(&rows, &def).is_ok());
        let statement = read(&rows, &def);
        assert_eq!(statement.rows.len(), 1);
        assert_eq!(statement.rows[0].row, 2);
        assert_eq!(statement.rows[0].spend, 500);

        let rows = table("取引明細\n2026/10/02,交通費,500");
        assert!(check_header(&rows, &def).is_ok());
        assert!(check_header(&table("取引明細\n2026/10/02,交通費"), &def).is_err());
        assert!(check_header(&table("取引明細"), &def).is_err());
    }

    #[test]
    fn guesses_header_row_and_roles_from_names() {
        let def = guess(&card_table(), TextEncoding::ShiftJis, Delimiter::Comma);
        assert_eq!(def.header_row, 2);
        assert_eq!(
            def.columns,
            [
                ColumnRole::Date,
                ColumnRole::Ignore,
                ColumnRole::Merchant,
                ColumnRole::Amount,
                ColumnRole::ForeignAmount
            ]
        );
        assert!(def.missing_roles().is_empty());
        assert_eq!(def.header[0], "ご利用日");
    }

    #[test]
    fn reads_rows_skips_totals_and_keeps_refunds_and_foreign_amounts() {
        let t = card_table();
        let def = guess(&t, TextEncoding::ShiftJis, Delimiter::Comma);
        let s = read(&t, &def);
        assert!(s.errors.is_empty());
        assert_eq!(s.rows.len(), 4);
        assert_eq!(s.rows[1].spend, 2200);
        assert_eq!(s.rows[2].spend, -1080);
        assert_eq!(s.rows[3].memo, "USD 100.00");
        assert_eq!(
            (s.first_date(), s.last_date()),
            (Some("2026-10-01"), Some("2026-10-07"))
        );
        assert_eq!(s.totals(), (1080 + 2200 + 15000, 1080));
        // 合計の行は日付がないので飛ばす
        assert_eq!(
            s.ignored,
            [RowNote {
                row: 7,
                problem: RowProblem::UnreadableDate
            }]
        );
    }

    #[test]
    fn unreadable_dates_are_errors_when_not_skipped_and_bad_amounts_are_always_errors() {
        let t = table("日付,内容,金額\n2026/10/01,店,abc\nメモ,,\n");
        let mut def = guess(&t, TextEncoding::Utf8, Delimiter::Comma);
        def.skip_unreadable_dates = false;
        let s = read(&t, &def);
        assert_eq!(
            s.errors,
            [
                RowNote {
                    row: 2,
                    problem: RowProblem::UnreadableAmount
                },
                RowNote {
                    row: 3,
                    problem: RowProblem::UnreadableDate
                }
            ]
        );
    }

    #[test]
    fn bank_statements_with_withdrawal_and_deposit_columns() {
        let t = table("取引日,摘要,お引出し,お預入れ,残高\n2026/10/01,カード引落,30000,,70000\n2026/10/02,振込 授業料,,8000,78000\n");
        let def = guess(&t, TextEncoding::Utf8, Delimiter::Comma);
        assert_eq!(
            &def.columns[..4],
            [
                ColumnRole::Date,
                ColumnRole::Merchant,
                ColumnRole::Withdrawal,
                ColumnRole::Deposit
            ]
        );
        let s = read(&t, &def);
        assert_eq!(
            s.rows.iter().map(|r| r.spend).collect::<Vec<_>>(),
            [30000, -8000]
        );
    }

    #[test]
    fn spend_negative_statements_flip_the_sign() {
        let t = table("日付,内容,金額\n2026/10/01,店,-500\n2026/10/02,返金,200\n");
        let mut def = guess(&t, TextEncoding::Utf8, Delimiter::Comma);
        def.sign = AmountSign::SpendNegative;
        assert_eq!(
            read(&t, &def)
                .rows
                .iter()
                .map(|r| r.spend)
                .collect::<Vec<_>>(),
            [500, -200]
        );
    }

    #[test]
    fn header_changes_are_reported_as_a_diff() {
        let t = card_table();
        let def = guess(&t, TextEncoding::ShiftJis, Delimiter::Comma);
        assert!(check_header(&t, &def).is_ok());
        let mut changed = t.clone();
        changed[1] = ["ご利用日", "ご利用内容", "ご利用金額（円）", "ポイント"]
            .map(str::to_string)
            .to_vec();
        let diff = check_header(&changed, &def).unwrap_err();
        assert_eq!(diff.added, ["ポイント"]);
        assert_eq!(diff.missing, ["データ処理日", "現地通貨額"]);
    }

    #[test]
    fn row_keys_count_identical_rows_so_overlapping_files_line_up() {
        let row = |date: &str, spend, merchant: &str| StatementRow {
            row: 0,
            date: date.into(),
            spend,
            merchant: merchant.into(),
            memo: String::new(),
        };
        let first = [
            row("2026-10-01", 500, "カフェ"),
            row("2026-10-01", 500, "カフェ"),
            row("2026-10-02", 800, "書店"),
        ];
        let second = [
            row("2026-10-01", 500, "カフェ"),
            row("2026-10-01", 500, "カフェ"),
            row("2026-10-02", 800, "書店"),
            row("2026-10-03", 100, "店"),
        ];
        let id = |s: &str| s.to_string();
        let a = row_keys(&first, "card", id);
        let b = row_keys(&second, "card", id);
        assert_ne!(a[0], a[1]);
        assert_eq!(a[..], b[..3]);
    }

    #[test]
    fn delimiters_are_guessed() {
        assert_eq!(guess_delimiter("a\tb\tc\n1\t2\t3"), Delimiter::Tab);
        assert_eq!(guess_delimiter("a;b;c\n1;2;3"), Delimiter::Semicolon);
        assert_eq!(guess_delimiter("a,b,c\n1,2,3"), Delimiter::Comma);
    }
}
