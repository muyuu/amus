# アーキテクチャ改善ロードマップ

## 背景

現在の設計ではView層がegui（`ui: &mut Ui`）と直接結合しており、UIフレームワーク変更時に全Feature/View/Interactionを書き換える必要がある。また、AppStateが肥大化しており管理が困難になりつつある。

## 目標

1. **UIフレームワークの分離** - egui以外への変更を容易にする
2. **AppStateの分割** - ドメイン単位でファイル分割し、見通しを良くする
3. **Redux/Elm的なデータフロー** - 明確なActions層とState読み取りの分離

## 現状の課題

### 1. View層のegui依存

```rust
// 現状: eguiの型が直接渡されている
impl LocationView {
    pub fn render(state: &AppState, ui: &mut Ui) -> LocationViewResult { ... }
}

impl LocationInteraction {
    pub fn handle(state: &mut AppState, result: &LocationViewResult, response: &Response, ui: &mut Ui) { ... }
}
```

### 2. AppStateの肥大化

`app_state.rs`が600行以上になり、以下が混在:
- ゲーム管理
- プレイヤー操作
- UI状態管理
- ドラッグ&ドロップ
- ウェーブ・ルート管理
- ストレージ

### 3. 不明確なデータフロー

ViewがAppStateを直接参照し、Interactionが更新する現在の設計では:
- 読み取りと書き込みの境界が曖昧
- テストが困難

## 改善方針

### Phase 1: Actionsのファイル分割

**目的**: AppStateの見通し改善（構造変更なし）

```
src/state/
├── mod.rs
├── app_state.rs      # 構造体定義 + 基本メソッドのみ
├── app_data.rs
├── actions/          # アクション群をドメイン単位で分割
│   ├── mod.rs
│   ├── game.rs       # start_new_game, reset_game, create_game_from_setup
│   ├── player.rs     # toggle_player_state, update_player_name, update_player_color
│   ├── location.rs   # add_location, remove_location, locations
│   ├── wave.rs       # select_wave, current_wave, routes, push_route
│   ├── ui.rs         # toggle_debug_view, set_erase_mode, selected_player_id
│   └── drag.rs       # dragging_player_id, dragging_location
└── storage.rs
```

**実装方法**: Rustでは同じ型に複数ファイルで`impl`できる

```rust
// src/state/actions/player.rs
use super::AppState;

impl AppState {
    pub fn toggle_player_state(&self, player_id: PlayerId) { ... }
    pub fn update_player_name(&self, id: PlayerId, name: String) { ... }
}
```

### Phase 2: Actions型の明示化

**目的**: ViewにActionsインターフェースを渡し、依存を明確化

```rust
// src/state/actions.rs
pub struct Actions<'a> {
    state: &'a AppState,
}

impl<'a> Actions<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    // Player actions
    pub fn select_player(&self, id: Option<PlayerId>) {
        self.state.set_selected_player_id(id);
    }

    pub fn toggle_player_state(&self, id: PlayerId) {
        self.state.toggle_player_state(id);
    }

    // Location actions
    pub fn add_location(&self, location_type: LocationType, player_id: PlayerId, point: Point) {
        self.state.add_location(location_type, player_id, point);
    }

    // ... 他のアクション
}
```

### Phase 3: State読み取りの分離（Sliceパターン）

**目的**: 読み取り専用アクセスを明示化

```rust
// ドメイン単位のSlice
pub struct GameSlice<'a> {
    data: Ref<'a, AppData>,
}

impl<'a> GameSlice<'a> {
    pub fn game(&self) -> Option<&Game> {
        self.data.game.as_ref()
    }

    pub fn area(&self) -> Option<&Area> {
        self.data.game.as_ref().map(|g| &g.area)
    }
}

pub struct PlayerSlice<'a> {
    data: Ref<'a, AppData>,
}

impl<'a> PlayerSlice<'a> {
    pub fn players(&self) -> Option<&Vec<Player>> {
        self.data.game.as_ref().map(|g| &g.players)
    }

    pub fn selected_player_id(&self) -> Option<PlayerId> {
        self.data.selected_player_id
    }
}
```

### Phase 4: View層の分離

**目的**: UIフレームワーク変更を容易にする

```rust
// 最終形: Viewは不変State + Actions + UI を受け取る
impl LocationView {
    pub fn render(
        game: &GameSlice,
        player: &PlayerSlice,
        actions: &Actions,
        ui: &mut Ui,  // ← egui依存はここだけ
    ) -> LocationViewResult {
        // ...
    }
}
```

将来的にはAdapter層を導入し、`ui: &mut Ui`をトレイト化することでegui完全分離も可能。

## パフォーマンス考慮

Rustの所有権システムにより、以下の懸念は問題にならない:

| 懸念 | 実態 |
|------|------|
| 無駄なコピー | 参照（`&`）で渡すのでコピーなし |
| 全体ロック | `RefCell`は借用単位 |
| 毎フレーム読み取り | 参照渡しで実質ゼロコスト |

**改善の余地**: 現在の`clone()`を減らし、参照ベースのアクセスを増やす

```rust
// Before: clone()コスト
pub fn game(&self) -> Option<Game> {
    self.data.borrow().game.clone()
}

// After: 参照を返す
pub fn with_game<R>(&self, f: impl FnOnce(Option<&Game>) -> R) -> R {
    let data = self.data.borrow();
    f(data.game.as_ref())
}
```

## 実装順序

1. **Phase 1** (低リスク): ファイル分割のみ。動作に影響なし ✅ 完了
2. **Phase 2** (中リスク): Actions型導入。Viewの引数を段階的に変更 ✅ 完了
3. **Phase 3** (中リスク): Slice導入。読み取りアクセスを整理 ✅ 完了
4. **Phase 4** (高リスク): View層分離。全Featureを書き換え

各Phaseは独立して実施可能。Phase 1だけでも大きな改善になる。

## 現在の状態

### Phase 3 完了時点の構造

```
src/state/
├── mod.rs
├── app_state.rs      # 構造体定義 + slices()メソッド
├── app_data.rs
├── actions/          # 状態変更用（Phase 2で導入）
│   ├── mod.rs
│   ├── game.rs
│   ├── player.rs
│   ├── location.rs
│   └── ...
└── slices/           # 読み取り専用アクセス（Phase 3で導入）
    ├── mod.rs        # Slices構造体
    ├── game.rs       # GameSlice
    ├── player.rs     # PlayerSlice
    ├── wave.rs       # WaveSlice
    ├── ui.rs         # UiSlice
    └── setup.rs      # SetupSlice
```

### 使用例

```rust
// Feature層での使用パターン
impl PlayerListFeature {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        // ViewにはSlices（読み取り専用）を渡す
        let slices = state.slices();
        let player_actions = PlayerListView::render(&slices, ui);

        // Actionsで状態を更新
        let mut actions = Actions::new(state);
        for action in player_actions {
            actions.handle_player(action);
        }
    }
}

// View層での使用パターン
impl PlayerListView {
    pub fn render(slices: &Slices<'_>, ui: &mut Ui) -> Vec<PlayerAction> {
        let players = slices.player().players();
        let is_selected = slices.player().is_selected(player_id);
        let is_dragging = slices.ui().is_dragging_player(player_id);
        // ...
    }
}
```

### 次のステップ（Phase 4）

残りのFeature/Viewを順次Sliceパターンに移行する。

## 関連ドキュメント

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 現在のアーキテクチャ
- [STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md) - 現在の状態管理パターン
