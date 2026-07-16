# アーキテクチャ方針（今後の方向性）

このドキュメントは、現状のアーキテクチャから先に目指す **設計の方向性** を述べる。
現状の確定したアーキテクチャ（Model / AppData / AppState / Slices / Actions / Feature / Resources）は
[ARCHITECTURE.md](./ARCHITECTURE.md) と [STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md) を参照。

> 進捗管理（どの項目を着手済み/未着手か）はこのドキュメントではなく、プロジェクト管理側で行う。
> ここには「あるべき姿」と、それを選ぶ理由・検討事項だけを書く。

## 現在オープンな方向性

現時点で未解決のアーキテクチャ方針はない。過去に検討した項目はいずれも決着済みで、
経緯は [decisions.md](./decisions.md) を参照:

- **中央 dispatch（`update → render → handle_actions`）**: 導入済み（ADR 0001）。
- **Actions 層の位置づけ**: 「段取り層」として残すと決定（ADR 0002）。
- **UI フレームワークの分離（egui 依存の抽象化）**: 追わないと決定（ADR 0008）。
  egui は即時モードで、抽象化しても乗り換え候補（保持モード/elm 的）へのコストは下がらず、
  投機的な間接層が負債になるため。

新たな方向性が生まれたらここに追記し、決着したら decisions.md に ADR として移す。

## 関連ドキュメント

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 現在のアーキテクチャ
- [STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md) - 現在の状態管理パターン
- [decisions.md](./decisions.md) - 設計判断記録（ADR）
