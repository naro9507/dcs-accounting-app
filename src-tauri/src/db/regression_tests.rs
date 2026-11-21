use crate::db::setup::tests::{input, set_up_db};
use crate::db::transactions::tests::{account, entry};
use crate::db::{
    backup, books, closing, documents, filing, fixed_assets, locks, recurring, reports, setup,
    transactions, Database,
};
use crate::domain::{
    depreciation::Method,
    document::{DocumentKind, LineInput},
    master::TaxCategory,
};
use crate::error::ErrorCode;
use crate::export::{csv, labels::Locale, report::Names};
use crate::import::{self, journal_csv};

fn opening(account_id: i64, amount: i64) -> closing::OpeningRow {
    closing::OpeningRow { account_id, amount }
}

fn pair(db: &Database, left: &str, right: &str) -> (i64, i64) {
    let conn = db.conn();
    (account(&conn, left), account(&conn, right))
}

fn invoice(db: &Database, lines: &[(i64, i64)]) -> documents::Document {
    let draft = documents::create(
        db,
        &documents::DocumentInput {
            kind: DocumentKind::Invoice,
            partner_id: None,
            recipient_name: "Test Client".into(),
            recipient_address: String::new(),
            recipient_email: String::new(),
            issue_date: "2026-04-01".into(),
            due_date: None,
            title: "Audit".into(),
            notes: String::new(),
            lines: lines
                .iter()
                .map(|&(unit_price, tax_rate)| LineInput {
                    name: "Item".into(),
                    quantity: 100,
                    unit: String::new(),
                    unit_price,
                    tax_rate,
                })
                .collect(),
        },
    )
    .unwrap();
    documents::issue(db, draft.id, &|_, _| Ok(b"%PDF-fake".to_vec())).unwrap()
}

#[test]
fn unconfirmed_opening_is_excluded_from_trial_balance_and_ledger() {
    let db = set_up_db();
    let (cash, _) = pair(&db, "101", "302");
    let saved = closing::save_opening(&db, 2026, &[opening(cash, 10_000)]).unwrap();
    assert!(!saved.confirmed && !saved.totals.balanced);
    let conn = db.conn();
    assert_eq!(
        books::ledger(&conn, 2026, 1, 12, cash)
            .unwrap()
            .closing_balance,
        0
    );
    assert!(reports::trial_balance(&conn, 2026, 1, 12).unwrap().balanced);
}

#[test]
fn recording_inventory_again_updates_the_opening_posting() {
    let db = set_up_db();
    let (stock, capital) = pair(&db, "108", "302");
    closing::save_opening(
        &db,
        2026,
        &[opening(stock, 100_000), opening(capital, 100_000)],
    )
    .unwrap();
    closing::confirm_opening(&db, 2026, true).unwrap();
    closing::record_inventory(&db, 2026, 80_000).unwrap();
    closing::confirm_opening(&db, 2026, false).unwrap();
    closing::save_opening(
        &db,
        2026,
        &[opening(stock, 120_000), opening(capital, 120_000)],
    )
    .unwrap();
    closing::confirm_opening(&db, 2026, true).unwrap();
    let status = closing::record_inventory(&db, 2026, 80_000).unwrap();
    assert!(status.opening_recorded && status.recorded);
    assert_eq!(status.cost_of_sales, Some(40_000));
    let conn = db.conn();
    assert_eq!(
        books::ledger(&conn, 2026, 1, 12, stock)
            .unwrap()
            .closing_balance,
        80_000
    );
    assert_eq!(
        reports::trial_balance(&conn, 2026, 1, 12)
            .unwrap()
            .pre_tax_income,
        -40_000
    );
}

#[test]
fn carry_forward_requires_confirmation_before_replacing_saved_rows() {
    let db = set_up_db();
    let (cash, capital) = pair(&db, "101", "302");
    closing::carry_forward(&db, 2026, false).unwrap();
    closing::confirm_opening(&db, 2027, false).unwrap();
    closing::save_opening(
        &db,
        2027,
        &[opening(cash, 99_000), opening(capital, 99_000)],
    )
    .unwrap();
    assert_eq!(
        closing::carry_forward(&db, 2026, false).unwrap_err().code,
        ErrorCode::OpeningConfirmed
    );
    assert_eq!(closing::opening(&db.conn(), 2027).unwrap().rows.len(), 2);
    closing::carry_forward(&db, 2026, true).unwrap();
    let after = closing::opening(&db.conn(), 2027).unwrap();
    assert!(after.confirmed && after.rows.is_empty());
}

#[test]
fn editing_a_payment_updates_an_invoice_and_protects_the_sale() {
    let db = set_up_db();
    let (cash, ar) = pair(&db, "101", "106");
    let d = invoice(&db, &[(10_000, 10)]);
    let sale = documents::post_sale(&db, d.id).unwrap();
    documents::record_payment(
        &db,
        d.id,
        &documents::PaymentInput {
            date: "2026-04-02".into(),
            amount: 10_000,
            account_id: cash,
            sub_account_id: None,
        },
    )
    .unwrap();
    let payment = transactions::list(&db.conn(), 2026, Some(4))
        .unwrap()
        .into_iter()
        .find(|t| t.id != sale.transaction_id.unwrap())
        .unwrap();
    let account = payment.lines[1].account_id;
    assert_eq!(account, ar);
    transactions::update(&db, payment.id, &entry("2026-04-02", cash, account, 4_000)).unwrap();
    let updated = documents::get(&db.conn(), d.id).unwrap();
    assert_eq!(updated.paid, 4_000);
    let conn = db.conn();
    assert_eq!(
        books::ledger(&conn, 2026, 1, 12, ar)
            .unwrap()
            .closing_balance,
        6_000
    );
    drop(conn);
    assert_eq!(
        transactions::delete(&db, sale.transaction_id.unwrap())
            .unwrap_err()
            .code,
        ErrorCode::HasPayments
    );
    assert_eq!(
        transactions::update(&db, payment.id, &entry("2026-04-02", cash, account, 11_000))
            .unwrap_err()
            .code,
        ErrorCode::OutOfRange
    );
    assert_eq!(documents::get(&db.conn(), d.id).unwrap().paid, 4_000);
    transactions::delete(&db, payment.id).unwrap();
    transactions::delete(&db, sale.transaction_id.unwrap()).unwrap();
    let after = documents::get(&db.conn(), d.id).unwrap();
    assert!(after.transaction_id.is_none());
    assert_eq!(after.paid, 0);
}

#[test]
fn recurring_invoice_payments_stay_at_the_received_amount_when_the_plan_changes() {
    let db = set_up_db();
    let (cash, sales) = pair(&db, "101", "401");
    let mut plan = recurring::PlanInput {
        partner_id: 1,
        name: "Monthly".into(),
        amount: 8_000,
        revenue_account_id: sales,
        start_month: "2026-04".into(),
        end_month: None,
        memo: String::new(),
    };
    let id = {
        let c = db.conn();
        c.execute(
            "INSERT INTO partners (id, code, name_ja, is_customer) VALUES (1, '1', 'Client', 1)",
            [],
        )
        .unwrap();
        recurring::create(&c, &plan).unwrap().id
    };
    let d = recurring::draft_invoices(&db, 2026, 4).unwrap().documents[0].id;
    documents::issue(&db, d, &|_, _| Ok(b"%PDF-fake".to_vec())).unwrap();
    documents::post_sale(&db, d).unwrap();
    documents::record_payment(
        &db,
        d,
        &documents::PaymentInput {
            date: "2026-04-02".into(),
            amount: 8_000,
            account_id: cash,
            sub_account_id: None,
        },
    )
    .unwrap();
    assert_eq!(
        recurring::monthly(&db.conn(), 2026, 4).unwrap().total_net,
        8_000
    );
    plan.amount = 12_000;
    recurring::update(&db.conn(), id, &plan).unwrap();
    assert_eq!(
        recurring::monthly(&db.conn(), 2026, 4).unwrap().total_net,
        8_000
    );
    assert_eq!(documents::get(&db.conn(), d).unwrap().paid, 8_000);
}

#[test]
fn invoice_discount_preserves_the_tax_buckets() {
    let db = set_up_db();
    db.conn()
        .execute("UPDATE fiscal_years SET tax_status = 'standard'", [])
        .unwrap();
    let d = invoice(&db, &[(10_000, 8), (-1_000, 0)]);
    assert_eq!(d.totals.total, 9_800);
    let d = documents::post_sale(&db, d.id).unwrap();
    let conn = db.conn();
    let sale = transactions::get(&conn, d.transaction_id.unwrap()).unwrap();
    assert_eq!(sale.lines[1].tax_category, Some(TaxCategory::TaxableSales8));
    let report = filing::consumption(&conn, 2026).unwrap();
    assert_eq!(report.accounts[0].tax_included, 10_800);
    assert_eq!(
        reports::trial_balance(&conn, 2026, 1, 12)
            .unwrap()
            .pre_tax_income,
        9_800
    );
}

#[test]
fn depreciation_postings_protect_the_closed_year_from_asset_edits() {
    let db = set_up_db();
    let (tools, _) = pair(&db, "115", "101");
    let mut asset = fixed_assets::tests::pc(tools);
    let id = fixed_assets::create(&db.conn(), &asset).unwrap().id;
    assert_eq!(fixed_assets::record(&db, 2026, &[id]).unwrap(), 1);
    locks::set_year_closed(&db, 2026, true).unwrap();
    asset.business_percent = 100;
    assert_eq!(
        fixed_assets::update(&db.conn(), id, &asset)
            .unwrap_err()
            .code,
        ErrorCode::InUse
    );
    let conn = db.conn();
    let report = closing::statement(&conn, 2026).unwrap();
    assert_eq!(report.depreciation[0].business, 27_000);
    assert_eq!(
        reports::trial_balance(&conn, 2026, 1, 12)
            .unwrap()
            .pre_tax_income,
        -27_000
    );
}

#[test]
fn small_asset_threshold_changes_in_april_2026() {
    let db = set_up_db();
    let (tools, _) = pair(&db, "115", "101");
    let mut a = fixed_assets::tests::pc(tools);
    a.cost = 350_000;
    a.method = Method::SmallAsset;
    assert!(fixed_assets::create(&db.conn(), &a).is_ok());
    a.acquired_on = "2026-03-31".into();
    let error = fixed_assets::create(&db.conn(), &a).unwrap_err();
    assert_eq!(
        (error.code, error.field),
        (ErrorCode::OutOfRange, Some("cost"))
    );
}

#[test]
fn small_asset_annual_limit_also_applies_across_batches() {
    let db = set_up_db();
    let (tools, _) = pair(&db, "115", "101");
    let mut a = fixed_assets::tests::pc(tools);
    a.cost = 290_000;
    a.business_percent = 100;
    a.method = Method::SmallAsset;
    let ids = {
        let conn = db.conn();
        (0..11)
            .map(|_| fixed_assets::create(&conn, &a).unwrap().id)
            .collect::<Vec<_>>()
    };
    assert_eq!(fixed_assets::record(&db, 2026, &ids[..9]).unwrap(), 9);
    assert_eq!(
        fixed_assets::record(&db, 2026, &ids[9..]).unwrap_err().code,
        ErrorCode::SmallAssetLimit
    );
    assert_eq!(fixed_assets::record(&db, 2026, &ids[9..10]).unwrap(), 1);
    assert_eq!(
        reports::trial_balance(&db.conn(), 2026, 1, 12)
            .unwrap()
            .pre_tax_income,
        -2_900_000
    );
}

#[test]
fn restore_does_not_depend_on_the_old_temporary_filename() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("books.db");
    let db = crate::db::vault::create(&path, "test password").unwrap();
    setup::complete(&db, &input()).unwrap();
    let file = backup::create(&db, dir.path(), backup::BackupKind::Manual).unwrap();
    let old = path.with_extension("db.replaced");
    std::fs::create_dir(&old).unwrap();
    backup::restore(&db, &dir.path().join(file), None, dir.path()).unwrap();
    assert!(old.is_dir());
    assert!(setup::status(&db.conn()).unwrap().completed);
    let reopened = crate::db::vault::unlock(&path, "test password").unwrap();
    assert!(setup::status(&reopened.conn()).unwrap().completed);
}

fn sample_csv(memo: &str, second_date: &str) -> String {
    format!("Transaction No,Date,Side,Account Code,Amount,Transaction memo\n1,2026-04-01,Debit,101,1500,{memo}\n1,{second_date},Credit,401,1500,{memo}\n")
}

#[test]
fn csv_preview_reports_locked_month() {
    let db = set_up_db();
    locks::set_month(&db, 2026, 4, true).unwrap();
    let parsed = journal_csv::parse(sample_csv("test", "2026-04-01").as_bytes()).unwrap();
    let p = import::preview(&db.conn(), &parsed, Some(2026), &[]).unwrap();
    assert_eq!((p.ready, p.issue_count), (0, 1));
    assert_eq!(p.issues[0].code, import::IssueCode::MonthLocked);
    assert_eq!(
        import::commit(&db, &parsed, Some(2026), &[])
            .unwrap_err()
            .code,
        ErrorCode::ImportBlocked
    );
}

#[test]
fn distinct_csv_memo_preserves_both_transactions() {
    let db = set_up_db();
    let (cash, sales) = pair(&db, "101", "401");
    let mut t = entry("2026-04-01", cash, sales, 1_500);
    t.memo = "Client A".into();
    transactions::create(&db, 2026, &t).unwrap();
    let b = sample_csv("Client B", "2026-04-01");
    let parsed = journal_csv::parse(b.as_bytes()).unwrap();
    let r = import::commit(&db, &parsed, Some(2026), &[]).unwrap();
    assert_eq!((r.imported, r.duplicates), (1, 0));
    let again = import::commit(&db, &parsed, Some(2026), &[]).unwrap();
    assert_eq!((again.imported, again.duplicates), (0, 1));
}

#[test]
fn csv_rejects_inconsistent_dates_within_one_transaction() {
    let db = set_up_db();
    let parsed = journal_csv::parse(sample_csv("test", "2027-01-01").as_bytes()).unwrap();
    assert_eq!(parsed.issues[0].code, import::IssueCode::InconsistentDate);
    assert_eq!(
        import::commit(&db, &parsed, Some(2026), &[])
            .unwrap_err()
            .code,
        ErrorCode::ImportBlocked
    );
    assert!(transactions::list(&db.conn(), 2026, None)
        .unwrap()
        .is_empty());
}

#[test]
fn csv_roundtrip_preserves_a_literal_leading_apostrophe() {
    let db = set_up_db();
    let (cash, sales) = pair(&db, "101", "401");
    let mut t = entry("2026-04-01", cash, sales, 1_500);
    t.memo = "'-test".into();
    transactions::create(&db, 2026, &t).unwrap();
    let conn = db.conn();
    let names = Names::new(
        crate::db::accounts::list(&conn).unwrap(),
        Vec::new(),
        Locale::En,
    );
    let b = csv::journal(
        &books::journal(&conn, 2026, 1, 12).unwrap(),
        &names,
        Locale::En,
    )
    .unwrap();
    let parsed = journal_csv::parse(b.as_bytes()).unwrap();
    assert_eq!(parsed.drafts[0].memo, "'-test");
}

#[test]
fn statement_cannot_be_matched_to_another_bank_subaccount() {
    use crate::db::{formats, inbox};
    use crate::domain::statement::StatementRow;
    let db = set_up_db();
    let (bank, expense) = pair(&db, "104", "610");
    let (a, b, format) = {
        let conn = db.conn();
        conn.execute("INSERT INTO sub_accounts (id, account_id, code, name_ja) VALUES (1, ?1, 'A', 'Bank A'), (2, ?1, 'B', 'Bank B')", [bank]).unwrap();
        let mut format = formats::tests::bank_format(&conn);
        format.payment_account_id = Some(bank);
        format.payment_sub_account_id = Some(1);
        (1, 2, format)
    };
    let mut t = entry("2026-04-01", expense, bank, 1_500);
    t.lines[1].sub_account_id = Some(b);
    let t = transactions::create(&db, 2026, &t).unwrap();
    let item = {
        let conn = db.conn();
        inbox::add_statement(
            &conn,
            &format,
            "bank-a.csv",
            &[StatementRow {
                row: 2,
                date: "2026-04-01".into(),
                spend: 1_500,
                merchant: "Test store".into(),
                memo: String::new(),
            }],
        )
        .unwrap();
        inbox::list(&conn, None).unwrap().remove(0)
    };
    assert_eq!(item.payment_sub_account_id, Some(a));
    assert_eq!(item.possible_duplicate_id, None);
    assert_eq!(
        inbox::link(&db, item.id, t.id).unwrap_err().code,
        ErrorCode::OutOfRange
    );
}

#[test]
fn an_already_taxable_year_keeps_sales_before_invoice_registration() {
    use crate::domain::master::TaxStatus;
    let db = set_up_db();
    db.conn()
        .execute("UPDATE fiscal_years SET tax_status = 'standard'", [])
        .unwrap();
    let (cash, sales) = pair(&db, "101", "401");
    transactions::create(&db, 2026, &entry("2026-03-01", cash, sales, 110_000)).unwrap();
    assert_eq!(
        filing::consumption(&db.conn(), 2026).unwrap().tax.rates[0].tax_included,
        110_000
    );
    filing::save_registration(
        &db,
        &filing::RegistrationInput {
            registration_no: "T1234567890123".into(),
            registered_on: "2026-06-01".into(),
            fiscal_year: 2026,
            tax_status: TaxStatus::Standard,
            simplified_category: None,
            was_exempt: None,
        },
    )
    .unwrap();
    let report = filing::consumption(&db.conn(), 2026).unwrap();
    assert_eq!(report.taxable_from.as_deref(), Some("2026-01-01"));
    assert_eq!(report.tax.rates[0].tax_included, 110_000);
    filing::save_registration(
        &db,
        &filing::RegistrationInput {
            registration_no: "T1234567890123".into(),
            registered_on: "2026-06-01".into(),
            fiscal_year: 2026,
            tax_status: TaxStatus::Standard,
            simplified_category: None,
            was_exempt: Some(true),
        },
    )
    .unwrap();
    assert_eq!(
        filing::taxable_from(&db.conn(), 2026).unwrap().as_deref(),
        Some("2026-06-01")
    );
}

#[test]
fn small_asset_special_treatment_is_rejected_in_white_filing() {
    use crate::domain::master::FilingType;
    let db = Database::open_in_memory().unwrap();
    let mut initial = input();
    initial.filing_type = FilingType::White;
    setup::complete(&db, &initial).unwrap();
    let (tools, _) = pair(&db, "115", "101");
    let mut a = fixed_assets::tests::pc(tools);
    a.cost = 250_000;
    a.business_percent = 100;
    a.method = Method::SmallAsset;
    assert_eq!(
        fixed_assets::create(&db.conn(), &a).unwrap_err().code,
        ErrorCode::SmallAssetIneligible
    );
}

#[test]
fn statement_import_rejects_more_than_100000_csv_records() {
    use crate::domain::statement::{Delimiter, TextEncoding};
    let mut contents = String::from("Date,Memo,Amount\n");
    contents.push_str(&"2026-04-01,Test store,100\n".repeat(99_999));
    contents.push_str("2026-04-02,Unique last row,900\n");
    assert!(contents.len() < 20 * 1024 * 1024);
    assert_eq!(
        import::statement_file::table(
            contents.as_bytes(),
            "test.csv",
            TextEncoding::Utf8,
            Delimiter::Comma
        )
        .unwrap_err()
        .code,
        ErrorCode::TooManyStatementRows
    );
}

#[test]
fn attachment_is_shared_and_can_be_detached_independently() {
    use crate::db::attachments;
    let db = set_up_db();
    let (cash, sales) = pair(&db, "101", "401");
    let a = transactions::create(&db, 2026, &entry("2026-04-01", cash, sales, 500)).unwrap();
    let b = transactions::create(&db, 2026, &entry("2026-04-02", cash, sales, 500)).unwrap();
    let conn = db.conn();
    let bytes = b"%PDF-test";
    let (first, _) = attachments::add(&conn, "first.pdf", bytes).unwrap();
    attachments::attach(&conn, first.id, a.id).unwrap();
    let (second, existed) = attachments::add(&conn, "second.pdf", bytes).unwrap();
    assert!(existed && first.id == second.id);
    attachments::attach(&conn, second.id, b.id).unwrap();
    assert_eq!(
        attachments::list_for_transaction(&conn, a.id)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        attachments::list_for_transaction(&conn, b.id)
            .unwrap()
            .len(),
        1
    );
    attachments::detach(&conn, second.id, b.id).unwrap();
    assert!(attachments::list_for_transaction(&conn, b.id)
        .unwrap()
        .is_empty());
    assert_eq!(
        attachments::list_for_transaction(&conn, a.id)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn migration_preserves_legacy_attachment_and_balanced_opening() {
    use rusqlite::Connection;
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    for sql in &super::MIGRATIONS[..11] {
        conn.execute_batch(sql).unwrap();
    }
    conn.execute_batch(
        "PRAGMA user_version = 11;
         INSERT INTO fiscal_years (year, filing_type, tax_status) VALUES (2026, 'blue65', 'exempt');
         INSERT INTO accounts (id, code, name_ja, category) VALUES
            (1, '101', 'Cash', 'asset'), (2, '302', 'Capital', 'equity');
         INSERT INTO opening_balances (fiscal_year, account_id, amount) VALUES
            (2026, 1, 500), (2026, 2, 500);
         INSERT INTO transactions (id, fiscal_year, date, kind, entry_mode)
            VALUES (1, 2026, '2026-05-01', 'normal', 'grid');
         INSERT INTO attachments (id, transaction_id, file_name, mime, size, sha256, data)
            VALUES (1, 1, 'receipt.pdf', 'application/pdf', 1, 'abc', X'00');",
    )
    .unwrap();
    let db = Database::init(conn).unwrap();
    {
        let conn = db.conn();
        assert_eq!(
            closing::opening_map(&conn, 2026).unwrap().get(&1),
            Some(&500)
        );
        assert_eq!(
            super::attachments::list_for_transaction(&conn, 1).unwrap()[0].id,
            1
        );
        assert!(!closing::opening(&conn, 2026).unwrap().confirmed);
    }
    closing::save_opening(&db, 2026, &[opening(1, 600), opening(2, 600)]).unwrap();
    assert!(closing::opening_map(&db.conn(), 2026).unwrap().is_empty());
}

#[test]
fn special_thirty_percent_is_available_only_in_2027_and_2028() {
    use crate::domain::filing::{consumption_tax, Rate, SalesAmount};
    let sales = [SalesAmount {
        rate: Rate::Standard10,
        category: 5,
        tax_included: 3_300_000,
    }];
    assert!(consumption_tax(2026, &sales).special30.is_none());
    for year in [2027, 2028] {
        let tax = consumption_tax(year, &sales);
        assert_eq!(tax.special30.unwrap().total, 90_000);
        assert!(tax.special20.is_none());
    }
    assert!(consumption_tax(2029, &sales).special30.is_none());
}
