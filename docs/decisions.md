# 設計判断記録（ADR）

採用した設計の「なぜ」を残す。決定そのもの（あるべき構成）は
[ARCHITECTURE.md](./ARCHITECTURE.md) 等に書き、ここには選定理由と却下した案を記録する。

---

## 0001. View→Actions を中央 dispatch にする（縦スライスではなく）

- 状態: 採用
- 日付: 2026-06-21
- 関連: #137, #139

### 背景

各 Feature が `render` 内で `Vec<Action>` を作り、その場で `Actions::new(state).handle_xxx(action)` まで
一気に dispatch していた。`Vec` を作って即消費するだけで、`&mut AppState` が描画ツリー全体を貫通する。
フレームの流れが feature ごとにバラバラで、状態変更を観測する単一点がない。

「アプリのフローを `update → render → action` の1本道に統一できないか」「フレームワーク(egui)の流儀として
何が良いか」「チーム開発で担当 feature だけ見ればよい構成（縦スライス）と両立するか」を検討した。

### 検討した選択肢

1. **縦スライス**: feature ディレクトリが view + action + handler を全部所有。`state/actions/` を
   各 feature へ引っ越し、AppState は原子的 mutation に痩せる。「自分の1ディレクトリだけ見ればよい」。
2. **中央 dispatch（水平レイヤ）**: View は intent(Action) を返すだけ。`AmusApp::handle_actions` が
   1 箇所で適用。`update → render → action` の3段に統一。
3. **ハイブリッド**: ドメインの core（不変条件・原子的操作）は中央(AppState)、プレゼンテーションは
   feature スライス。

### 決定

**中央 dispatch（2 を core 中央のハイブリッド 3 として実現）** を採用する。

`update()` を次の3段に統一する:

1. `resources.update(ctx)` … リソースの定期更新（タイマー・非同期完了・voice_memo の VAD/ポーリング等）
2. `render_views(ctx) -> Vec<AppAction>` … render は `&AppState` 読み取りのみで Action を返す
3. `handle_actions(Vec<AppAction>)` … `&mut` で 1 箇所に集約して dispatch

### 理由

- **このアプリは「1つの共有ドメイン（ゲーム状態: players / waves / routes / 選択）への複数 View」型**で、
  独立したビジネス能力（課金・認証…各自データを持つ）の集合ではない。縦スライスが規模で勝つのは
  独立能力型のとき。共有ドメイン型では不変条件が各スライスに散り、共有状態のバグが規模で増える。
  ゲームのルール（色重複禁止・wave 範囲・選択と描画の整合）は中央(AppState)に集約する方が
  規模での認知負荷が低い。
- **egui 即時モード＋Rust の借用と相性が良い**。render が `&AppState` 読み取りに揃い、`&mut` を
  描画ツリーに通さなくて済む。mutation は1フェーズに閉じる。
- 「`update → render → action` の1本道」はフレーム全体の見通しを良くし、新 feature も同じ型に従う。

### 帰結

- 全 Feature の `render` は `Vec<AppAction>` を返す（egui のクロージャは `.inner` で回収して上へ畳む）。
- `AppAction` enum が各ドメイン Action を束ねる。`handle_actions` がドメイン handler へ委譲する。
- feature の責務は **view（intent を出す）＋ その feature の handler**。AppState はドメインの core
  （データ・不変条件・原子的操作）を持ち続ける。
- voice_memo も例外にせず3段に乗せる（① で VAD/ポーリング、② で `VoiceMemoAction` を返す、
  ③ で Resources 込みに適用）。「特別」を1つ作ると増殖するため、統一を優先する。

### 却下した案（縦スライス）の理由

独立能力型アプリでは「変更の局所性」で規模に強い。しかし本アプリは共有ドメイン型のため、
ロジックを feature へ分散するとドメインの不変条件が散り、横断的な一貫性ドリフトと共有状態バグが
規模で増える。よって採らない。
