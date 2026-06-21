# アーキテクチャ方針（今後の方向性）

このドキュメントは、現状のアーキテクチャから先に目指す **設計の方向性** を述べる。
現状の確定したアーキテクチャ（Model / AppData / AppState / Slices / Actions / Feature / Resources）は
[ARCHITECTURE.md](./ARCHITECTURE.md) と [STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md) を参照。

> 進捗管理（どの項目を着手済み/未着手か）はこのドキュメントではなく、プロジェクト管理側で行う。
> ここには「あるべき姿」と、それを選ぶ理由・検討事項だけを書く。

## 目指す姿

1. **UIフレームワークの分離** — View層を egui 以外へも載せ替えられるようにする
2. **データフローの一方向化** — View は読み取り(`Slices`)と操作結果(Action)の生成に徹し、状態更新は一箇所に集約する

## 課題と方針

### 1. View層の egui 依存

現状、View は `ui: &mut Ui` を直接受け取って描画しており、egui の型に結合している。

```rust
impl LocationView {
    pub fn render(slices: &Slices<'_>, ui: &mut Ui) -> LocationResult { ... }
}
```

**方針**: View が受け取る描画インターフェースを抽象化（Adapter 化）し、`ui: &mut Ui` をトレイト越しに扱えるようにする。
これにより egui 固有の型を View から切り離し、別バックエンドへの載せ替えを可能にする。

`fluffy` / `golem` のようなクロスプラットフォーム前提のプロジェクトではないが、UIフレームワークの選定変更コストを下げる狙い。

### 2. アクション処理の中央 dispatch 化

現状、各 Feature が自分のドメインの Action を生成し、`Actions::new(state)` に渡して処理している。

**方針**: 各 Feature の Action を横断的に集約する単一の入口（例: `AppAction` への統合、または `AmusApp` 側での一括 dispatch）を設け、
「View が Action を返す → 一箇所で状態更新」という流れを明確にする。これは View 層分離（課題1）とも整合する。

検討事項:
- `AppAction` enum に統合するか、ドメインごとの Action のまま `handle_*` を呼び分けるか
- egui の `Window::show` クロージャ内で `&mut self` が必要になる場面を、Action 収集 → 後処理パターンでどう回避するか

### 3. Feature の render シグネチャ規約化

Feature ごとに `render` の引数（`&mut AppState` / `&Response` / `&mut Ui` の組み合わせ）が揺れている。
View 層分離・中央 dispatch 化の形に合わせて、Feature/View の `render` シグネチャの規約を定める。

### 4. Actions 層の位置づけの見直し

`Slices`（読み取り）と `Actions`（書き込み）で読み書きを分離している。
View 層分離・中央 dispatch 化を進めた結果として、`Actions` 層が担う役割が薄くなる/重複する可能性がある。
その時点で、`Actions` を中央 dispatch に統合するか、独立した層として残すかを判断する。

## 関連ドキュメント

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 現在のアーキテクチャ
- [STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md) - 現在の状態管理パターン
