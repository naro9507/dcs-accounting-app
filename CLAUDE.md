# CLAUDE.md

## プロジェクト概要

個人事業主向けの確定申告用会計ソフト（Electronアプリ）

## 技術スタック

- **フレームワーク**: Electron + Next.js (TypeScript)
- **UI**: Shadcn UI + Tailwind CSS
- **状態管理**: Zustand
- **データベース**: SQLite
- **テスト**: Vitest

## プロジェクト構造

```
dcs-accounting-app/
├── electron/
│   └── main.js           # Electronメインプロセス
├── src/
│   ├── app/[locale]/     # Next.js App Router
│   │   └── income/       
│   │       ├── components/       # ページ固有コンポーネント
│   │       └── page.tsx          # ページコンポーネント
│   ├── features/         # 機能別ディレクトリ
│   │   └── income/       # 収入機能
│   │       ├── income.model.ts   # データ型定義・バリデーション
│   │       ├── income.service.ts # ビジネスロジック・DB操作
│   │       └── hooks/            # カスタムフック
│   ├── shared/           # 共通コンポーネント・ユーティリティ
│   │   └── components/   # 汎用Compound Patternコンポーネント
│   │       ├── form/     # フォーム関連
│   │       └── data-list/ # データリスト関連
│   ├── lib/              # 共通ライブラリ・DB接続・暗号化
│   ├── i18n/             # 国際化設定
│   └── middleware.ts     # Next.jsミドルウェア
├── dist/                 # Next.jsビルド出力
└── package.json
```

## 主要機能

- 収入・支出の記録
- 確定申告書の生成
- データのエクスポート（CSV/PDF）
- バックアップ・復元

## 開発ルール

- ESLintを使用
- コミットメッセージは日本語OK
- 個人情報は絶対にハードコードしない
- セキュリティを最優先

## ビルド・デプロイ

### Makefileコマンド
```bash
# セットアップ
make setup               # 初期セットアップ
make quick-start         # 新規開発者向けクイックスタート

# 開発
make dev                 # 開発サーバー起動
make prisma-studio       # データベース管理UI
make test                # テスト実行
make check               # コード品質チェック

# ビルド
make build               # Next.jsビルド
make dist                # 配布パッケージ作成
make release-ready       # リリース準備

# その他
make clean               # クリーンアップ
make info                # システム情報表示
make help                # ヘルプ表示
```

### npmコマンド（従来）
```bash
npm run dev              # 開発サーバー起動
npm run prisma:studio    # データベース管理UI
npm run build:prod       # 本番用ビルド
npm run dist             # 配布パッケージ作成
```

### 環境変数
- `.env.development`: 開発環境用設定
- `.env.production`: 本番環境用設定
- データベースパスは実行時に動的決定

## 注意事項

- 税法に関する正確性は税理士に確認
- 個人情報の取り扱いに注意
- オフラインで動作すること

## 導入済み機能

### 国際化 (i18n)
- next-intlを使用
- 日本語（ja）と英語（en）に対応
- メッセージファイル: `src/i18n/messages/`

### ログ出力
- pinoロガーを使用
- Electronメインプロセスとレンダラーで統合
- 開発環境では詳細ログ、本番環境では必要最小限

### 自動アップデート
- electron-updaterを使用
- GitHub Releasesからの自動更新
- 本番環境でのみ有効

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

## アーキテクチャ

### シンプルな機能別構成
- **機能別ディレクトリ**: `features/[feature-name]/`
- **3つのファイル構成**：
  - `model.ts`: データ型定義・バリデーション
  - `service.ts`: ビジネスロジック・DB操作
  - `hooks/`: React カスタムフック

### 分かりやすい役割分担
- **Model**: データの形や検証ルール
- **Service**: データの保存・取得・更新・削除
- **Hooks**: React での状態管理とサービス呼び出し
- **Components**: UI コンポーネント（ページ固有）

### Compound Pattern コンポーネント
- **Context APIベース**の状態管理
- **再利用可能**なコンポーネント設計
- **shared/components**で共通化

### 設計の利点
- **学習コストが低い**: 複雑な概念を排除
- **ファイル数が少ない**: 迷わない構成
- **役割が明確**: 何をどこに書くかが分かりやすい