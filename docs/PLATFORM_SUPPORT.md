# プラットフォーム対応

## 🖥️ 対応OS

### ✅ 開発環境
- **macOS**: ✅ フル対応
- **Windows**: ❌ 非対応（WSL2必須）
- **Linux**: ✅ フル対応

### ✅ 配布アプリ
- **macOS**: DMG, ZIP (Intel + Apple Silicon)
- **Linux**: AppImage, DEB

## 🪟 Windows開発環境

**⚠️ 重要**: Windows ネイティブ環境での開発は非対応です。**WSL2を使用**してください。

### WSL2セットアップ手順

#### 1. WSL2のインストール
```powershell
# PowerShellを管理者として実行
wsl --install
```

#### 2. Ubuntu のインストール
```powershell
# Microsoft Storeから「Ubuntu」をインストール
# または
wsl --install -d Ubuntu
```

#### 3. プロジェクトセットアップ
```bash
# WSL2 内で実行
git clone https://github.com/naro9507/dcs-accounting-app.git
cd dcs-accounting-app
make setup
```

## 🤝 サポート

WSL2環境で問題が発生した場合：
1. [Issues](https://github.com/naro9507/dcs-accounting-app/issues)に投稿
2. WSL2バージョン、Node.jsバージョンを明記  
3. エラーメッセージ全文を添付