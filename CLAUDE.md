# CLAUDE.md

## プロジェクト概要

個人事業主向けの確定申告用会計ソフト（Electronアプリ）

## 技術スタック

- **Node.js**: v24.2.0+
- **パッケージマネージャー**: pnpm v10.0.0+
- **フレームワーク**: Electron + Next.js (TypeScript)
- **UI**: Shadcn UI + Tailwind CSS
- **状態管理**: Zustand
- **データベース**: SQLite
- **テスト**: Vitest

## 開発ガイド

開発に関する詳細な情報は [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) を参照してください。

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



## 注意事項

- 税法に関する正確性は税理士に確認
- 個人情報の取り扱いに注意
- オフラインで動作すること

