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

---

## 0002. Actions 層を「段取り層」として残す

- 状態: 採用
- 日付: 2026-06-21
- 関連: #139, #137

### 背景

中央 dispatch（0001）を導入した結果、`handle_actions` から呼ぶ各ドメインの `handle_xxx`
（`Actions<'a>` の impl）は `&mut AppState` を包んで mutator を呼ぶだけの薄いラッパに見える。
`Actions` 層を廃止して `handle_actions` か `AppState` に畳むか、層として残すかを検討した。

### 決定

`Actions` 層を **「intent を受けて AppState の原子的操作を段取りする層」** として残す。

### 理由

- 責務が分かれている: **AppState** = データ・不変条件・原子的操作（`add_location` 等）、
  **Actions(`handle_xxx`)** = intent を受けて原子的操作を段取りする層（例: location の
  `HandleAreaClick` の判定列）。畳むと段取りロジックが AppState か `handle_actions` に散る。
- `state/actions/*.rs` がドメイン単位の handler 置き場として機能している。
- **重要**: undo/redo・replay・ロギング等の横断フックを挟む単一点は `Actions` 層ではなく
  **`AmusApp::handle_actions`**（全 `AppAction` が通る所、0001 で導入済み）。これは
  `Actions` 層の存廃とは独立。よって「フックのために Actions を残す」必要はなく、
  残す理由はあくまで上記のコード構成（段取り層の分離）。

### 帰結

- `Actions` は `state/actions/` にドメイン分割で残す。
- 横断的な記録 / replay / undo が必要になったら `handle_actions` にフックを挟む
  （undo/redo は別途、Action の可逆化 or スナップショットが必要）。

---

## 0003. エラーは産出元ドメインごとに型付けする（横断メガ enum を作らない）

- 状態: 採用
- 日付: 2026-06-22
- 関連: #141

### 背景

各所が `Result<_, String>` を返しており、失敗理由が文字列に潰れて呼び手が分岐できない。
thiserror で型付けするにあたり、(a) クレート全体を束ねる単一の `AmusError` を作るか、
(b) ドメインごとに型を置くか、(c) `String` のまま残す境界をどこに引くか、を検討した。

### 決定

**産出元のドメインごとに独立したエラー型**を、その型を生むモジュールの近くに置く。

- `state`: `ColorError`（色変更の重複）
- `storage`: `StorageError`（serialize / deserialize、key と source を保持）
- リソース（ネイティブ）: `DownloadError` / `RecorderError` / `TranscribeError` /
  `TranscriberThreadError`

### 理由

- **消費側は全て Display 経由**（`eprintln!` / `log_error!` / 表示用フィールド）で、
  文字列内容による分岐はない。横断メガ enum にすると全モジュールが互いの依存クレート
  （cpal / whisper_rs / ureq 等）のエラー型を抱き込み、結合が広がるだけで利点がない。
- ドメインごとの型は失敗理由を呼び手が型で識別でき、`#[from]` で変換も簡潔になる。

### 帰結

- 表示専用の集約フィールド（`VoiceMemoState::error`、`TranscribeResult::error`）は
  ヘテロなエラーを束ねる**メッセージのキャッシュ**なので `String` のまま。境界で
  `e.to_string()` に変換する（これは伝搬する `Result` ではない）。
- **エラーを誰も使っていない箇所は typed error にせず、より正直なシグネチャにする**:
  `current_wave_mut` は `Option<&mut Wave>`、常に成功する `stop_recording` は `()` を返す。
  「`Result<_, String>` を機械的に typed error へ」ではなく、その値が本当に失敗を表すかで選ぶ。

---

## 0004. 公開 API はクレートルート（`lib.rs`）で閉じる（個別 pub の機械的絞り込みはしない）

- 状態: 採用
- 日付: 2026-06-22
- 関連: #145, #2

### 背景

`lib.rs` が全トップレベルモジュールを `pub mod` で公開しており、内部構造がまるごと
ライブラリの公開 API になっていた（bare `pub` が約 790 件）。これが「API 表面が広く
リファクタしづらい」原因。個々の項目を `pub` → `pub(crate)` に機械的に絞るか、
ルートで閉じるかを検討した。

### 決定

**`lib.rs` の `pub mod` を `mod`（private）にして、公開 API を wasm エントリの `start()`
だけに閉じる。** 個別項目の `pub` → `pub(crate)` 一括置換はしない。

### 理由

- wasm ライブラリの公開 API は `start()` のみ。モジュールを private にすれば、内部の
  `pub` 項目はクレート内で相互利用できるが**外部からは到達不能**になり、`pub(crate)` に
  落とすのと外部可視性の観点では等価。約 790 件を書き換える churn に見合う利点がない。
- ネイティブ側のデッドコード検出は bin クレート（`main.rs` 側のモジュールツリー、
  blanket allow なし）が担うため、ルートを閉じても検出力は落ちない。

### 帰結

- bin（`main.rs`）が唯一跨いでいた `memongus_wasm::log` 依存を解消し、自身の `crate::log`
  を使う。これで lib のモジュールを全て private 化できる。
- ルートを閉じた結果、**ネイティブ専用パス（ウィンドウアイコン・タイトル・カラー付き
  ログ整形・`voice_memo`）からのみ呼ばれる項目**が wasm でデッドコードとして顕在化した。
  これらは `#[cfg(not(target_arch = "wasm32"))]` で「ネイティブ専用」と明示する
  （blanket `allow(dead_code)` で覆い隠さない）。
- 個別の `pub(crate)`/`pub(super)` 絞りは、可視性をさらに文書化したい局所でのみ随時行う
  （`AppData` の `pub(in crate::state)` のように）。全面置換は目的化しない。

---

## 0005. サボタージュ種別を enum 化し、解決状態を集合で持つ

- 状態: 採用
- 日付: 2026-07-16
- 関連: #166

### 背景

「サボタージュ種別（comms / lights / o2 / reactor）」という単一のデータ軸が、feature
ディレクトリ単位で 6 レイヤー（view / mod / constants / Player の 4 bool / AppState の
`toggle_*` 4 メソッド / `WindowAction` の 4 バリアント）に横展開・コピペされていた。
差分は GridId・タイトルキー・パネル位置だけで、種別を 1 つ足すたびに横断的な複製が要る。

### 決定

`Sabotage` enum を導入し、種別ごとの差分（`title_key` / `grid_id` / `panel_offset_x` /
`ALL`）を enum の 1 箇所に集約する。Player の解決状態は `resolved: HashSet<Sabotage>` で持つ。

### 理由

- 「解決済みサボタージュの集合」を素直に表現でき、`contains` / `insert` / `remove` で
  判定・トグルが書ける。種別追加は enum への 1 バリアント追加に閉じ、view / action /
  state のコピペが不要になる。
- View は `Sabotage::ALL` を回して単一 `SabotageWindowView::render(.., kind)` を呼ぶだけ。

### 却下した案

- **`[bool; N]` 固定配列**: メモリは最小だが index と enum の対応を別途保つ必要があり、
  「解決済み集合」という意味が表に出ない。可読性を優先し集合表現を採った。

### 帰結

- `Player` は serde で永続化されるため、旧保存データ（4 bool 形式）の進行中ゲームは
  deserialize に失敗して復元されない（`load_from_storage` が握り潰すためクラッシュはしない）。
  ローカル用途につき許容した。

---

## 0006. Action のドメイン軸を Slices 軸に揃え、`WindowAction` を廃止する

- 状態: 採用
- 日付: 2026-07-16
- 関連: #170

### 背景

Slices は state 軸（game / player / setup / ui / wave）で分かれているが、Actions は feature 軸で、
`WindowAction` が player の resolved 操作（`ToggleSabotage`）と wave 操作（`SelectWave`）を
1 enum に同居させていた。Slices 側に `window` が無いことからも window は state ドメインでなく
UI グルーピングであり、Action enum 名から「どの state を触るか」が読めなかった。

### 決定

Action のドメイン軸を Slices 軸に揃える。`ToggleSabotage` を `PlayerAction` へ、`SelectWave` を
新設 `WaveAction::Select` へ移し、`WindowAction` / `AppAction::Window` を廃止する。

### 理由

- Action enum 名が「どの state を触るか」を表し、`handle_actions` の dispatch の見通しが上がる。
- UI 上のウィンドウという括りは view 側（`features/window/`）のディレクトリで表現すれば足り、
  Action 層に持ち込む必要はない。

### 帰結

- Action の軸は Slices と 1:1 対応に近づく（game / player / wave / …）。UI グルーピングと
  state ドメインを混同しない。

---

## 0007. Player の進行状態を `PlayerProgress` に分離する

- 状態: 採用
- 日付: 2026-07-16
- 関連: #171

### 背景

`Player` が永続属性（id / role / color / name）と、1 ゲーム中の進行状態（生死・死亡ターン・
ボタン・サボタージュ解決）を同一構造体に平坦化しており、記録項目が増えるたびに Player の
トップレベルが膨張する起点になっていた。

### 決定

進行状態を `PlayerProgress { state, death, done_button, resolved }` にまとめ、`Player` は
`progress` 1 フィールドで持つ。`is_dead` / `is_ejected` / `is_resolved` は `progress` へ委譲する。

### 理由

- 永続属性と 1 ゲーム限りの進行状態が型で分かれ、進行項目の増減が `PlayerProgress` に閉じる。
- 委譲メソッドを Player に残すことで呼び出し側は不変（`player.is_dead()` のまま）。

### 却下した案

- **setup 用 / game 用で型分離（SetupPlayer / GamePlayer）**: setup / game 間の同期境界は
  最も明示的になるが、`setup_state` / `create_game_from_setup` / プレイヤー同期の各所に
  変更が波及して差分が大きい。今回は内包案（setup 用 Player は `progress` を default のまま持つ）
  を採り、型分離は将来 #169（両コレクション同期の集約）と併せて再検討する余地を残した。

---

## 0008. UI フレームワークの分離（egui 依存の抽象化）は追わない

- 状態: 採用（やらないことの決定）
- 日付: 2026-07-16
- 関連: #173

### 背景

かつて ARCHITECTURE_ROADMAP に「UI フレームワークの分離 — View 層を egui 以外へも
載せ替えられるようにする」を目指す姿として掲げ、View が受け取る描画インターフェースを
Adapter 化して `ui: &mut Ui` をトレイト越しに扱う案（#173）があった。着手するか検討した。

### 決定

**egui 依存の抽象化（UI フレームワーク分離）は追わない。** #173 はクローズし、ロードマップの
当該目標を撤回する。

### 理由

- egui は**即時モード**で、描画とイベント取得が同一呼び出し（`ui.button().clicked()`）に
  一体化している。これは egui のモデルそのもので、Adapter で覆っても即時モード概念を
  漏らすか、抽象化しすぎて実フレームワークのどれにも綺麗に対応しない。
- 乗り換え候補（iced / gpui / Dioxus / bevy_ui 等）は**保持モード or elm 的**で描画モデルが
  根本的に異なる。実際に乗り換えるなら View 層は結局書き直しになり、投機的なアダプタでは
  乗り換えコストがほとんど下がらない。
- 本アプリはクロスプラットフォーム前提ではなく、乗り換えの現実的要求もない。投機的抽象化は
  利点が薄く、負債（不自然な間接層）だけが残る。

### 帰結

- View は従来どおり `ui: &mut Ui` を受け取って描画する（現状維持）。
- 「入力＋状態 → どの Action を出すか」の判断ロジックにテスト価値が出た場合は、その View
  単位で `ui` を触らない純関数へ切り出してテストすればよい（framework 非依存の局所対応で
  足り、横断的な抽象化層は設けない）。
