# Amus - Among Us Map Utility Software

## プロジェクト概要

Among Usのゲーム中の情報を記録・可視化するデスクトップアプリケーション。
Rust + egui + WebAssembly で構築。

## ビルド・実行

```bash
# 初期化（依存インストール + git フック設定）。clone / worktree 作成時に一度
mise run init

# ネイティブ実行
cargo run            # または mise run start
mise run dev         # ファイル監視 + 自動リロード

# WASM（wasm-pack で web/pkg にビルドし、miniserve で配信）
mise run build-wasm
mise run serve       # http://localhost:8080
```

### ネイティブビルドの前提

`voice_memo`（音声録音・Whisper 書き起こし）が cpal / whisper-rs を使うため、ネイティブビルドには
追加の前提がある。

- **Linux**: cpal がオーディオに ALSA を使うため `libasound2-dev` が必要（例: `apt install libasound2-dev`）。
- **whisper-rs のビルドは重い**（C/C++ の whisper.cpp をコンパイルするため初回は数分かかる）。CI では
  キャッシュミス時にここがスパイクする。
- macOS / Windows は追加パッケージ不要。WASM ビルドに音声機能は含まれない（`voice_memo` はネイティブ専用）。

### 書き起こしの GPU ビルド

whisper.cpp のバックエンドはコンパイル時に固定されるため、GPU を使うビルドは feature で
指定する。既定は CPU。

```bash
mise run build-gpu   # GPU ビルド
mise run start-gpu   # GPU 版を起動
```

バックエンドは OS で決まる（Windows / Linux は Vulkan、macOS は Metal）。NVIDIA 専用の
`gpu-cuda` feature もあるが、実行側に CUDA ランタイムを要求するため配布には使わない。

`gpu-vulkan` のビルドには **Vulkan SDK**（`VULKAN_SDK` 環境変数）が要る。要るのはビルドする
マシンだけで、実行側には GPU ドライバ同梱のローダー（`vulkan-1.dll` / `libvulkan.so.1`）しか
要らない。インストール直後はシェルを開き直さないと環境変数が反映されない。

### 配布用ビルド

```bash
mise run release-gpu   # → dist/memongus-<os>-gpu
mise run release-cpu   # → dist/memongus-<os>-cpu
```

動かすマシンの OS 向けに作る。whisper.cpp と Vulkan のシェーダを含むためクロスコンパイル
はせず、配布する OS ごとにそのマシンで実行する。リリースビルドはアセットを実行ファイルへ
埋め込むため、成果物は 1 ファイルで動く（デバッグビルドは `assets/` を実行時に読むので
配布には使えない）。

### GPU ビルドの target ディレクトリ

GPU ビルドは CPU ビルドと target ディレクトリを分ける。feature が違うとビルドグラフ全体が
無効化されるため、共有すると切り替えるたびに全再ビルドになる。Windows ではこれが必須でもある。
ggml-vulkan がシェーダ生成ツールを入れ子の ExternalProject として建てる都合で中間ファイルの
パスが深くなり、リポジトリ内の `target/` では MAX_PATH(260) を超えて `cl.exe` が pdb を
開けなくなる（OS の LongPathsEnabled は MSVC のツール群に効かない）。既定の移動先は
`C:\amus-build` で、`AMUS_GPU_TARGET_DIR` で変えられる。

GPU feature を付けると whisper.cpp のビルドがさらに重くなるため、CI と pre-push フックは
CPU ビルドのまま回す。

### push 前ゲート（git pre-push フック）

`mise run init`（worktree/clone 初期化）が `core.hooksPath` を `.githooks` に向け、push 前に
`mise run pre-push`（fmt 検査 + native/wasm clippy + test）を実行するようにする。CI は PR で
fmt と wasm clippy しか回さない（native clippy・test は main への push 時のみ）ため、テストと
native clippy の回帰防止はこのフックで担保する。緊急時は `git push --no-verify` でスキップ可。

## アーキテクチャ

詳細は [docs/](./docs/) を参照。

### 設計ドキュメント

- **[ARCHITECTURE.md](./docs/ARCHITECTURE.md)** - レイヤー構成、データフロー、Feature設計
- **[STATE_MANAGEMENT.md](./docs/STATE_MANAGEMENT.md)** - AppState/AppData、Slices/Actions、ビジネスロジック配置

### 基本方針

```
Model (データ構造)
  ↓
AppData (データコンテナ)
  ↓
AppState (アクセス制御 + ビジネスロジック)
  ├─ Slices  (読み取り)
  └─ Actions (書き込み)
  ↓
Feature → View
```

- **Model**: 純粋なデータ構造。ロジックを持たない
- **AppState**: 全てのビジネスロジックを集約。`AppData`を値で所有する（内部可変性は使わない）
- **Slices**: 読み取り専用アクセス。Viewはこれでデータを読む
- **View**: 描画のみ。状態を直接変更せず操作結果（Action）を返す
- **Actions**: Viewが返した操作結果を受けてAppStateを更新

### アーキテクチャ方針

現時点で未解決のアーキテクチャ方針はない（過去の検討は decisions.md の ADR で決着済み）。
今後の方向性は [ARCHITECTURE_ROADMAP.md](./docs/ARCHITECTURE_ROADMAP.md) を参照。

## ディレクトリ構造

```
src/
├── main.rs          # ネイティブのエントリーポイント
├── lib.rs           # WASM のエントリーポイント
├── app.rs           # eframe::App 実装（update → render → handle_actions）
├── app_action.rs    # 各ドメイン Action を束ねる AppAction
├── constants.rs     # 定数
├── state/           # AppState, AppData, Slices, Actions
├── models/          # データ構造
├── features/        # 機能単位モジュール (Feature + View)
├── components/      # 再利用UIコンポーネント
├── resources/       # ハードウェア依存リソース（画像・録音・書き起こし）
├── i18n/            # 多言語対応
├── log/             # ロギング（native=env_logger / wasm=console）
└── common/          # 共通ユーティリティ
```

## コード規約

- Viewから直接AppDataを変更しない（読み取りはSlices、書き込みはActions経由）
- Modelに状態変更ロジックを持たせない（AppStateに集約）
- `AppData`のフィールドは`pub(in crate::state)`。stateモジュール外からは直接触らない
