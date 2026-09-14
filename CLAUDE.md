# Amus - Among Us Map Utility Software

## プロジェクト概要

Among Usのゲーム中の情報を記録・可視化するデスクトップアプリケーション。
Rust + egui + WebAssembly で構築。

エンドユーザー向けの説明は [README.md](./README.md)、開発の背景・詳細な制約は
[docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) を参照。

## リポジトリ運用

**`main` への直接 push は禁止。変更は必ずブランチを切って PR で行う。**
GitHub 側でも `main` への直接 push を禁止するブランチ保護を設定済み。

## ビルド・実行（クイックリファレンス）

```bash
# 初期化（依存インストール + git フック設定）。clone / worktree 作成時に一度
mise run init

# ネイティブ実行
cargo run            # または mise run start
mise run dev         # ファイル監視 + 自動リロード

# GPU バックエンド付き（書き起こしが速くなる。認識精度は CPU 版と同じ）
mise run build-gpu
mise run start-gpu

# WASM（wasm-pack で web/pkg にビルドし、miniserve で配信）
mise run build-wasm
mise run serve       # http://localhost:8080

# 配布用ビルド（dist/ へ）
mise run release-gpu
mise run release-cpu
```

`voice_memo`（音声録音・Whisper 書き起こし）はネイティブ専用で cpal / whisper-rs を使う。
Linux の追加パッケージ、GPU ビルドの Vulkan SDK / target ディレクトリ分離の理由など、
ビルドの詳細と背景は [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) を参照。

### push 前ゲート（git pre-push フック）

`mise run init`（worktree/clone 初期化）が `core.hooksPath` を `.githooks` に向け、push 前に
`mise run pre-push`（fmt 検査 + native/wasm clippy + test）を実行するようにする。CI は PR で
fmt と wasm clippy しか回さない（native clippy・test は main への push 時のみ）ため、テストと
native clippy の回帰防止はこのフックで担保する。緊急時は `git push --no-verify` でスキップ可。

## アーキテクチャ

詳細は [docs/](./docs/) を参照。

### 設計ドキュメント

- **[DEVELOPMENT.md](./docs/DEVELOPMENT.md)** - セットアップ・ビルド・配布
- **[ARCHITECTURE.md](./docs/ARCHITECTURE.md)** - レイヤー構成、データフロー、Feature設計
- **[STATE_MANAGEMENT.md](./docs/STATE_MANAGEMENT.md)** - AppState/AppData、Slices/Actions、ビジネスロジック配置
- **[VOICE_MEMO.md](./docs/VOICE_MEMO.md)** - 音声メモ（録音・VAD・書き起こし）の設計指針

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
├── resources/       # ハードウェア依存リソース（画像・録音・書き起こし。録音/書き起こし本体は crates/transcribe）
├── i18n/            # 多言語対応
├── log/             # ロギング（native=env_logger / wasm=console）
└── common/          # 共通ユーティリティ

crates/
└── transcribe/      # 音声認識（GUIに依存しない別クレート。transcribe-check CLI も持つ）
```

## コード規約

- Viewから直接AppDataを変更しない（読み取りはSlices、書き込みはActions経由）
- Modelに状態変更ロジックを持たせない（AppStateに集約）
- `AppData`のフィールドは`pub(in crate::state)`。stateモジュール外からは直接触らない
