# プラットフォーム対応

## 🖥️ 対応OS

### ✅ 開発環境
- **macOS**: ✅ フル対応
- **Windows**: ⚠️ 一部制限あり
- **Linux**: ✅ フル対応

### ✅ 配布アプリ
- **macOS**: DMG, ZIP (Intel + Apple Silicon)
- **Windows**: NSIS installer, Portable
- **Linux**: AppImage, DEB

## 🪟 Windows開発環境

### 必要なツール
1. **Node.js 18+**: [公式サイト](https://nodejs.org/)
2. **Git**: [Git for Windows](https://gitforwindows.io/)
3. **Make**: 以下のいずれか
   - **推奨**: [Chocolatey](https://chocolatey.org/) → `choco install make`
   - **WSL**: Windows Subsystem for Linux
   - **MSYS2**: [公式サイト](https://www.msys2.org/)

### セットアップ手順（Windows）

#### 方法1: Chocolatey使用（推奨）
```powershell
# PowerShellを管理者として実行
Set-ExecutionPolicy Bypass -Scope Process -Force
iex ((New-Object System.Net.WebClient).DownloadString('https://chocolatey.org/install.ps1'))

# Makeインストール
choco install make

# プロジェクトセットアップ
git clone https://github.com/naro9507/dcs-accounting-app.git
cd dcs-accounting-app
make setup
```

#### 方法2: WSL使用
```bash
# WSLを有効化してUbuntuをインストール後
git clone https://github.com/naro9507/dcs-accounting-app.git
cd dcs-accounting-app
make setup
```

#### 方法3: npmコマンド使用（Makeなし）
```cmd
# コマンドプロンプトまたはPowerShell
git clone https://github.com/naro9507/dcs-accounting-app.git
cd dcs-accounting-app

# 手動セットアップ
npm install
npm run prisma:generate
npm run prisma:push

# 開発開始
npm run dev
```

## 🔧 Windows特有の問題と解決法

### 1. パス区切り文字
- **問題**: Windows uses `\`, Unix uses `/`
- **解決**: Node.js の `path` モジュールが自動対応済み

### 2. 権限問題
- **問題**: ファイル作成時の権限エラー
- **解決**: 管理者権限でターミナル実行

### 3. 長いパス名
- **問題**: Windowsのパス文字数制限
- **解決**: Git設定で長いパス名を有効化
```cmd
git config --system core.longpaths true
```

### 4. SQLite動作
- **確認済み**: Windows環境でSQLiteは正常動作
- **Prisma**: Windows完全対応

### 5. Electron
- **確認済み**: Windows向けアプリ生成可能
- **配布**: NSIS installer、Portable版対応

## 📋 開発コマンド対応表

| 操作 | Makefile (全OS) | Windows CMD/PowerShell |
|------|----------------|------------------------|
| セットアップ | `make setup` | `npm install && npm run prisma:generate && npm run prisma:push` |
| 開発開始 | `make dev` | `npm run dev` |
| ビルド | `make build` | `npm run build` |
| テスト | `make test` | `npm run test` |
| DB管理 | `make prisma-studio` | `npm run prisma:studio` |
| パッケージ作成 | `make dist` | `npm run dist` |

## 🚨 Windows制限事項

### ❌ 使用できない機能
- 暗号化機能の一部（file permission設定）
- Makefileの一部コマンド（makeがない場合）

### ⚠️ 注意が必要
- ファイルパス（自動対応済み）
- 権限設定（管理者権限推奨）

### ✅ 正常動作
- Electron アプリ開発
- Next.js 開発サーバー
- Prisma ORM
- TypeScript コンパイル
- テスト実行
- アプリ配布

## 💡 推奨環境

### 開発効率重視
1. **WSL2 + Ubuntu**: Linux環境をWindowsで
2. **VS Code**: WSL拡張機能使用

### シンプル重視
1. **Chocolatey + Make**: Windows標準環境
2. **npmコマンド**: Makefileなし

## 🆘 トラブルシューティング

### node-gyp エラー
```powershell
npm install -g windows-build-tools
npm config set msvs_version 2019
```

### Python エラー
```powershell
npm config set python python3
```

### 権限エラー
```powershell
# PowerShellを管理者として実行
Set-ExecutionPolicy RemoteSigned
```

## 🤝 サポート

Windows環境で問題が発生した場合：
1. [Issues](https://github.com/naro9507/dcs-accounting-app/issues)に投稿
2. OS、Node.jsバージョンを明記
3. エラーメッセージ全文を添付