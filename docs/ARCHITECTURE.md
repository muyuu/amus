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
    ├─ Model (グローバルなデータ)
    └─ 各feature
        ├─ Feature Model (必要に応じて)
        ├─ State (機能固有の状態)
        ├─ View (描画)
        └─ Interaction (インタラクション処理)
```

**重要**: 
- **画面に描画されているものは全てfeatureとして扱う**
- `app.rs`は`AppState`と各featureだけを使用する
- Stateは2種類ある: `AppState`（アプリケーション全体）と各featureの`State`（機能固有）
- Viewには必ず対になるinteraction.rsがある
- データフローは `model > state > view > interaction > model更新 > state更新` で統一

## レイヤー構成

### 1. Model (`src/models/`)

**役割**: 永続化可能なデータ（ストレージに保存するデータ）

- シリアライズ可能（`Serialize`/`Deserialize` を実装）
- アプリ再起動後に復元されるデータ
- ビジネスロジックを持たない（純粋なデータ構造）

**例**: `Game`, `Wave`, `User`, `Route`

### 2. State (状態管理)

Stateには2種類あります：

#### AppState (`src/game/state.rs`)

**役割**: アプリケーション全体の状態管理

- Modelを保持（`model: AppModel`）
- グローバルな状態（現在のwave、選択中のユーザーなど）
- 各featureのStateを保持
- 共通リソース（画像、翻訳など）

**重要**: AppStateはModelを保持するだけで、直接更新しない

#### 各featureのState (`src/features/<feature_name>/state.rs`)

**役割**: 機能固有の状態管理

- UI状態（選択中、ドラッグ中など）
- Model操作メソッド（Modelを更新する）
- State更新メソッド（State自身を更新する）

**例**: `LocationState`, `RouteDrawingState`

### 3. View (`src/features/<feature_name>/view.rs`)

**役割**: UI描画のみ

- Modelを参照して描画
- Stateを参照して描画の挙動を変える
- Interactionを実行する（インタラクション検出時に呼び出す）

**例**: `LocationView`, `RouteDrawingView`

**重要**: Viewは描画のみを担当し、ModelやStateを更新しない

### 4. Interaction (`src/features/<feature_name>/interaction.rs`)

**役割**: インタラクション処理とModel/State更新

- インタラクションを処理する
- Modelを更新する
- Stateを更新する

**例**: `LocationInteraction`, `RouteDrawingInteraction`

**重要**: 
- Viewには必ず対になるinteraction.rsがある
- InteractionはModelとStateを更新する唯一の場所

## ディレクトリ構造

```
src/
├── models/              # グローバルなデータモデル
│   ├── game.rs
│   ├── wave.rs
│   └── mod.rs
├── game/                # アプリケーション状態管理
│   ├── state.rs         # AppState
│   └── mod.rs
├── features/            # 機能単位のモジュール
│   ├── main/            # メイン画面機能
│   │   ├── view.rs      # MainView (描画のみ)
│   │   ├── interaction.rs # MainInteraction (インタラクション処理)
│   │   └── mod.rs
│   ├── location/        # 出現位置・終了時位置機能
│   │   ├── model.rs     # LocationData (必要に応じて)
│   │   ├── state.rs     # LocationState
│   │   ├── view.rs      # LocationView (描画のみ)
│   │   ├── interaction.rs # LocationInteraction (インタラクション処理)
│   │   └── mod.rs
│   └── mod.rs
├── app.rs               # アプリケーションエントリーポイント (AppStateと各featureのみを使用)
└── main.rs
```

## Feature の粒度

### Feature の定義

「機能」とは、ユーザーが認識できる独立した操作単位です。

**重要な原則**: **画面に描画されているものは全てfeatureとして扱う**

**例**:
- **メイン画面**: マップとユーザー一覧を表示する機能
- **出現位置・終了時位置の管理**: ユーザーの開始地点と終了地点を設定・移動する機能
- **軌跡描画**: ユーザーの移動軌跡を描画する機能
- **ターン選択**: ターンを切り替える機能
- **ユーザー選択**: 操作対象のユーザーを選択する機能
- **デバッグビュー**: デバッグ情報を表示する機能

### Feature の分割基準

1. **データの独立性**: 機能が独自のデータ（Model）を持つ、または既存のModelの特定部分を操作する
2. **UIの独立性**: 機能が独自のUI要素（View）を持つ、または既存のViewの特定部分を描画する
3. **状態の独立性**: 機能が独自の状態（State）を持つ、または既存のStateの特定部分を管理する
4. **操作の独立性**: 機能が独立した操作フローを持つ

### Feature Model の必要性

**各featureにmodelが必要かどうかは、featureによる**

#### Modelが必要なfeature

以下の条件を満たすfeatureは、独自のModelを持つ：

1. **永続化可能なデータを持つ**: ファイルに保存したいデータがある
2. **機能固有のデータ構造を持つ**: 他のfeatureと独立したデータ構造
3. **複雑なデータ構造を持つ**: 単純な値ではなく、構造化されたデータ

**例**: `location` (出現位置・終了時位置), `route_drawing` (軌跡)

#### Modelが不要なfeature

以下の条件を満たすfeatureは、独自のModelを持たない：

1. **UI状態のみを持つ**: 選択中、ドラッグ中などの一時的な状態
2. **グローバルなModelで十分**: 既存のModel（`Game`, `Wave`など）で表現できる
3. **単純な状態のみ**: `bool`, `Option<usize>`などの単純な値のみ

**例**: `debug_view` (表示/非表示), `user_selection` (選択中のユーザー)

## グローバルなModelの更新

### 原則

**グローバルなModel（`Game`, `User`など）の更新は、各featureのStateのメソッドで行う**

### 実装パターン

#### パターン1: Feature固有のModelを更新する場合

```rust
// features/location/interaction.rs
impl LocationInteraction {
    // Model更新とState更新を同時に行う
    pub fn handle_click(
        model: &mut AppModel,
        state: &mut LocationState,
        wave_index: usize,
        user_id: usize,
        point: Point,
    ) {
        // Model更新: feature固有のModelを更新
        model.location_data.spawn_locations.insert(user_id, point);
        
        // State更新: 必要に応じてStateも更新
        // (この例ではStateの更新は不要)
    }
}

// features/location/view.rs
impl LocationView {
    pub fn show(
        model: &AppModel,
        state: &LocationState,
        // ... 描画処理
    ) {
        // ModelとStateを参照して描画
    }
    
    pub fn handle_interaction(
        model: &mut AppModel,
        state: &mut LocationState,
        response: &egui::Response,
    ) {
        if response.clicked() {
            // Interactionを実行
            LocationInteraction::handle_click(model, state, wave_index, user_id, point);
        }
    }
}
```

#### パターン2: グローバルなModelを更新する場合

```rust
// features/user_management/interaction.rs
impl UserManagementInteraction {
    // Model更新とState更新を同時に行う
    pub fn handle_name_update(
        model: &mut AppModel,
        state: &mut UserManagementState,
        user_id: usize,
        name: String,
    ) {
        // Model更新: グローバルなModelを更新
        if let Some(game) = &mut model.game {
            if let Some(user) = game.users.get_mut(user_id) {
                user.name = name;
            }
        }
        
        // State更新: 必要に応じてStateも更新
        // (この例ではStateの更新は不要)
    }
}
```

#### パターン3: 複数のModelを更新する場合（AppStateで調整）

```rust
// game/state.rs
impl AppState {
    // 複数のfeatureを連携させる場合
    pub fn handle_user_removed(&mut self, user_id: usize) {
        // Model更新: 複数のModelを更新
        if let Some(game) = &mut self.model.game {
            game.users.remove(user_id);
            
            for wave in &mut game.waves {
                wave.location_data.spawn_locations.remove(&user_id);
                wave.route_drawing_data.routes.retain(|r| r.user_id != user_id);
            }
        }
        
        // State更新: 必要に応じてStateも更新
        self.location_state.clear_dragging();
        self.route_drawing_state.clear_temp_points();
    }
}
```

#### パターン4: State更新のみの場合

```rust
// features/debug_view/interaction.rs
impl DebugViewInteraction {
    // State更新のみ（Model更新は不要）
    pub fn handle_toggle(state: &mut DebugViewState) {
        // State更新: UI状態のみを更新
        state.show_debug_view = !state.show_debug_view;
    }
}

// features/debug_view/view.rs
impl DebugView {
    pub fn show(state: &DebugViewState) {
        // Stateを参照して描画
    }
    
    pub fn handle_interaction(state: &mut DebugViewState, response: &egui::Response) {
        if response.clicked() {
            // Interactionを実行
            DebugViewInteraction::handle_toggle(state);
        }
    }
}
```

### 重要な原則

1. **データフローは `model > state > view > interaction > model更新 > state更新` で統一**
2. **Viewには必ず対になるinteraction.rsがある**
3. **Viewは描画のみを担当し、ModelやStateを更新しない**
4. **InteractionはModelとStateを更新する唯一の場所**
5. **Modelの更新は各featureのInteractionで行う**
6. **Stateの更新も各featureのInteractionで行う**
7. **AppStateはModelを保持するだけで、直接更新しない**
8. **複数のfeatureを連携させる必要がある場合は、AppStateで調整する**

## データフロー

### 統一されたデータフロー

```
Model → State → View → Interaction → Model更新 → State更新 → Model
```

### フレーム N: 描画とインタラクション

```
1. Model (データを参照)
   ↓
2. State (状態を参照)
   ↓
3. View (ModelとStateを参照して描画)
   ↓
4. Viewがインタラクションを検出
   ↓
5. Interactionを実行
   ├─ Model更新 (データを更新)
   └─ State更新 (状態を更新)
```

### フレーム N+1: 更新された描画

```
1. Model (更新されたデータを参照)
   ↓
2. State (更新された状態を参照)
   ↓
3. View (更新されたModelとStateを参照して描画 - 変更が反映される)
```

### 具体例

```rust
// フレーム N: ユーザーがクリック

// features/location/view.rs
impl LocationView {
    pub fn show(
        model: &AppModel,           // 1. Modelを参照
        state: &LocationState,      // 2. Stateを参照
        // ... 描画処理
    ) {
        // 3. Viewで描画
    }
    
    pub fn handle_interaction(
        model: &mut AppModel,
        state: &mut LocationState,
        response: &egui::Response,
    ) {
        if response.clicked() {     // 4. インタラクションを検出
            // 5. Interactionを実行
            LocationInteraction::handle_click(model, state, wave_index, user_id, point);
        }
    }
}

// features/location/interaction.rs
impl LocationInteraction {
    pub fn handle_click(
        model: &mut AppModel,       // 5. Model更新
        state: &mut LocationState,  // 5. State更新
        wave_index: usize,
        user_id: usize,
        point: Point,
    ) {
        // Model更新
        model.location_data.spawn_locations.insert(user_id, point);
        
        // State更新
        state.start_dragging(location);
    }
}

// フレーム N+1: 更新されたModelとStateを参照して描画
impl LocationView {
    pub fn show(
        model: &AppModel,           // 更新されたModelを参照
        state: &LocationState,      // 更新されたStateを参照
        // ... 描画処理（変更が反映される）
    ) {
    }
}
```

## 命名規則

- **Model**: 既存の命名規則に従う（例: `Game`, `Wave`, `LocationData`）
- **State**: `<FeatureName>State` (例: `LocationState`)
- **View**: `<FeatureName>View` (例: `LocationView`)
- **Interaction**: `<FeatureName>Interaction` (例: `LocationInteraction`)

## 禁止事項

### ❌ 禁止: AppStateから直接Modelを更新

```rust
// ❌ 悪い例
impl AppState {
    pub fn update_spawn_location(&mut self, user_id: usize, point: Point) {
        // AppStateから直接Modelを更新しない
    }
}
```

### ❌ 禁止: Viewから直接ModelやStateを更新

```rust
// ❌ 悪い例
impl LocationView {
    pub fn handle_interaction(&mut self, model: &mut AppModel, state: &mut LocationState) {
        // Viewから直接ModelやStateを更新しない
        model.location_data.spawn_locations.insert(user_id, point);
        state.start_dragging(location);
    }
}
```

**正しい例**: ViewはInteractionを実行するだけ

```rust
// ✅ 良い例
impl LocationView {
    pub fn handle_interaction(
        &mut self,
        model: &mut AppModel,
        state: &mut LocationState,
        response: &egui::Response,
    ) {
        if response.clicked() {
            // Interactionを実行
            LocationInteraction::handle_click(model, state, wave_index, user_id, point);
        }
    }
}
```

### ❌ 禁止: ModelからAppStateを参照（循環参照）

```rust
// ❌ 悪い例
pub struct Game {
    pub app_state: AppState,  // 循環参照になる
}
```

## App の役割

### app.rs の構成

`app.rs`は以下のみを使用します：

1. **AppState**: アプリケーション全体の状態
2. **各feature**: 画面に描画される全ての機能

```rust
// app.rs
impl eframe::App for AmusApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // AppStateと各featureのみを使用
        MainView::show(&mut self.state, ctx);
        LocationView::show(&mut self.state, ctx);
        // ...
    }
}
```

**重要**: 
- `app.rs`は`AppState`と各featureだけを使用する
- 画面に描画されているものは全てfeatureとして扱う
- `ui/`ディレクトリは不要（全て`features/`に統合）

## 参考

- eframe/eguiの状態管理: `eframe::App`トレイトを実装する構造体が状態を保持
- AppStateはeframe/eguiの機能ではなく、独自に定義した構造体

