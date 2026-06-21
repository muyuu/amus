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

### 現在の課題（検討中）

View層がegui（`ui: &mut Ui`）と直接結合している。詳細な方針は [ARCHITECTURE_ROADMAP.md](./docs/ARCHITECTURE_ROADMAP.md) を参照。

## ディレクトリ構造

```
src/
├── app.rs           # エントリーポイント
├── state/           # AppState, AppData
├── models/          # データ構造
├── features/        # 機能単位モジュール (Feature + View)
├── components/      # 再利用UIコンポーネント
├── assets/          # 画像リソース管理
├── i18n/            # 多言語対応
└── common/          # 共通ユーティリティ
```

## コード規約

- Viewから直接AppDataを変更しない（読み取りはSlices、書き込みはActions経由）
- Modelに状態変更ロジックを持たせない（AppStateに集約）
- `AppData`のフィールドは`pub(in crate::state)`。stateモジュール外からは直接触らない
