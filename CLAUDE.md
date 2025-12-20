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
- **[STATE_MANAGEMENT.md](./docs/STATE_MANAGEMENT.md)** - AppState/AppData、RefCell、ビジネスロジック配置

### 基本方針

```
Model (データ構造)
  ↓
AppData (データコンテナ)
  ↓
AppState (アクセス制御 + ビジネスロジック)
  ↓
Feature → View → Interaction
```

- **Model**: 純粋なデータ構造。ロジックを持たない
- **AppState**: 全てのビジネスロジックを集約。Modelを直接触らせない
- **View**: 描画のみ。操作結果をResultで返す
- **Interaction**: Viewの結果を受けてAppStateを更新

### 現在の課題（検討中）

View層がegui（`ui: &mut Ui`）と直接結合している。
フレームワーク変更を見据える場合、Actions層の明確化を検討。

## ディレクトリ構造

```
src/
├── app.rs           # エントリーポイント
├── state/           # AppState, AppData
├── models/          # データ構造
├── features/        # 機能単位モジュール (View + Interaction)
├── components/      # 再利用UIコンポーネント
├── assets/          # 画像リソース管理
├── i18n/            # 多言語対応
└── common/          # 共通ユーティリティ
```

## コード規約

- Viewから直接AppDataを変更しない（Interaction経由）
- Modelに状態変更ロジックを持たせない（AppStateに集約）
- `data_mut()`は最終手段。専用メソッドを優先
