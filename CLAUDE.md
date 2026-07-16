# Amus - Among Us Map Utility Software

## プロジェクト概要

Among Usのゲーム中の情報を記録・可視化するデスクトップアプリケーション。
Rust + egui + WebAssembly で構築。

## ビルド・実行

```bash
# ネイティブ実行
cargo run

# WASM ビルド
trunk serve  # 開発サーバー
trunk build --release  # 本番ビルド
```

### ネイティブビルドの前提

`voice_memo`（音声録音・Whisper 書き起こし）が cpal / whisper-rs を使うため、ネイティブビルドには
追加の前提がある。

- **Linux**: cpal がオーディオに ALSA を使うため `libasound2-dev` が必要（例: `apt install libasound2-dev`）。
- **whisper-rs のビルドは重い**（C/C++ の whisper.cpp をコンパイルするため初回は数分かかる）。CI では
  キャッシュミス時にここがスパイクする。
- macOS / Windows は追加パッケージ不要。WASM ビルドに音声機能は含まれない（`voice_memo` はネイティブ専用）。

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
