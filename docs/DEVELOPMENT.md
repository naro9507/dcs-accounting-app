# 開発ガイド

## 📋 利用可能なコマンド

### 基本コマンド

```bash
make help          # ヘルプ表示
make setup         # 初期セットアップ
make dev           # 開発サーバー起動
make build         # ビルド
make dist          # 配布パッケージ作成
make clean         # クリーンアップ
```

### 開発コマンド

```bash
make test          # テスト実行
make lint          # リント実行
make format        # コードフォーマット
make typecheck     # 型チェック
make check         # コード品質チェック（全て）
```

### データベース

```bash
make prisma-studio # データベース管理UI
make prisma-setup  # Prismaセットアップ
make prisma-reset  # データベースリセット
```

### Git Hooks

```bash
make hooks-install # Git hooksをインストール
make hooks-test    # Git hooksをテスト実行
make hooks-skip    # 次回のGit操作でhooksをスキップ
```

### その他

```bash
make info          # システム情報表示
make release-ready # リリース準備
make clean-all     # 完全クリーンアップ
```

## 🔧 開発セットアップ

### 1. 依存関係

- **Node.js**: v24.2.0以上
- **pnpm**: v10.0.0以上
- **Make** (Windows: [Chocolatey](https://chocolatey.org/)推奨)

### 2. OS別セットアップ

#### macOS / Linux

```bash
make setup
make dev
```

#### Windows

**重要**: WSL2必須（ネイティブWindows非対応）

```cmd
# Makeがある場合
make setup
make dev

# Makeがない場合
pnpm install
pnpm run prisma:generate  
pnpm run prisma:push
pnpm run dev
```

### Node.jsバージョン管理

```bash
# miseを使用してNode.jsバージョンを固定（推奨）
mise install     # .node-versionファイルに基づいてバージョンをインストール
mise use         # バージョンを切り替え

# nvmでも利用可能
nvm use          # .node-versionファイルに基づいてバージョンを切り替え
nvm install      # 必要なバージョンをインストール
```

詳細: [PLATFORM_SUPPORT.md](./PLATFORM_SUPPORT.md)

## 📦 ビルド・配布

### 開発ビルド

```bash
make build
```

### 配布パッケージ作成

```bash
make dist
```

作成されるパッケージ:

- **macOS**: DMG, ZIP
- **Windows**: NSIS installer, Portable
- **Linux**: AppImage, DEB

## 🗄️ データベース

### 開発環境

- ファイル: `./dev.db`
- 管理: `make prisma-studio`

### 本番環境

- 場所: ユーザーデータディレクトリ
- 暗号化: AES-256-GCM

## 🚢 デプロイ

### 自動更新

- GitHub Releasesを使用
- electron-updaterで自動更新
- 本番環境でのみ有効

### 環境変数

- `.env.development`: 開発環境用設定
- `.env.production`: 本番環境用設定
- データベースパスは実行時に動的決定

## 🤝 開発ガイド

### コード品質

```bash
make check    # リント・フォーマット・型チェック
```

### テスト

```bash
make test            # 単発実行
make test-watch      # ウォッチモード
```

### データベース操作

```bash
make prisma-studio   # 管理UI起動
make prisma-reset    # データリセット
```

## 🏗️ プロジェクト構造

```
dcs-accounting-app/
├── Makefile              # プロジェクト操作自動化
├── docs/                 # ドキュメント
├── src/
│   ├── app/[locale]/     # Next.js App Router
│   ├── features/         # 機能別ディレクトリ
│   ├── shared/           # 共通コンポーネント
│   └── lib/              # ユーティリティ
├── electron/             # Electronメインプロセス
├── prisma/               # データベーススキーマ
└── build/                # ビルド設定
```

## 🛠️ 技術スタック

- **フレームワーク**: Electron + Next.js (TypeScript)
- **UI**: Shadcn UI + Tailwind CSS
- **データベース**: SQLite
- **状態管理**: Zustand
- **暗号化**: AES-256-GCM
- **国際化**: next-intl
- **ログ**: pino
- **テスト**: Vitest

## 📝 アーキテクチャ

### シンプルな機能別構成

#### Feature-Based Architecture

```
src/features/[feature]/
├── [feature].model.ts    # データ型定義・バリデーション
├── [feature].service.ts  # ビジネスロジック・DB操作
└── hooks/
    └── use-[feature].ts  # React カスタムフック
```

#### 役割

- **Model**: データの形や検証ルール
- **Service**: データの保存・取得・更新・削除
- **Hooks**: React での状態管理とサービス呼び出し
- **Components**: UI コンポーネント（ページ固有）

### Compound Pattern コンポーネント

```
src/shared/components/
├── form/                 # フォーム関連（Compound Pattern）
├── data-list/            # データ表示（Compound Pattern）
└── index.ts              # エクスポート
```

- **構造化**された構成のため視覚的に意図がわかりやすくなるため採用
- **再利用可能**なコンポーネント設計
- **shared/components**で共通化

### 設計の利点

- **学習コストが低い**: 複雑な概念を排除
- **ファイル数が少ない**: 迷わない構成
- **役割が明確**: 何をどこに書くかが分かりやすい

## 🪝 Git Hooks

### 自動品質チェック

**Pre-push Hook（プッシュ時）:**
- 全ファイルのリントチェック
- TypeScript型チェック
- テスト実行

### Hooks管理

```bash
# Git hooksのテスト実行
make hooks-test

# 緊急時：hooks無効化でプッシュ
git push --no-verify
```

## 💻 開発フロー

### 1. 機能追加

```bash
# 新機能ブランチ作成
git checkout -b feature/new-feature

# 開発開始
make dev

# コード品質チェック（任意）
make check

# テスト実行（任意）
make test
```

### 2. コミット・プッシュ

```bash
# ファイルをステージング
git add .

# コミット
git commit -m "feat: 新機能追加"

# プッシュ（pre-pushフックが自動実行）
git push origin feature/new-feature
```

> **Note**: プッシュ時にGit hooksにより、品質チェックとテストが自動実行されます

### 3. リリース準備

```bash
# リリース準備（テスト・ビルド・パッケージ）
make release-ready
```

## 🔍 デバッグ

### 開発ツール

- **VS Code, Cursor**: 推奨エディタ
- **Electron DevTools**: 自動で開く（開発モード）
- **Prisma Studio**: `make prisma-studio`

### ログ確認

- **開発**: コンソールに pretty print
- **本番**: 構造化JSON形式

## 🚨 トラブルシューティング

### よくある問題

1. **Prismaエラー**: `make prisma-reset`
2. **依存関係エラー**: `make clean-all && make setup`
3. **ビルドエラー**: `make clean && make build`

### Windowsでの問題

詳細は [PLATFORM_SUPPORT.md](./PLATFORM_SUPPORT.md) を参照

## 📄 関連ドキュメント

- [PLATFORM_SUPPORT.md](./PLATFORM_SUPPORT.md) - Windows開発対応
- [CLAUDE.md](../CLAUDE.md) - プロジェクト概要・指針
- [README.md](../README.md) - 基本情報

## 🔐 セキュリティ

### データ暗号化

- **ファイル暗号化**: Node.js crypto モジュール（AES-256-GCM）
- **データベース**: better-sqlite3 + アプリケーションレベル暗号化
- **キー管理**: 自動生成・安全保存
- **暗号化対象**: 機密データ（description, notes など）
- **機能**:
  - 暗号化データの挿入・取得（`insertEncryptedData`, `getEncryptedData`）
  - ファイル暗号化・復号化
  - パスワードハッシュ化・検証
  - セキュア一時ファイル作成

### セキュリティベストプラクティス

- 個人情報は絶対にハードコードしない
- 秘密鍵やAPIキーの露出・ログ出力を避ける
- リポジトリへの秘密鍵やキーのコミットを防ぐ