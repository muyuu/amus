# アーキテクチャ設計指針

## 概要

このドキュメントは、チームのオンボーディングと設計判断の参照用に、決定されたアーキテクチャ設計を説明します。

## 基本構成

### 一連のデータフロー

```
Model (データ)
    ↓ 参照
State (状態管理)
    ↓ 参照
View (描画)
    ↓ 実行
Interaction (インタラクション処理)
    ↓ 更新
Model更新 (データ更新)
    ↓ 更新
State更新 (状態更新)
    ↓ 反映
Model (更新された状態)
```

### レイヤー構成

```
AppState (アプリケーション全体の状態)
    ├─ AppData (実際のデータを保持)
    │   ├─ Model (Game, Player, Wave, Route など)
    │   ├─ UI状態 (選択中のプレイヤー、ドラッグ状態など)
    │   └─ 設定 (SetupState, Translator など)
    └─ 各feature
        ├─ Feature (機能のエントリーポイント)
        ├─ View (描画)
        └─ Interaction (インタラクション処理、オプション)
```

**重要**:
- **画面に描画されているものは全てfeatureとして扱う**
- `app.rs`は`AppState`と各featureだけを使用する
- Stateは`AppState`（`RefCell<AppData>`を内包）で一元管理
- Viewには必要に応じてInteractionが対になる
- データフローは `model > state > view > interaction > model更新 > state更新` で統一

## レイヤー構成

### 1. Model (`src/models/`)

**役割**: 永続化可能なデータ（ストレージに保存するデータ）

- シリアライズ可能（`Serialize`/`Deserialize` を実装）
- アプリ再起動後に復元されるデータ
- ビジネスロジックを持たない（純粋なデータ構造）

**現在のモデル**:
- `Game` - ゲーム全体の状態（プレイヤー、ウェーブ、エリア）
- `Player` - プレイヤー情報（名前、色、役職、状態）
- `Wave` - 各ターンの状態（出現位置、終了位置、ルート）
- `Route` - プレイヤーの移動軌跡
- `Area` - マップエリア（Skeld, Mira, Polus, Airship）
- `Color` - プレイヤーの色
- `Role` - 役職（Crew, Impostor）
- `Point` - 座標データ
- `Theme` - UIテーマ

### 2. State (状態管理)

> **詳細**: `AppData`と`AppState`の関係性、設計意図、実装パターンについては [状態管理パターン](./STATE_MANAGEMENT.md) を参照してください。

#### AppState (`src/state/app_state.rs`)

**役割**: アプリケーション全体の状態管理

- `RefCell<AppData>`を内包し、内部可変性を提供
- 各種データへのアクセスメソッドを提供
- ストレージへの保存・読み込みを担当
- **重要**: データの所有権を管理し、`mut`アクセスを限定的にする

**主な機能**:
- ゲーム管理（`start_new_game`, `create_game_from_setup`, `reset_game`）
- プレイヤー操作（`toggle_player_state`, `update_player_name`, `update_player_color`）
- UI状態管理（`selected_player_id`, `erase_mode`, `show_debug_view`）
- ドラッグ&ドロップ管理
- ウェーブ・ルート管理
- ストレージ操作（`load_from_storage`, `save_to_storage`）

#### AppData (`src/state/app_data.rs`)

**役割**: 実際のアプリケーションデータを保持（純粋なデータコンテナ）

- ビジネスロジックを持たない（データ構造のみ）
- `AppState`経由でのみアクセスされる
- **重要**: 実装側は直接`AppData`を触らず、必ず`AppState`経由でアクセスする

```rust
pub struct AppData {
    pub asset_manager: Option<AssetManager>,  // 画像リソース管理
    pub current_wave_index: usize,            // 現在のターン
    pub dragging_location: Option<DraggingLocation>,  // ドラッグ状態
    pub dragging_player_id: Option<PlayerId>,
    pub editing_name_player_id: Option<PlayerId>,
    pub erase_mode: bool,
    pub game: Option<Game>,
    pub selected_player_id: Option<PlayerId>,
    pub setup_state: SetupState,
    pub show_debug_view: bool,
    pub show_setup_dialog: bool,
    pub translator: Translator,
}
```

#### その他のState

- `SetupState` (`src/state/setup_state.rs`) - ゲーム設定時の状態
- `DraggingLocation` (`src/state/location.rs`) - 位置ドラッグの状態管理
- `StorageKeys` (`src/state/storage_keys.rs`) - ストレージキーの定数
- `AppStorage` (`src/state/storage.rs`) - ストレージ操作ユーティリティ

### 3. Feature (`src/features/<feature_name>/`)

**役割**: 機能単位のモジュール

各featureは以下の構成を持つ:
- `mod.rs` - モジュール定義とFeature構造体
- `view.rs` - UI描画
- `interaction.rs` - インタラクション処理（オプション）
- `constants.rs` - 定数定義（オプション）

**典型的なFeature構成**:

```rust
pub struct SomeFeature;

impl SomeFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        let result = SomeView::render(state, ui);
        SomeInteraction::handle(state, &result);
    }
}
```

### 4. View (`src/features/<feature_name>/view.rs`)

**役割**: UI描画のみ

- Stateを参照して描画
- ユーザー操作の結果（クリック、ドラッグなど）を返す
- 直接ModelやStateを更新しない

### 5. Interaction (`src/features/<feature_name>/interaction.rs`)

**役割**: インタラクション処理とState更新

- Viewから受け取った操作結果を処理
- AppStateのメソッドを通じてデータを更新

## 現在のFeature一覧

| Feature | 説明 | Interaction |
|---------|------|-------------|
| `debug_view` | デバッグ情報の表示 | なし |
| `eraser` | 消しゴムモード（軌跡削除） | あり |
| `location` | 出現位置・終了位置の配置・移動 | あり |
| `main` | メイン画面（マップ表示エリア） | なし |
| `map` | マップ画像の表示 | なし |
| `player_info` | サイドバーのプレイヤー情報表示 | あり |
| `player_list` | 下部のプレイヤー一覧 | なし |
| `route_drawing` | プレイヤーの移動軌跡描画 | あり |
| `setup_dialog` | ゲーム設定ダイアログ | あり |
| `turn` | ターン選択UI | あり |
| `welcome` | ウェルカム画面 | なし |

## その他のモジュール

### assets (`src/assets/`)

**役割**: 画像リソースの管理

- `AssetManager` - マップ画像などのアセットを読み込み・管理

### common (`src/common/`)

**役割**: 共通ユーティリティ

- `position` - 座標計算のユーティリティ
- `texts` - 共通テキスト定義

### components (`src/components/`)

**役割**: 再利用可能なUIコンポーネント

- `player` - プレイヤー表示コンポーネント
- `text` - テキスト表示コンポーネント

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

**役割**: アプリケーション全体の定数

- 色定義（`WINDOW_BG_COLOR_DARK`など）
- サイズ定義

## ディレクトリ構造

```
src/
├── app.rs               # アプリケーションエントリーポイント
├── main.rs              # メイン関数
├── lib.rs               # ライブラリルート
├── constants.rs         # グローバル定数
├── assets/              # アセット管理
├── common/              # 共通ユーティリティ
├── components/          # 再利用可能なUIコンポーネント
├── features/            # 機能単位のモジュール
├── i18n/                # 多言語対応
├── log/                 # ロギング
├── models/              # データモデル
└── state/               # 状態管理
```

## Feature の粒度

### Feature の定義

「機能」とは、ユーザーが認識できる独立した操作単位です。

**重要な原則**: **画面に描画されているものは全てfeatureとして扱う**

### Feature の分割基準

1. **データの独立性**: 機能が独自のデータを操作する
2. **UIの独立性**: 機能が独自のUI要素を持つ
3. **操作の独立性**: 機能が独立した操作フローを持つ

### Interactionの必要性

**Interactionが必要なfeature**:
- ユーザー操作に応じてStateを更新する必要がある
- クリック、ドラッグなどのイベントを処理する

**Interactionが不要なfeature**:
- 表示のみで、ユーザー操作を処理しない
- 他のfeatureから呼び出される描画専用コンポーネント

## データフロー

### 統一されたデータフロー

```
AppState.data (AppData) → Feature.render() → View.render() → Interaction.handle() → AppState更新
```

### フレーム N: 描画とインタラクション

```
1. AppState.data を参照
   ↓
2. Feature.render() を呼び出し
   ↓
3. View.render() で描画（操作結果を返す）
   ↓
4. Interaction.handle() で操作を処理
   ↓
5. AppStateのメソッドを通じてデータを更新
```

### フレーム N+1: 更新された描画

```
1. 更新された AppState.data を参照
   ↓
2. Feature.render() を呼び出し
   ↓
3. View.render() で描画（変更が反映される）
```

### 具体例

```rust
// features/location/mod.rs
pub struct LocationFeature;

impl LocationFeature {
    pub fn render(state: &mut AppState, response: &Response, ui: &mut Ui) {
        // 1. Viewで描画し、操作結果を取得
        let result = LocationView::render(state, ui);
        // 2. Interactionで操作を処理
        LocationInteraction::handle(state, &result, response, ui);
    }
}

// features/location/interaction.rs
impl LocationInteraction {
    pub fn handle(state: &mut AppState, result: &LocationResult, response: &Response, ui: &mut Ui) {
        // AppStateのメソッドを通じて更新
        if let Some(player_id) = result.clicked_player {
            state.set_selected_player_id(Some(player_id));
        }
        if let Some((player_id, point)) = result.dropped_spawn {
            state.add_spawn_location(player_id, point);
        }
    }
}
```

## 命名規則

- **Model**: PascalCase（例: `Game`, `Player`, `Wave`）
- **Feature**: `<FeatureName>Feature`（例: `LocationFeature`）
- **View**: `<FeatureName>View`（例: `LocationView`）
- **Interaction**: `<FeatureName>Interaction`（例: `LocationInteraction`）
- **State**: `AppState`, `AppData`, `SetupState`

## 禁止事項

### ❌ 禁止: Viewから直接AppDataを更新

```rust
// ❌ 悪い例
impl LocationView {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        // Viewから直接データを更新しない
        state.data_mut().selected_player_id = Some(player_id);
    }
}
```

**正しい例**: ViewはInteractionに処理を委譲

```rust
// ✅ 良い例
impl LocationView {
    pub fn render(state: &mut AppState, ui: &mut Ui) -> LocationResult {
        // 操作結果を返すだけ
        LocationResult { clicked_player: Some(player_id) }
    }
}

impl LocationInteraction {
    pub fn handle(state: &mut AppState, result: &LocationResult) {
        // Interactionで更新
        if let Some(player_id) = result.clicked_player {
            state.set_selected_player_id(Some(player_id));
        }
    }
}
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
2. **各Feature**: 画面に描画される全ての機能

```rust
// app.rs
impl eframe::App for AmusApp {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        // セットアップダイアログの表示
        if self.state.show_setup_dialog() {
            SetupView::show(&mut self.state, ctx);
            return;
        }

        // メニューUI
        self.build_menu_ui(ctx, frame);

        // メインUI
        SidePanel::right("player_info_panel").show(ctx, |ui| {
            PlayerInfoFeature::render(&mut self.state, ui);
        });

        CentralPanel::default().show(ctx, |ui| {
            MainView::render(&mut self.state, ui);
        });

        TopBottomPanel::bottom("player_list_panel").show(ctx, |ui| {
            PlayerListView::show(&mut self.state, ui);
        });
    }
}
```

## 参考

- eframe/eguiの状態管理: `eframe::App`トレイトを実装する構造体が状態を保持
- AppStateはeframe/eguiの機能ではなく、独自に定義した構造体
- `RefCell`を使用した内部可変性パターンにより、借用の競合を回避
