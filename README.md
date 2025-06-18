# DCS会計アプリ

個人事業主向けの確定申告用会計ソフト（Electronアプリ）

## 🚀 クイックスタート

### macOS / Linux
```bash
# プロジェクトのクローン
git clone https://github.com/naro9507/dcs-accounting-app.git
cd dcs-accounting-app

# 初期セットアップ（一発で完了）
make setup

# 開発サーバー起動
make dev
```

### Windows
```cmd
# プロジェクトのクローン
git clone https://github.com/naro9507/dcs-accounting-app.git
cd dcs-accounting-app

# Option 1: Makeを使用（推奨）
make setup
make dev

# Option 2: npmコマンド使用
npm install && npm run prisma:generate && npm run prisma:push
npm run dev
```

> **Windows注意**: Makeが必要です。詳細は [プラットフォーム対応](./docs/PLATFORM_SUPPORT.md) をご確認ください。

## 🛠️ 技術スタック

- **フレームワーク**: Electron + Next.js (TypeScript)
- **UI**: Shadcn UI + Tailwind CSS
- **データベース**: SQLite + Prisma ORM
- **状態管理**: Zustand
- **暗号化**: AES-256-GCM
- **国際化**: next-intl
- **ログ**: pino
- **テスト**: Vitest

## 📁 プロジェクト構造

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

## 🎯 主要機能

- 📊 収入・支出の記録と管理
- 📋 確定申告書の生成
- 📤 データのエクスポート（CSV/PDF）
- 🔄 バックアップ・復元
- 🌐 多言語対応（日本語・英語）
- 🔒 データ暗号化
- 🚀 自動アップデート

## 📚 ドキュメント

- [開発ガイド](./docs/DEVELOPMENT.md) - 詳細な開発手順
- [プラットフォーム対応](./docs/PLATFORM_SUPPORT.md) - Windows開発環境
- [プロジェクト指針](./CLAUDE.md) - 技術的な指針

## 📄 ライセンス

ISC

## 🙋‍♂️ サポート

問題や質問がある場合は、[Issues](https://github.com/naro9507/dcs-accounting-app/issues)にお気軽にご投稿ください。