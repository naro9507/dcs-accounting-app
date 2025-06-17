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
│   ├── app/              # Next.js App Router
│   ├── components/       # Reactコンポーネント
│   ├── lib/              # ユーティリティ・DB接続
│   └── store/            # Zustand状態管理
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

## 注意事項

- 税法に関する正確性は税理士に確認
- 個人情報の取り扱いに注意
- オフラインで動作すること