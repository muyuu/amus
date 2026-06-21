# アーキテクチャ方針（今後の方向性）

このドキュメントは、現状のアーキテクチャから先に目指す **設計の方向性** を述べる。
現状の確定したアーキテクチャ（Model / AppData / AppState / Slices / Actions / Feature / Resources）は
[ARCHITECTURE.md](./ARCHITECTURE.md) と [STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md) を参照。

> 進捗管理（どの項目を着手済み/未着手か）はこのドキュメントではなく、プロジェクト管理側で行う。
> ここには「あるべき姿」と、それを選ぶ理由・検討事項だけを書く。

> 中央 dispatch（`update → render → handle_actions` の 3 段、`AppAction` への統合）は導入済み。
> 経緯は [decisions.md](./decisions.md) 0001、現状は [ARCHITECTURE.md](./ARCHITECTURE.md) を参照。

## 目指す姿

1. **UIフレームワークの分離** — View層を egui 以外へも載せ替えられるようにする

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

### 2. Actions 層の位置づけの見直し

`Slices`（読み取り）と `Actions`（書き込み）で読み書きを分離している。
中央 dispatch を導入した結果、ドメイン handler の薄いラッパが目立つ場合がある。
`Actions` を独立した層として残すか、`handle_actions` に畳むかを改めて判断する。

## 関連ドキュメント

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 現在のアーキテクチャ
- [STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md) - 現在の状態管理パターン
