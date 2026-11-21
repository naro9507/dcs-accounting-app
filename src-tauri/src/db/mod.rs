//! 帳簿のデータベース（SQLite + SQLCipher）。
//!
//! 接続は 1 本だけ持ち、`Mutex` で順番に使う。個人の帳簿は書き込みが少なく、
//! 1 本で十分に速い。スキーマは `migrations/` の SQL を順に当てて作る。

pub mod accounts;
pub mod allocation_rules;
pub mod attachments;
pub mod audit;
pub mod backup;
pub mod books;
pub mod closing;
pub mod deposits;
pub mod documents;
pub mod filing;
pub mod fixed_assets;
pub mod formats;
pub mod inbox;
pub mod locks;
pub mod partners;
pub mod purposes;
pub mod receipts;
pub mod received_bills;
pub mod recovery;
pub mod recurring;
pub mod reports;
pub mod rules;
pub mod setup;
pub mod sub_accounts;
pub mod templates;
pub mod transactions;
pub mod unlock_guard;
pub mod vault;

use crate::error::{AppError, AppResult};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

/// スキーマの変更。番号順に 1 回だけ当てる。適用済みの番号は `PRAGMA user_version` に持つ。
/// 一度配布したファイルは書き換えず、変更は新しいファイルとして足す。
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/0001_init.sql"),
    include_str!("migrations/0002_journal.sql"),
    include_str!("migrations/0003_templates.sql"),
    include_str!("migrations/0004_inbox.sql"),
    include_str!("migrations/0005_receipts.sql"),
    include_str!("migrations/0006_closing.sql"),
    include_str!("migrations/0007_security.sql"),
    include_str!("migrations/0008_documents.sql"),
    include_str!("migrations/0009_filing.sql"),
    include_str!("migrations/0010_recurring.sql"),
    include_str!("migrations/0011_payments.sql"),
    include_str!("migrations/0012_accounting_consistency.sql"),
    include_str!("migrations/0013_receipt_points.sql"),
    include_str!("migrations/0014_document_mail_template.sql"),
    include_str!("migrations/0015_received_bills.sql"),
];

pub struct Database {
    conn: Mutex<Connection>,
    /// 帳簿のファイル。メモリ上の帳簿（テスト）なら None
    path: Option<PathBuf>,
    /// 暗号化の鍵（起動時のパスワード）。バックアップ・復元・パスワードの変更に使う。
    /// 画面・ログには出さない
    key: Mutex<Option<vault::Password>>,
    /// 最後にバックアップしたときの、この接続での変更の数。変わっていなければ終了時に作らない
    backed_up_changes: Mutex<u64>,
}

impl std::fmt::Debug for Database {
    /// 鍵は出さない。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

/// ファイルを開いて設定する（マイグレーションはしない）。鍵が違うと、最初の読み込みで失敗する。
pub(crate) fn connect(path: &Path, key: Option<&str>) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    if let Some(key) = key {
        conn.pragma_update(None, "key", key)?;
    }
    configure(&conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> AppResult<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |_| Ok(()))?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

impl Database {
    /// ファイルを開く（なければ作る）。`key` を渡すと SQLCipher で暗号化する。
    pub fn open(path: &Path, key: Option<&str>) -> AppResult<Self> {
        let conn = connect(path, key)?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: Some(path.to_path_buf()),
            key: Mutex::new(key.map(vault::Password::new)),
            backed_up_changes: Mutex::new(0),
        })
    }

    /// テスト用に、メモリ上に作る。
    pub fn open_in_memory() -> AppResult<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> AppResult<Self> {
        configure(&conn)?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: None,
            key: Mutex::new(None),
            backed_up_changes: Mutex::new(0),
        })
    }

    /// 帳簿のファイル。
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// 暗号化の鍵。暗号化していなければ None。
    pub(crate) fn key(&self) -> Option<vault::Password> {
        self.key.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    pub(crate) fn set_key(&self, key: Option<vault::Password>) {
        *self.key.lock().unwrap_or_else(|p| p.into_inner()) = key;
    }

    /// 前のバックアップのあとに変更があったか。
    fn changed_since_backup(&self, conn: &Connection) -> bool {
        conn.total_changes()
            != *self
                .backed_up_changes
                .lock()
                .unwrap_or_else(|p| p.into_inner())
    }

    pub(crate) fn mark_backed_up(&self, conn: &Connection) {
        *self
            .backed_up_changes
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = conn.total_changes();
    }

    /// 接続を借りる。前の処理が途中で落ちていても、SQLite 側は
    /// トランザクション単位で整合しているので、そのまま使い続ける。
    pub fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// トランザクションの中で `f` を実行する。エラーなら何も書き込まない。
    pub fn transaction<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction<'_>) -> AppResult<T>,
    ) -> AppResult<T> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let value = f(&tx)?;
        tx.commit()?;
        Ok(value)
    }
}

fn migrate(conn: &Connection) -> AppResult<()> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current < 0 || current as usize > MIGRATIONS.len() {
        // 新しい版のアプリで作った DB を古い版で開いた
        eprintln!(
            "データベースの版 {current} は、このアプリが知っている版 {} より新しい",
            MIGRATIONS.len()
        );
        return Err(AppError::new(crate::error::ErrorCode::Database));
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let version = index + 1;
        conn.execute_batch(&format!(
            "BEGIN;\n{sql}\nPRAGMA user_version = {version};\nCOMMIT;"
        ))
        .inspect_err(|_| {
            let _ = conn.execute_batch("ROLLBACK;");
        })?;
    }
    Ok(())
}

/// 真偽値の列（0 / 1）を読むための補助。
pub(crate) fn flag(value: bool) -> i64 {
    i64::from(value)
}

#[cfg(test)]
mod regression_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_tables_and_set_version() {
        let db = Database::open_in_memory().unwrap();
        let conn = db.conn();
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version as usize, MIGRATIONS.len());
        let tables: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN ('accounts', 'partners', 'journal_lines')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(tables, 3);
    }

    #[test]
    fn migration_0002_marks_standard_cash_accounts_in_existing_books() {
        // フェーズ 1 の版で作った帳簿（0001 だけ当たっている）
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(MIGRATIONS[0]).unwrap();
        conn.execute_batch(
            "PRAGMA user_version = 1;
             INSERT INTO accounts (code, name_ja, category, is_system) VALUES
                 ('101', '現金', 'asset', 1),
                 ('610', '消耗品費', 'expense', 1),
                 ('204', '未払金', 'liability', 1),
                 ('900', '自分で足した科目', 'asset', 0);",
        )
        .unwrap();
        let db = Database::init(conn).unwrap();
        let conn = db.conn();
        let mut stmt = conn
            .prepare("SELECT code FROM accounts WHERE is_payment_account = 1 ORDER BY code")
            .unwrap();
        let codes: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(codes, ["101", "204"]);
    }

    #[test]
    fn migration_0003_adds_templates_to_existing_books() {
        // フェーズ 3 の版で作った帳簿（0002 まで当たっている）
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(MIGRATIONS[0]).unwrap();
        conn.execute_batch(MIGRATIONS[1]).unwrap();
        conn.execute_batch("PRAGMA user_version = 2;").unwrap();
        let db = Database::init(conn).unwrap();
        let count: i64 = db
            .conn()
            .query_row("SELECT count(*) FROM templates", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn reopening_does_not_reapply_migrations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("books.db");
        Database::open(&path, None).unwrap();
        // 2 回目に CREATE TABLE が走ると失敗する
        Database::open(&path, None).unwrap();
    }

    #[test]
    fn encrypted_database_cannot_be_read_without_the_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("books.db");
        {
            let db = Database::open(&path, Some("correct horse battery staple")).unwrap();
            db.conn()
                .execute("INSERT INTO settings (key, value) VALUES ('a', 'b')", [])
                .unwrap();
        }
        assert!(Database::open(&path, None).is_err());
        assert!(Database::open(&path, Some("wrong key")).is_err());

        let db = Database::open(&path, Some("correct horse battery staple")).unwrap();
        let value: String = db
            .conn()
            .query_row("SELECT value FROM settings WHERE key = 'a'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(value, "b");
    }

    #[test]
    fn foreign_keys_are_enforced() {
        let db = Database::open_in_memory().unwrap();
        let result = db.conn().execute(
            "INSERT INTO sub_accounts (account_id, code, name_ja) VALUES (999, '1', 'x')",
            [],
        );
        assert!(result.is_err());
    }
}
