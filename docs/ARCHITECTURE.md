# アーキテクチャ設計指針

## 概要

このドキュメントは、チームのオンボーディングと設計判断の参照用に、決定されたアーキテクチャ設計を説明します。

## 基本構成

### 一連のデータフロー

```
Model (データ)
    ↓ 保持
AppData (データコンテナ)
    ↓ アクセス制御
AppState (状態管理 + ビジネスロジック)
    ├─ Slices  (読み取り専用アクセス)
    └─ Actions (状態変更)
    ↓
Feature → View (描画・操作結果を返す)
    ↓ 操作結果
Actions (状態更新)
    ↓
AppState 更新
```

### レイヤー構成

```
AppState (アプリケーション全体の状態)
    ├─ AppData (実際のデータを保持)
    │   ├─ Model (Game, Player, Wave, Route など)
    │   ├─ UI状態 (選択中のプレイヤー、ドラッグ状態など)
    │   └─ 設定 (SetupState, Translator など)
    ├─ Slices  (読み取り専用ビュー)
    └─ Actions (状態変更ハンドラ)

各 feature
    ├─ Feature (機能のエントリーポイント。Slices→View→Actions を配線)
    └─ View    (描画。Slices を受け取り、操作結果を返す)
```

**重要**:

- **画面に描画されているものは全てfeatureとして扱う**
- `app.rs`は`AppState`と各featureだけを使用する
- 状態は`AppState`が`AppData`を所有して一元管理する（内部可変性は使わず、`&self`/`&mut self`で借用を表現する）
- Viewは`Slices`（読み取り専用）を受け取り、状態を直接変更せず操作結果を返す
- 操作結果は`Actions`を通じて`AppState`に反映する
- データフローは `Model > AppData > AppState(Slices/Actions) > View > Actions > AppState更新` で統一

## レイヤー構成

### 1. Model (`src/models/`)

**役割**: 永続化可能なデータ（ストレージに保存するデータ）

- シリアライズ可能（`Serialize`/`Deserialize` を実装）
- アプリ再起動後に復元されるデータ
- ビジネスロジックを持たない（純粋なデータ構造。値の整合を保つ程度のシンプルなヘルパーは持ってよい）
- UIフレームワークに依存しない（egui の型を持ち込まない）

**現在のモデル**:
- `Game` - ゲーム全体の状態（プレイヤー、ウェーブ、エリア）
- `Player` / `PlayerId` / `PlayerState` - プレイヤー情報（名前、色、役職、状態）
- `Wave` - 各ターンの状態（出現位置、終了位置、ルート）
- `Route` - プレイヤーの移動軌跡
- `Area` - マップエリア（Skeld, Mira, Polus, Airship）
- `Color` - プレイヤーの色
- `Role` - 役職（Crew, Impostor）
- `Point` - 座標データ
- `Theme` - UIテーマ
- `DraggingLocation` / `LocationType` - 位置ドラッグの種別と対象
- `SetupState` - ゲーム設定時の状態

### 2. State (状態管理)

> **詳細**: `AppData`と`AppState`の関係性、設計意図、実装パターンについては [状態管理パターン](./STATE_MANAGEMENT.md) を参照してください。

#### AppState (`src/state/app_state/`)

**役割**: アプリケーション全体の状態管理

- `AppData`を値で所有する（`RefCell`等の内部可変性は使わない）
- 読み取りは`slices()`が返す`Slices`、書き込みは各種メソッド / `Actions`を経由させる
- ストレージへの保存・読み込みを担当
- **重要**: データの所有権を管理し、`&mut`アクセスをメソッド内に閉じ込める

肥大化を避けるため、ドメイン単位でファイル分割している（同一の型に複数ファイルで`impl`する）:

```
src/state/app_state/
├── mod.rs        # 構造体定義 + コンストラクタ + slices()
├── game.rs       # ゲーム管理 (start_new_game, create_game_from_setup, reset_game)
├── player.rs     # プレイヤー操作 (toggle_player_state, update_player_name, ...)
├── wave.rs       # ウェーブ・ルート管理 (current_wave, push_route, ...)
├── ui.rs         # UI状態 (selected_player_id, erase_mode, setup_state, ...)
├── drag_drop.rs  # ドラッグ&ドロップ、位置情報 (add_location, ...)
└── storage.rs    # ストレージ操作 (load_from_storage, save_to_storage)
```

#### AppData (`src/state/app_data.rs`)

**役割**: 実際のアプリケーションデータを保持（純粋なデータコンテナ）

- ビジネスロジックを持たない（データ構造のみ）
- フィールドは`pub(in crate::state)`に絞られており、`state`モジュールの外（Feature / View 等）からは直接アクセスできない。読み取りは`Slices`、書き込みは`Actions` / `AppState`メソッドを経由する規約をコンパイラが強制する

```rust
pub struct AppData {
    pub(in crate::state) current_wave_index: usize,
    pub(in crate::state) dragging_location: Option<DraggingLocation>,
    pub(in crate::state) dragging_player_id: Option<PlayerId>,
    pub(in crate::state) editing_name_player_id: Option<PlayerId>,
    pub(in crate::state) erase_mode: bool,
    pub(in crate::state) game: Option<Game>,
    pub(in crate::state) selected_player_id: Option<PlayerId>,
    pub(in crate::state) setup_state: SetupState,
    pub(in crate::state) show_debug_view: bool,
    pub(in crate::state) show_setup_dialog: bool,
    pub(in crate::state) translator: Translator,
}
```

#### Slices (`src/state/slices/`)

**役割**: ドメイン単位の読み取り専用アクセス

`AppState::slices()`が返す`Slices`を入口に、`GameSlice` / `PlayerSlice` / `WaveSlice` / `UiSlice` / `SetupSlice`へアクセスする。Viewはこの`Slices`だけを受け取り、`AppData`を直接読まない。

```rust
let slices = state.slices();
let players = slices.player().players();
let area = slices.game().area();
```

#### Actions (`src/state/actions/`)

**役割**: 状態変更の集約

Feature は状態を直接変更せず、`AppAction`（各ドメイン Action を束ねた enum、`src/app_action.rs`）を返す。
`AmusApp::handle_actions` が 1 箇所でまとめて dispatch し、ドメイン handler（`handle_player` /
`handle_location` など、ドメイン単位で別ファイルに`impl`）へ委譲する（中央 dispatch）。

```rust
// AmusApp::handle_actions（抜粋）
match action {
    AppAction::Player(a) => Actions::new(&mut self.state).handle_player(a),
    AppAction::Location(a) => Actions::new(&mut self.state).handle_location(a),
    // ...
}
```

#### その他のState

- `StorageKeys` (`src/state/storage_keys.rs`) - ストレージキーの定数
- `AppStorage` (`src/state/storage.rs`) - ストレージ操作ユーティリティ

### 3. Feature (`src/features/<feature_name>/`)

**役割**: 機能単位のモジュール

各featureは以下の構成を持つ:
- `mod.rs` - モジュール定義とFeature構造体（描画して `Vec<AppAction>` を返すエントリーポイント）
- `view.rs` - UI描画
- `constants.rs` - 定数定義（オプション）

**render の規約**:

- 読み取りは `&Slices` で受ける（`&AppState` を見せない）。
- 戻り値は必ず `Vec<AppAction>`（描画のみで操作が無い feature も空 Vec を返す）。
- 状態は変更しない（変更は `AmusApp::handle_actions` が一括で行う）。
- egui の入力は描画先に応じて必要なものだけ受ける（基本は `&mut Ui`。`ctx` は `ui.ctx()` から取るので受けない。中央マップ領域を共有する location/route/map のみ `&Response` を併せて受ける）。

```rust
pub struct PlayerListFeature;

impl PlayerListFeature {
    pub fn render(slices: &Slices, ui: &mut Ui) -> Vec<AppAction> {
        PlayerListView::render(slices, ui)
            .into_iter()
            .map(AppAction::Player)
            .collect()
    }
}
```

### 4. View (`src/features/<feature_name>/view.rs`)

**役割**: UI描画のみ

- `Slices`（読み取り専用）を参照して描画する
- ユーザー操作の結果（クリック、ドラッグなど）を Action 値として返す
- 直接ModelやStateを更新しない

## 現在のFeature一覧

| Feature | 説明 |
|---------|------|
| `debug_view` | デバッグ情報の表示 |
| `eraser` | 消しゴムモード（軌跡削除） |
| `location` | 出現位置・終了位置の配置・移動 |
| `main` | メイン画面（マップ表示エリア） |
| `map` | マップ画像の表示 |
| `player_info` | サイドバーのプレイヤー情報表示 |
| `player_list` | 下部のプレイヤー一覧 |
| `route_drawing` | プレイヤーの移動軌跡描画 |
| `setup_dialog` | ゲーム設定ダイアログ |
| `voice_memo` | 音声メモの録音・書き起こし（ネイティブのみ） |
| `welcome` | ウェルカム画面 |
| `window` | ターン選択や各タスク窓（`comms` / `lights` / `o2` / `reactor` / `turn`） |

## その他のモジュール

### resources (`src/resources/`)

**役割**: ハードウェアや重い初期化を必要とするリソースの管理

- `AppData`に含めない（シリアライズ不可・永続化対象外）
- `AssetManager` - マップ画像・プレイヤー画像などのテクスチャを読み込み・管理（egui の `Context` 経由でシングルトン管理）
- 音声メモ用リソース（ネイティブのみ）: `VoiceRecorder`、`WhisperTranscriber`、モデルダウンロード・書き起こしスレッドなど

### common (`src/common/`)

**役割**: 共通ユーティリティ（座標計算、レイアウト、egui との色変換アダプタなど）

### components (`src/components/`)

**役割**: 再利用可能なUIコンポーネント（`grid`、`tile`、`select`、`window`、`change_color` など）

### i18n (`src/i18n/`)

**役割**: 多言語対応（国際化）

- `Translator` - 翻訳処理
- `Language` - 言語選択（Japanese, English）
- `keys` - 翻訳キーの定数
- `ja`, `en` - 各言語の翻訳定義
- `words/` - 単語レベルの翻訳定義

### log (`src/log/`)

**役割**: アプリケーションロギング

- `LogLevel` - ログレベル（Error, Warn, Info, Debug, Trace）
- `Logger` - ログ出力
- WASM/ネイティブ両対応

### constants (`src/constants.rs`)

**役割**: アプリケーション全体の定数（色定義・サイズ定義など）

## ディレクトリ構造

```
src/
├── app.rs               # アプリケーション本体（AmusApp）
├── main.rs              # ネイティブのエントリーポイント
├── lib.rs               # wasm のエントリーポイント / ライブラリルート
├── constants.rs         # グローバル定数
├── resources/           # リソース管理（画像テクスチャ・音声など）
├── common/              # 共通ユーティリティ
├── components/          # 再利用可能なUIコンポーネント
├── features/            # 機能単位のモジュール
├── i18n/                # 多言語対応
├── log/                 # ロギング
├── models/              # データモデル
└── state/               # 状態管理（app_data / app_state / slices / actions / storage）
```

## Feature の粒度

### Feature の定義

「機能」とは、ユーザーが認識できる独立した操作単位です。

**重要な原則**: **画面に描画されているものは全てfeatureとして扱う**

### Feature の分割基準

1. **データの独立性**: 機能が独自のデータを操作する
2. **UIの独立性**: 機能が独自のUI要素を持つ
3. **操作の独立性**: 機能が独立した操作フローを持つ

## データフロー

毎フレーム（`AmusApp::update`）を 3 段に統一する（中央 dispatch）。詳細・選定理由は
[decisions.md](./decisions.md) 0001 を参照。

```
① resources.update(ctx)              リアルタイム更新（タイマー・非同期・voice_memo の VAD/ポーリング）
② render_views(ctx) → Vec<AppAction> render は &AppState 読み取りのみ。各 Feature は Action を返すだけ
③ handle_actions(Vec<AppAction>)     &mut で 1 箇所に集約して dispatch → AppState 更新
```

- render(②)は読み取り専用なので `&mut AppState` を描画ツリーに通さない。
- egui のクロージャからは `.inner` で `Vec<AppAction>` を回収して上位へ畳む。
- 状態変更は③の 1 箇所だけで起きる。

### 具体例

```rust
// features/location/mod.rs — render は Action を返すだけ
impl LocationFeature {
    pub fn render(slices: &Slices, response: &Response, ui: &mut Ui) -> Vec<AppAction> {
        let result = LocationView::render(slices, ui);

        let mut location_actions = Vec::new();
        if response.clicked() {
            if let Some(point) = current_point_with_ui(ui) {
                location_actions.push(LocationAction::HandleAreaClick(point));
            }
        }
        // ...（ドラッグ・色変更・削除なども同様に Action へ）

        location_actions.into_iter().map(AppAction::Location).collect()
    }
}

// app.rs — 収集した Action を 1 箇所で dispatch
fn handle_actions(&mut self, actions: Vec<AppAction>) {
    for action in actions {
        match action {
            AppAction::Location(a) => Actions::new(&mut self.state).handle_location(a),
            // ... 他ドメインも同様
        }
    }
}
```

## 命名規則

- **Model**: PascalCase（例: `Game`, `Player`, `Wave`）
- **Feature**: `<FeatureName>Feature`（例: `LocationFeature`）。単数形は常に「1 つの機能」を指す。
- **View**: `<FeatureName>View`（例: `LocationView`）
- **Slice**: `<Domain>Slice`（例: `PlayerSlice`, `WaveSlice`）
- **Action**: `<Domain>Action`（例: `PlayerAction`, `LocationAction`）
- **State**: `AppState`, `AppData`, `SetupState`

### 単数 Feature と集約 `StatefulFeatures`

Feature には状態を持つものと持たないものがある。

- **stateless Feature**: フレームをまたぐ状態を持たない。unit struct `<Name>Feature`
  （例: `LocationFeature`）として定義し、`render` を静的に呼ぶだけ。インスタンスは持たない。
- **stateful Feature**: フレームをまたいで状態を保持する（例: `voice_memo` の録音・書き起こし状態）。
  インスタンスを `StatefulFeatures`（`src/features/mod.rs`）に集約し、`AmusApp` が所有する。

複数形の型は `StatefulFeatures` の 1 つだけで、「状態を持つ Feature インスタンスの集約」を意味する。
単数 `<Name>Feature` と取り違えないよう、集約側は `Features` のような汎称ではなく
`StatefulFeatures` と役割を名前に出す。

## 禁止事項

### ❌ 禁止: Viewから直接Stateを更新

```rust
// ❌ 悪い例: View が状態を直接書き換える
impl LocationView {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        state.set_selected_player_id(Some(player_id));
    }
}
```

**正しい例**: ViewはSlicesで読み、操作結果をActionとして返す

```rust
// ✅ 良い例
impl LocationView {
    pub fn render(slices: &Slices<'_>, ui: &mut Ui) -> LocationResult {
        // 読み取りは slices、書き込みはせず結果を返すだけ
        LocationResult { /* clicked / dragged など */ }
    }
}

// Feature 側で Actions に渡して更新する
let mut actions = Actions::new(state);
actions.handle_location(action);
```

### ❌ 禁止: Modelから直接UIを操作

```rust
// ❌ 悪い例
impl Game {
    pub fn render(&self, ui: &mut Ui) {
        // ModelはUIを知らない
    }
}
```

## App の役割

### app.rs の構成

`app.rs`は以下のみを使用:

1. **AppState**: アプリケーション全体の状態
2. **Resources**: 描画に必要なリソース（テクスチャ・音声など）
3. **各Feature**: 画面に描画される全ての機能

```rust
// app.rs（抜粋）— update は ①リアルタイム更新 → ②描画して収集 → ③dispatch の 3 段
impl eframe::App for AmusApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if self.state.show_setup_dialog() {
            let actions = SetupDialogFeature::render(&self.state, ctx);
            self.handle_actions(actions);
            return;
        }

        // ① リアルタイム更新（リソース系のステートフル feature）
        #[cfg(not(target_arch = "wasm32"))]
        self.features.voice_memo.update(&self.resources, ctx);

        self.build_menu_ui(ctx, frame);

        // ② 描画して Action を収集（render は &AppState 読み取りのみ）
        let actions = self.build_main_ui(ctx, frame);

        // ③ 1 箇所で dispatch
        self.handle_actions(actions);
    }
}
```

## 参考

- eframe/eguiの状態管理: `eframe::App`トレイトを実装する構造体が状態を保持する
- AppStateはeframe/eguiの機能ではなく、独自に定義した構造体
- 状態は`AppState`が`AppData`を値で所有して管理する。読み取り(`Slices`)と書き込み(`Actions`)を型で分離することで、Viewからの不用意な変更を防ぐ
