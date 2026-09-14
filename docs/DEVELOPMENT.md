# 開発ガイド

セットアップ・ビルド・配布まわりの手順と、その背景にある制約をまとめる。
アーキテクチャ（コードの設計方針）は [ARCHITECTURE.md](./ARCHITECTURE.md) を参照。

## セットアップ

[mise](https://mise.jdx.dev/) でツールチェイン・依存関係を管理する。

```bash
mise run init   # 依存インストール + git フック（push 前ゲート）設定
```

### ネイティブビルドの前提

`voice_memo`（音声録音・Whisper 書き起こし）が cpal / whisper-rs を使うため、ネイティブビルドには
追加の前提がある。

- **Linux**: 以下のパッケージが必要。
  - `libasound2-dev`（cpal / ALSA）
  - `libudev-dev`・`libxkbcommon-dev`（winit / wgpu）
  - `cmake` / `clang` / `libclang-dev`（whisper.cpp のビルド）
- **whisper-rs のビルドは重い**（C/C++ の whisper.cpp をコンパイルするため初回は数分かかる）。CI では
  キャッシュミス時にここがスパイクする。
- macOS / Windows は追加パッケージ不要。WASM ビルドに音声機能は含まれない（`voice_memo` はネイティブ専用）。

## ビルド・実行

```bash
# ネイティブ
cargo run            # または mise run start
mise run dev         # ファイル監視 + 自動リロード

# WASM（wasm-pack で web/pkg にビルドし、miniserve で配信）
mise run build-wasm
mise run serve       # http://localhost:8080
```

音声メモは初回に Whisper モデル（約 466MB）をアプリのデータディレクトリへダウンロードする
（SHA-256 検証つき）。

## 書き起こしの GPU ビルド

whisper.cpp のバックエンドはコンパイル時に固定されるため、GPU を使うビルドは feature で
指定する。既定は CPU。

```bash
mise run build-gpu   # GPU ビルド
mise run start-gpu   # GPU 版を起動
```

バックエンドは OS で決まる（Windows / Linux は Vulkan、macOS は Metal）。NVIDIA 専用の
`gpu-cuda` feature もあるが、実行側に CUDA ランタイムを要求するため配布には使わない。

GPU / CPU の違いは処理速度だけで認識精度には影響しないため、モデルは GPU / CPU で
揃えている（詳細は [VOICE_MEMO.md](./VOICE_MEMO.md) の「実行バックエンドとモデル」）。

`gpu-vulkan` のビルドには **Vulkan SDK**（`VULKAN_SDK` 環境変数）が要る。要るのはビルドする
マシンだけで、実行側には GPU ドライバ同梱のローダー（`vulkan-1.dll` / `libvulkan.so.1`）しか
要らない。インストール直後はシェルを開き直さないと環境変数が反映されない。

### GPU ビルドの target ディレクトリ

GPU ビルドは CPU ビルドと target ディレクトリを分ける。feature が違うとビルドグラフ全体が
無効化されるため、共有すると切り替えるたびに全再ビルドになる。Windows ではこれが必須でもある。
ggml-vulkan がシェーダ生成ツールを入れ子の ExternalProject として建てる都合で中間ファイルの
パスが深くなり、リポジトリ内の `target/` では MAX_PATH(260) を超えて `cl.exe` が pdb を
開けなくなる（OS の LongPathsEnabled は MSVC のツール群に効かない）。既定の移動先は
`C:\amus-build` で、`AMUS_GPU_TARGET_DIR` で変えられる。

GPU feature を付けると whisper.cpp のビルドがさらに重くなるため、CI と pre-push フックは
CPU ビルドのまま回す。

## 配布用ビルド

```bash
mise run release-gpu   # → dist/memongus-<os>-gpu
mise run release-cpu   # → dist/memongus-<os>-cpu
```

動かすマシンの OS 向けに作る。whisper.cpp と Vulkan のシェーダを含むためクロスコンパイル
はせず、配布する OS ごとにそのマシンで実行する。リリースビルドはアセットを実行ファイルへ
埋め込むため、成果物は 1 ファイルで動く（デバッグビルドは `assets/` を実行時に読むので
配布には使えない）。

### モデル比較検証用ビルド（配布しない）

```bash
mise run release-gpu-record-audio
```

`transcribe` クレートの `record-audio` feature を足したビルド。書き起こしに渡した音声
チャンクを実行ファイルの隣の `./recorded/*.mp3` に保存する。Whisper のモデル選定は手元の
少数サンプルだけでは決め切れないため、実機で使ってもらいながら実データを集める用途。
成果物名に `-record-audio` を付けて通常の配布ビルドと区別しており、`default` feature には
含めていない。

## push 前ゲート（git pre-push フック）

`mise run init`（worktree/clone 初期化）が `core.hooksPath` を `.githooks` に向け、push 前に
`mise run pre-push`（fmt 検査 + native/wasm clippy + test）を実行するようにする。CI は PR で
fmt と wasm clippy しか回さない（native clippy・test は main への push 時のみ）ため、テストと
native clippy の回帰防止はこのフックで担保する。緊急時は `git push --no-verify` でスキップ可。

## リポジトリ運用

`main` への直接 push は禁止。変更はすべてブランチ + PR で行う。
