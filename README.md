<div align="center">
  <img src="assets/images/logo.png" alt="Amus" width="320" />
</div>

# Amus — Among Us Map Utility Software

Among Us のゲーム進行（プレイヤーの位置・移動・生死・サボタージュ等）を記録・可視化する
デスクトップ / WebAssembly アプリケーション。Rust + [egui](https://github.com/emilk/egui) 製。

## 主な機能

- **マップ記録**: プレイヤーの出現位置・移動軌跡をマップ上に配置・描画
- **プレイヤー管理**: 生死・色・名前、ボタン使用状況
- **ターン（wave）管理**: ターンごとに記録を切り替え
- **サボタージュ**: 通信 / 停電 / 酸素 / 原子炉の解決状況を管理
- **音声メモ**（ネイティブのみ）: マイク録音と [whisper.cpp](https://github.com/ggerganov/whisper.cpp) による書き起こし
- **多言語**（日本語 / 英語）、テーマ、UI 拡大率の設定
- **ネイティブ / WebAssembly** の両対応

## 動作環境・前提

- [mise](https://mise.jdx.dev/) … タスクランナー兼ツールチェイン管理（Rust ツールチェイン等は `mise run init` で導入）
- **Linux でネイティブビルドする場合**の追加パッケージ:
  - `libasound2-dev`（cpal / ALSA）、`libudev-dev`・`libxkbcommon-dev`（winit / wgpu）
  - `cmake` / `clang` / `libclang-dev`（whisper.cpp のビルド）
  - whisper-rs のビルドは whisper.cpp を C/C++ コンパイルするため初回は数分かかる
- macOS / Windows は追加パッケージ不要。WASM ビルドに音声機能は含まれない（`voice_memo` はネイティブ専用）

## セットアップ

```bash
mise run init   # 依存インストール + git フック（push 前ゲート）設定
```

## ビルド・実行

```bash
# ネイティブ
cargo run         # または mise run start
mise run dev      # ファイル監視 + 自動リロード

# WebAssembly（wasm-pack で web/pkg にビルドし、miniserve で配信）
mise run build-wasm
mise run serve    # http://localhost:8080
```

音声メモは初回に Whisper モデル（約 466MB）をアプリのデータディレクトリへダウンロードする
（SHA-256 検証つき）。

## プロジェクト構成

```
src/
├── main.rs / lib.rs   # native / WASM の各エントリーポイント
├── app.rs             # eframe::App（update → render → handle_actions）
├── shell.rs           # native/wasm 共通のシェル初期化
├── state/             # AppState / AppData / Slices（読み取り）/ Actions（書き込み）
├── models/            # データ構造
├── features/          # 機能単位モジュール（Feature + View）
├── components/        # 再利用 UI コンポーネント
├── resources/         # ハードウェア依存リソース（画像・録音・書き起こし）
├── i18n/ · log/ · common/ · constants.rs
```

設計の詳細は [docs/](./docs/) を参照:

- [ARCHITECTURE.md](./docs/ARCHITECTURE.md) — レイヤー構成・データフロー
- [STATE_MANAGEMENT.md](./docs/STATE_MANAGEMENT.md) — 状態管理パターン
- [decisions.md](./docs/decisions.md) — 設計判断記録（ADR）
- [ARCHITECTURE_ROADMAP.md](./docs/ARCHITECTURE_ROADMAP.md) — 今後の方向性

## 開発

`mise run init` が push 前フック（`mise run pre-push`）を有効化する。push 前に
**フォーマット検査 + native/wasm の clippy + テスト**が走る。緊急時は `git push --no-verify` でスキップ可。

PR では CI が `fast`（fmt + wasm clippy）と `test`（voice_memo を除いた軽量テスト）を実行する。
