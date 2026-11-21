//! 帳簿を開く（起動時のパスワード）・バックアップと復元・月締め・変更履歴のコマンド。
//!
//! 帳簿（`Database`）は、パスワードで開けたときに初めてアプリに登録する。それまでは帳簿を使う
//! コマンドは動かない（画面はパスワードの画面だけを出す）。

use super::files;
use crate::db::audit::{self, AuditFilters, AuditPage};
use crate::db::backup::{self, BackupFile, BackupKind, BackupSummary};
use crate::db::locks::{self, LockStatus};
use crate::db::recovery;
use crate::db::unlock_guard::{self, Attempts};
use crate::db::vault::{self, BookState};
use crate::db::Database;
use crate::error::{AppError, AppResult, ErrorCode};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

/// 帳簿のファイルと、バックアップの既定の保存先。起動したときに決める。
pub struct BookFiles {
    pub path: PathBuf,
    pub default_backup_dir: PathBuf,
    /// 起動・パスワード変更・復旧コードの発行・帳簿の入れ替えを直列に行う。
    pub access: Mutex<()>,
}

/// 復元に選んだファイル。場所は画面に渡さず、ここで覚えておく。
#[derive(Default)]
pub struct RestoreState(Mutex<Option<PathBuf>>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookStatus {
    pub state: BookState,
    /// パスワードで開けたあとか
    pub unlocked: bool,
    /// パスワード・復旧コードの入力の制限
    #[serde(flatten)]
    pub attempts: Attempts,
}

#[tauri::command]
pub async fn book_status(app: AppHandle, files: State<'_, BookFiles>) -> AppResult<BookStatus> {
    Ok(BookStatus {
        state: vault::state(&files.path)?,
        unlocked: app.try_state::<Database>().is_some(),
        attempts: unlock_guard::status(&unlock_guard::path_for(&files.path)),
    })
}

/// 開けた帳簿をアプリに登録する。2 回目以降は何もしない（もう開いている）。
pub(super) fn open_books(app: &AppHandle, db: Database) {
    db.mark_backed_up(&db.conn());
    app.manage(db);
}

pub(super) fn ensure_not_open(app: &AppHandle) -> AppResult<()> {
    if app.try_state::<Database>().is_some() {
        return Err(AppError::new(ErrorCode::AlreadySetUp));
    }
    Ok(())
}

/// 初めての起動: パスワードを決めて、暗号化した帳簿を作る。
#[tauri::command]
pub async fn create_books(
    app: AppHandle,
    files: State<'_, BookFiles>,
    password: String,
) -> AppResult<()> {
    let _access = files.access.lock().unwrap_or_else(|p| p.into_inner());
    ensure_not_open(&app)?;
    let db = vault::create(&files.path, &password)?;
    open_books(&app, db);
    Ok(())
}

/// パスワードを入れて帳簿を開く。
#[tauri::command]
pub async fn unlock_books(
    app: AppHandle,
    files: State<'_, BookFiles>,
    password: String,
) -> AppResult<()> {
    let _access = files.access.lock().unwrap_or_else(|p| p.into_inner());
    ensure_not_open(&app)?;
    let db = unlock_guard::guard(
        &unlock_guard::path_for(&files.path),
        "password",
        ErrorCode::WrongPassword,
        || vault::unlock(&files.path, &password),
    )?;
    open_books(&app, db);
    Ok(())
}

/// 前の版で作った（暗号化していない）帳簿に、パスワードを決めて暗号化する。
/// 暗号化できたら、すぐにバックアップを 1 つ作る。
#[tauri::command]
pub async fn encrypt_books(
    app: AppHandle,
    files: State<'_, BookFiles>,
    password: String,
) -> AppResult<()> {
    let _access = files.access.lock().unwrap_or_else(|p| p.into_inner());
    ensure_not_open(&app)?;
    let db = vault::encrypt_existing(&files.path, &password)?;
    let dir = backup::dir(&db.conn(), &files.default_backup_dir)?;
    // バックアップに失敗しても、暗号化した帳簿は開ける
    let _ = backup::create(&db, &dir, BackupKind::Manual);
    open_books(&app, db);
    Ok(())
}

#[tauri::command]
pub async fn change_password(
    db: State<'_, Database>,
    login: State<'_, super::account_login::LoginState>,
    files: State<'_, BookFiles>,
    current: String,
    new: String,
) -> AppResult<()> {
    let _guard = login.0.lock().await;
    let updated = if vault::validate_password_change(&db, &current, &new)?.is_some() {
        super::account_login::prepare_password_change(&db, &current, &new).await?
    } else {
        None
    };
    let _access = files.access.lock().unwrap_or_else(|p| p.into_inner());
    vault::change_password(&db, &current, &new)?;
    if let Some(updated) = updated {
        super::account_login::finish_password_change(&updated);
    }
    audit::event(&db.conn(), "books", "change_password", None, None)
}

#[tauri::command]
pub async fn issue_recovery_code(
    db: State<'_, Database>,
    files: State<'_, BookFiles>,
    current: String,
) -> AppResult<String> {
    let _access = files.access.lock().unwrap_or_else(|p| p.into_inner());
    recovery::issue(&db, &current)
}

#[tauri::command]
pub async fn reset_books_password(
    login: State<'_, super::account_login::LoginState>,
    app: AppHandle,
    files: State<'_, BookFiles>,
    recovery_code: String,
    new: String,
) -> AppResult<()> {
    let _guard = login.0.lock().await;
    let _access = files.access.lock().unwrap_or_else(|p| p.into_inner());
    ensure_not_open(&app)?;
    let db = unlock_guard::guard(
        &unlock_guard::path_for(&files.path),
        "recoveryCode",
        ErrorCode::InvalidRecoveryCode,
        || recovery::reset(&files.path, &recovery_code, &new),
    )?;
    super::account_login::discard_device_record();
    // 履歴の書き込みに失敗しても、再設定したパスワードで帳簿を開けるようにする。
    if let Err(error) = audit::event(&db.conn(), "books", "change_password", None, None) {
        eprintln!("パスワード再設定の履歴を保存できませんでした: {error}");
    }
    app.manage(db);
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSettings {
    /// 保存先のフォルダ（画面に見せる）
    pub dir: String,
    pub is_default: bool,
    pub files: Vec<BackupFile>,
    /// 帳簿のファイルの大きさ（バイト）。証憑の画像も含む
    pub book_size: u64,
}

fn backup_dir(db: &Database, files: &BookFiles) -> AppResult<PathBuf> {
    backup::dir(&db.conn(), &files.default_backup_dir)
}

#[tauri::command]
pub async fn backup_settings(
    db: State<'_, Database>,
    files: State<'_, BookFiles>,
) -> AppResult<BackupSettings> {
    let dir = backup_dir(&db, &files)?;
    Ok(BackupSettings {
        is_default: dir == files.default_backup_dir,
        dir: dir.to_string_lossy().into_owned(),
        files: backup::list(&dir)?,
        book_size: book_size(&files.path),
    })
}

/// 帳簿の大きさ。WAL のファイルにまだ書き戻していない分も含める。
fn book_size(path: &std::path::Path) -> u64 {
    let mut wal = path.as_os_str().to_owned();
    wal.push("-wal");
    [path.to_path_buf(), PathBuf::from(wal)]
        .iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum()
}

/// いますぐバックアップを作る。作ったファイルの名前を返す。
#[tauri::command]
pub async fn backup_now(db: State<'_, Database>, files: State<'_, BookFiles>) -> AppResult<String> {
    let dir = backup_dir(&db, &files)?;
    backup::create(&db, &dir, BackupKind::Manual)
}

/// 保存先のフォルダを選ぶ。やめたら false。
#[tauri::command]
pub async fn choose_backup_dir(app: AppHandle, db: State<'_, Database>) -> AppResult<bool> {
    let Some(dir) = files::pick_folder(app).await else {
        return Ok(false);
    };
    backup::set_dir(&db.conn(), Some(&dir))?;
    Ok(true)
}

#[tauri::command]
pub async fn reset_backup_dir(db: State<'_, Database>) -> AppResult<()> {
    backup::set_dir(&db.conn(), None)
}

/// 復元するファイルを、ほかの場所から選ぶ。やめたら None。
#[tauri::command]
pub async fn choose_restore_file(
    app: AppHandle,
    state: State<'_, RestoreState>,
) -> AppResult<Option<String>> {
    let Some(path) = files::open_path(
        app,
        (
            "Chobiyori backup",
            &[backup::EXTENSION, backup::LEGACY_EXTENSION],
        ),
    )
    .await
    else {
        return Ok(None);
    };
    let name = files::file_name(&path);
    *state.0.lock().unwrap_or_else(|p| p.into_inner()) = Some(path);
    Ok(Some(name))
}

/// 保存先にあるバックアップを、復元するファイルに選ぶ。
#[tauri::command]
pub async fn select_backup(
    db: State<'_, Database>,
    files: State<'_, BookFiles>,
    state: State<'_, RestoreState>,
    name: String,
) -> AppResult<()> {
    // 一覧にある名前だけを受け付ける（ほかの場所のファイルは読まない）
    let dir = backup_dir(&db, &files)?;
    if !backup::list(&dir)?.iter().any(|f| f.name == name) {
        return Err(AppError::new(ErrorCode::NotFound));
    }
    *state.0.lock().unwrap_or_else(|p| p.into_inner()) = Some(dir.join(name));
    Ok(())
}

fn chosen(state: &RestoreState) -> AppResult<PathBuf> {
    state
        .0
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
        .ok_or(AppError::field(ErrorCode::Required, "file"))
}

/// 選んだバックアップの中身を見る。`password` はそのバックアップのパスワード（None ならいまの）。
#[tauri::command]
pub async fn inspect_backup(
    db: State<'_, Database>,
    state: State<'_, RestoreState>,
    password: Option<String>,
) -> AppResult<BackupSummary> {
    let path = chosen(&state)?;
    let key = password.or_else(|| db.key().map(|k| k.as_str().to_string()));
    backup::inspect(&path, key.as_deref())
}

/// 選んだバックアップから復元する。入れ替える前に、いまの帳簿のバックアップを作る。
#[tauri::command]
pub async fn restore_backup(
    login: State<'_, super::account_login::LoginState>,
    db: State<'_, Database>,
    files: State<'_, BookFiles>,
    state: State<'_, RestoreState>,
    password: Option<String>,
) -> AppResult<()> {
    let _guard = login.0.lock().await;
    let _access = files.access.lock().unwrap_or_else(|p| p.into_inner());
    let path = chosen(&state)?;
    let dir = backup_dir(&db, &files)?;
    backup::restore(&db, &path, password.as_deref(), &dir)?;
    super::account_login::discard_device_record();
    *state.0.lock().unwrap_or_else(|p| p.into_inner()) = None;
    Ok(())
}

/// 終了するときの自動バックアップ。失敗しても終了は止めない（ログには名前や金額を出さない）。
pub fn backup_on_exit(app: &AppHandle) {
    let (Some(db), Some(files)) = (app.try_state::<Database>(), app.try_state::<BookFiles>())
    else {
        return;
    };
    let result = backup_dir(&db, &files).and_then(|dir| backup::auto(&db, &dir));
    if let Err(error) = result {
        eprintln!("終了時のバックアップに失敗しました: {error}");
    }
}

#[tauri::command]
pub async fn lock_status(db: State<'_, Database>, fiscal_year: i64) -> AppResult<LockStatus> {
    locks::status(&db.conn(), fiscal_year)
}

#[tauri::command]
pub async fn set_month_lock(
    db: State<'_, Database>,
    fiscal_year: i64,
    month: u32,
    locked: bool,
) -> AppResult<LockStatus> {
    locks::set_month(&db, fiscal_year, month, locked)
}

#[tauri::command]
pub async fn set_year_closed(
    db: State<'_, Database>,
    fiscal_year: i64,
    closed: bool,
) -> AppResult<LockStatus> {
    locks::set_year_closed(&db, fiscal_year, closed)
}

#[tauri::command]
pub async fn list_history(
    db: State<'_, Database>,
    fiscal_year: Option<i64>,
    page: i64,
    query: Option<String>,
    start_at: Option<i64>,
    end_before: Option<i64>,
) -> AppResult<AuditPage> {
    audit::list(
        &db.conn(),
        fiscal_year,
        page,
        AuditFilters {
            query: query.as_deref(),
            start_at,
            end_before,
        },
    )
}
