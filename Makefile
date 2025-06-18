# DCS会計アプリ - Makefile
# プロジェクトのセットアップ・開発・ビルドを自動化

.PHONY: help setup install dev build dist clean test lint format typecheck prisma-setup prisma-reset prisma-studio hooks-install hooks-test hooks-skip

# デフォルトターゲット
.DEFAULT_GOAL := help

# ヘルプ表示
help: ## このヘルプを表示
	@echo "DCS会計アプリ - 利用可能なコマンド:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# 初期セットアップ
setup: ## プロジェクトの初期セットアップを実行
	@echo "👋 DCS会計アプリへようこそ！"
	@echo "🚀 プロジェクトのセットアップを開始..."
	make install
	make prisma-setup
	make hooks-install
	@echo ""
	@echo "✅ セットアップが完了しました！"
	@echo ""
	@echo "🎉 以下のコマンドが利用できます:"
	@echo "  make dev          - 開発サーバー起動"
	@echo "  make prisma-studio - データベース管理"
	@echo "  make test         - テスト実行"
	@echo "  make dist         - 配布パッケージ作成"
	@echo ""
	@echo "🪝 Git hooks が有効化されました:"
	@echo "  - プッシュ時: リント・型チェック・テスト自動実行"
	@echo ""
	@echo "詳細は 'make help' をご確認ください。"

# 依存関係インストール
install: ## npm依存関係をインストール
	@echo "📦 依存関係をインストール中..."
	npm install

# 開発サーバー起動
dev: ## 開発サーバーを起動
	@echo "🔧 開発サーバーを起動中..."
	npm run dev

# Next.jsビルド
build: ## Next.jsアプリをビルド
	@echo "🏗️  Next.jsアプリをビルド中..."
	npm run build

# Electronアプリビルド
build-electron: ## Electronアプリをビルド
	@echo "⚡ Electronアプリをビルド中..."
	npm run build:electron

# 本番用完全ビルド
build-prod: ## 本番用の完全ビルド
	@echo "🏭 本番用ビルドを実行中..."
	npm run build:prod

# 配布パッケージ作成
dist: ## 配布用パッケージを作成
	@echo "📦 配布パッケージを作成中..."
	npm run dist

# クリーンアップ
clean: ## ビルド成果物をクリーンアップ
	@echo "🧹 クリーンアップ中..."
	rm -rf dist/
	rm -rf release/
	rm -rf node_modules/.cache/
	rm -rf .next/
	@echo "✅ クリーンアップ完了"

# 完全クリーンアップ
clean-all: clean ## すべてをクリーンアップ（node_modules含む）
	@echo "🗑️  完全クリーンアップ中..."
	rm -rf node_modules/
	rm -rf dev.db
	rm -rf prisma/dev.db
	@echo "✅ 完全クリーンアップ完了"

# テスト実行
test: ## テストを実行
	@echo "🧪 テストを実行中..."
	npm run test

# テスト（ウォッチモード）
test-watch: ## テストをウォッチモードで実行
	@echo "👀 テスト（ウォッチモード）を実行中..."
	npm run test -- --watch

# リント実行
lint: ## コードリントを実行
	@echo "🔍 リントを実行中..."
	npm run lint

# リント修正
lint-fix: ## コードリントを実行して自動修正
	@echo "🔧 リント修正を実行中..."
	npm run lint:fix

# フォーマット
format: ## コードフォーマットを実行
	@echo "💅 コードフォーマットを実行中..."
	npm run format:fix

# 型チェック
typecheck: ## TypeScript型チェックを実行
	@echo "🔍 型チェックを実行中..."
	npm run typecheck

# コード品質チェック（全て）
check: ## コード品質チェック（リント・フォーマット・型チェック）を実行
	@echo "✅ コード品質チェックを実行中..."
	make lint
	make format
	make typecheck

# Prismaセットアップ
prisma-setup: ## Prismaの初期セットアップ
	@echo "🗄️  Prismaセットアップ中..."
	npm run prisma:generate
	npm run prisma:push
	@echo "✅ Prismaセットアップ完了"

# Prismaクライアント生成
prisma-generate: ## Prismaクライアントを生成
	@echo "⚙️  Prismaクライアントを生成中..."
	npm run prisma:generate

# Prismaスキーマ適用
prisma-push: ## Prismaスキーマをデータベースに適用
	@echo "📊 Prismaスキーマを適用中..."
	npm run prisma:push

# Prisma Studio起動
prisma-studio: ## Prisma Studioを起動
	@echo "🎛️  Prisma Studioを起動中..."
	npm run prisma:studio

# データベースリセット
prisma-reset: ## データベースをリセット
	@echo "🔄 データベースをリセット中..."
	rm -f dev.db
	rm -f prisma/dev.db
	make prisma-setup
	@echo "✅ データベースリセット完了"

# 開発環境準備
dev-ready: ## 開発環境の準備を完了
	@echo "🚀 開発環境を準備中..."
	make install
	make prisma-setup
	make check
	@echo "✅ 開発環境の準備が完了しました！"

# 本番リリース準備
release-ready: ## リリース準備（テスト・ビルド・パッケージ作成）
	@echo "🚀 リリース準備を開始..."
	make clean
	make install
	make check
	make test
	make dist
	@echo "✅ リリース準備が完了しました！"

# 開発者向けセットアップ（推奨）
start: setup ## 新しい開発者向けセットアップ（setupの別名）

# 開発者向けクイックスタート（後方互換性）
quick-start: setup ## クイックスタート（setupと同じ）

# システム情報表示
info: ## システム情報を表示
	@echo "📋 システム情報:"
	@echo "Node.js: $$(node --version)"
	@echo "npm: $$(npm --version)"
	@echo "OS: $$(uname -s)"
	@echo "PWD: $$(pwd)"
	@echo ""
	@echo "📦 プロジェクト情報:"
	@if [ -f package.json ]; then \
		echo "名前: $$(cat package.json | grep '"name"' | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')"; \
		echo "バージョン: $$(cat package.json | grep '"version"' | head -1 | sed 's/.*: *"\([^"]*\)".*/\1/')"; \
	fi
	@echo ""
	@echo "🗄️  データベース:"
	@if [ -f dev.db ]; then \
		echo "開発DB: ✅ 存在"; \
	else \
		echo "開発DB: ❌ 未作成"; \
	fi

# Git hooks関連
hooks-install: ## Git hooksをインストール
	@echo "🪝 Git hooksをインストール中..."
	npm run prepare
	@echo "✅ Git hooksインストール完了"

hooks-test: ## Git hooksをテスト実行
	@echo "🧪 Git hooksのテスト実行..."
	@echo "Pre-push hookテスト:"
	@echo "📋 Running lint, format, and type checks..."
	npm run lint
	npm run format  
	npm run typecheck
	@echo "🧪 Running tests..."
	npm run test
	@echo "✅ All hook tests passed!"

hooks-skip: ## 次回のコミット/プッシュでhooksをスキップ
	@echo "⚠️  次回のGit操作でhooksをスキップします"
	@echo "コミット時: git commit --no-verify"
	@echo "プッシュ時: git push --no-verify"