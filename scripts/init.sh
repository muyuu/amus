#!/bin/bash

echo "🚀 開発環境の初期設定を開始します..."

# OS判定
detect_os() {
    case "$OSTYPE" in
        msys*|cygwin*|win32*) echo "windows" ;;
        darwin*) echo "macos" ;;
        linux*) echo "linux" ;;
        *) echo "unknown" ;;
    esac
}

OS=$(detect_os)
echo "🖥️  検出されたOS: $OS"

# cargo-watchをインストール（watchexecの代替）
echo "📦 cargo-watchをチェック中..."
if command -v cargo-watch >/dev/null 2>&1; then
    echo "✅ cargo-watchは既にインストールされています"
else
    echo "📥 cargo-watchをインストール中..."
    if cargo install cargo-watch; then
        echo "✅ cargo-watchのインストールが完了しました"
    else
        echo "⚠️  警告: cargo-watchのインストールに失敗しました"
    fi
fi

# Rustターゲットを追加
echo "🎯 Rustターゲットを追加中..."
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add wasm32-unknown-unknown
echo "✅ Rustターゲットの追加が完了しました"

# wasm-packをインストール
echo "📦 wasm-packをチェック中..."
if command -v wasm-pack >/dev/null 2>&1; then
    echo "✅ wasm-packは既にインストールされています"
else
    echo "📥 wasm-packをインストール中..."
    if cargo install wasm-pack; then
        echo "✅ wasm-packのインストールが完了しました"
    else
        echo "⚠️  警告: wasm-packのインストールに失敗しました"
    fi
fi

# miniserveをインストール
echo "📦 miniserveをチェック中..."
if command -v miniserve >/dev/null 2>&1; then
    echo "✅ miniserveは既にインストールされています"
else
    echo "📥 miniserveをインストール中..."
    if cargo install miniserve; then
        echo "✅ miniserveのインストールが完了しました"
    else
        echo "⚠️  警告: miniserveのインストールに失敗しました"
    fi
fi

# OS別のクロスコンパイル環境セットアップ
case "$OS" in
    "windows")
        echo "🪟 Windows環境を検出しました"
        echo "✅ Windows環境ではmingw-w64は不要です（MSVCを使用）"
        ;;
    "macos")
        echo "🍎 macOS環境を検出しました"
        echo "🔧 Linux用クロスコンパイルツール（不要なのでスキップ）..."
        echo "✅ macOSではクロスコンパイルツールのセットアップは不要です"
        ;;
    "linux")
        echo "🐧 Linux環境を検出しました"
        echo "🔧 Windows用クロスコンパイルツールをチェック中..."
        if command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
            echo "✅ mingw-w64は既にインストールされています"
        else
            echo "📥 mingw-w64をインストール中..."
            if [[ -f /etc/debian_version ]]; then
                sudo apt update && sudo apt install -y gcc-mingw-w64-x86-64
            elif [[ -f /etc/fedora-release ]]; then
                sudo dnf install -y mingw64-gcc
            elif [[ -f /etc/arch-release ]]; then
                sudo pacman -S --noconfirm mingw-w64-gcc
            else
                echo "⚠️  警告: サポートされていないLinuxディストリビューションです"
                echo "   手動でmingw-w64をインストールしてください"
            fi
        fi
        ;;
    *)
        echo "⚠️  警告: サポートされていないOSです"
        ;;
esac

echo ""
echo "🎉 セットアップ完了！"
echo ""
echo "次のコマンドで開発を開始できます:"
echo "  mise run dev     # 開発環境起動（ファイル監視付き）"
echo "  mise run build   # ビルド"
echo "  mise run test    # テスト実行"
echo ""
