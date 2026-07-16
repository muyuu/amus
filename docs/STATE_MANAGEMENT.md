# 状態管理パターン

## 概要

このドキュメントでは、`AppData`と`AppState`の関係性、Model層の役割、設計意図、および実装パターンについて説明します。

## AppData と AppState の関係性

### 構造

`AppState`が`AppData`を値で所有する。内部可変性（`RefCell`等）は使わず、借用は`&self`（読み取り）/ `&mut self`（書き込み）で表現する。

```rust
pub struct AppState {
    pub(super) data: AppData,
}

pub struct AppData {
    pub(in crate::state) current_wave_index: usize,
    pub(in crate::state) dragging_location: Option<DraggingLocation>,
    pub(in crate::state) game: Option<Game>,
    // ... その他のフィールド（すべて pub(in crate::state)）
}
```

`AppData`のフィールドは`pub(in crate::state)`に絞られており、`state`モジュールの外からは直接読み書きできない。読み取りは`Slices`、書き込みは`AppState`メソッド / `Actions`を経由する規約をコンパイラが強制する。

### 読み取りアクセス（Slices）

ViewはSlicesを通じて読み取り専用でデータにアクセスする。

```rust
// AppStateからSlicesを取得
let slices = state.slices();

// 各ドメインのSliceを使用
let players = slices.player().players();
let is_selected = slices.player().is_selected(player_id);
let is_dragging = slices.ui().is_dragging_player(player_id);
let current_wave = slices.wave().current_wave();
```

### 書き込みアクセス（Actions）

状態の変更はActionsを通じて行う。

```rust
let mut actions = Actions::new(state);
actions.handle_player(PlayerAction::Select(player_id));
```

### 役割分担

#### Model: ドメインデータとシンプルなヘルパー

**責務**:
- ドメインデータの構造を定義する
- シリアライズ可能なデータ構造（`Serialize`/`Deserialize`）
- シンプルなヘルパーメソッド（コンストラクタ、ゲッター、判定メソッド）
- UIフレームワーク（egui）に依存しない

**許可されるメソッド**:
- ✅ `new()` - コンストラクタ
- ✅ `is_empty()`, `is_dead()` - 純粋な判定（状態を変更しない）
- ✅ `get_xxx()` - ゲッター（状態を変更しない）
- ✅ `pub(crate) fn xxx_mut()` - 可変参照の取得（state モジュールからのみアクセス可能）

**禁止されるメソッド**:
- ❌ `add_xxx()`, `remove_xxx()` - コレクション操作
- ❌ `toggle_xxx()`, `next()` - 状態遷移
- ❌ `update_xxx()` - 状態更新
- ❌ ビジネスロジック全般

永続属性（id/role/color/name）と、1 ゲーム中の進行状態（生死・ボタン・サボタージュ解決）は
別構造体に分ける。進行状態は `PlayerProgress` にまとめ、`Player` は `progress` 1 フィールドで持つ。

```rust
// Model: データ構造とシンプルなヘルパー
pub struct Player {
    pub id: PlayerId,
    pub role: Role,
    pub color: Color,
    pub name: String,
    pub progress: PlayerProgress, // 1 ゲーム中の進行状態
}

pub struct PlayerProgress {
    pub state: PlayerState,
    pub death: Option<usize>,
    pub done_button: bool,
    pub resolved: HashSet<Sabotage>, // 解決済みのサボタージュ種別
}

impl Player {
    // ✅ コンストラクタはOK
    pub fn new(role: Role, color: Color, name: String) -> Self {
        Self { /* progress: PlayerProgress::default() */ }
    }

    // ✅ 純粋な判定メソッドはOK（進行状態は progress へ委譲）
    pub fn is_dead(&self) -> bool {
        self.progress.state == PlayerState::Killed
    }

    // ❌ 状態遷移はNG（AppStateに移動）
    // pub fn kill(&mut self) { ... }
}
```

#### AppData: 純粋なデータコンテナ

**責務**:
- アプリケーションの実際のデータを保持する
- ビジネスロジックを持たない（データ構造のみ）

**設計意図**:
- データの構造を明確にする
- テスト時にモックデータを作りやすい
- シリアライズ/デシリアライズが容易
- データの全体像を把握しやすい

```rust
// AppDataは純粋なデータ構造（フィールドのみ、メソッドはない）
pub struct AppData {
    pub(in crate::state) game: Option<Game>,
    pub(in crate::state) selected_player_id: Option<PlayerId>,
    pub(in crate::state) erase_mode: bool,
    // ...
}
```

#### AppState: データへのアクセス制御とビジネスロジック

**責務**:
- `AppData`を所有し、読み取り（`&self` / `Slices`）と書き込み（`&mut self`）を提供する
- **全てのビジネスロジック**を実装する
- データの一貫性を保つ（バリデーションやロジックを一箇所に集約）

**設計意図**:
- データへの直接アクセスを制限し、カプセル化を実現
- 操作の意図を明確にする（メソッド名で何をするか分かる）
- 読み取りと書き込みを型（`&self`/`&mut self`、`Slices`/`Actions`）で分離し、Viewからの不用意な変更を防ぐ

肥大化を避けるため、`AppState`の`impl`はドメイン単位で`src/state/app_state/`配下に分割されている（`game.rs` / `player.rs` / `wave.rs` / `ui.rs` / `drag_drop.rs` / `storage.rs`）。

```rust
impl AppState {
    // Slicesを通じた読み取り専用アクセス
    pub fn slices(&self) -> Slices<'_> {
        Slices::new(&self.data)
    }

    // 単純なセッター（&mut self）
    pub fn set_selected_player_id(&mut self, player_id: Option<PlayerId>) {
        self.data.selected_player_id = player_id;
    }

    // ビジネスロジックを含む操作（&mut self）
    pub fn toggle_player_state(&mut self, player_id: PlayerId) {
        if let Some(game) = &mut self.data.game {
            if let Some(player) = game.players.iter_mut().find(|p| p.id == player_id) {
                player.state = match player.state {
                    PlayerState::Alive => PlayerState::Killed,
                    PlayerState::Killed => PlayerState::Ejected,
                    PlayerState::Ejected => PlayerState::Alive,
                };
            }
        }
    }
}
```

## 設計上の利点

### 1. 所有権の明確化

**問題**: 複数の場所から直接データを変更すると、誰がデータを所有しているか不明確になる

**解決**: `AppState`が`AppData`を所有し、操作はメソッド経由に限定する

```rust
// ❌ 悪い例: 直接AppDataを触る（pub(in crate::state) によりそもそもコンパイルが通らない）
app_data.selected_player_id = Some(player_id);

// ✅ 良い例: AppState経由で操作
state.set_selected_player_id(Some(player_id));
```

### 2. &mut を限定的にする

**問題**: データへの可変参照が広く取れると、予期しない変更が起きやすい

**解決**: 変更は`&mut self`メソッドに閉じ込め、読み取りだけが必要な箇所には`&self` / `Slices`を使う。`AppData`のフィールドが`pub(in crate::state)`なので、外部は意図を表すメソッドを通すしかない。

```rust
// ✅ 推奨: 専用メソッドで意図を明確にする
state.set_selected_player_id(Some(player_id));
state.set_erase_mode(true);
state.select_wave(3);
```

### 3. データの一貫性

**問題**: 複数のフィールドを同時に変更する必要がある場合、途中で不整合な状態になりうる

**解決**: `AppState`のメソッドでまとまった操作を提供する。「読み取って収集 → まとめて更新」を一つのメソッド内で行う。

```rust
impl AppState {
    // 色の重複を避けつつ更新する（収集 → 更新を一つのメソッドにまとめる）
    pub fn try_update_player_color(
        &mut self,
        player_id: PlayerId,
        new_color: Color,
    ) -> Result<(), String> {
        // 1. 衝突相手を収集
        let conflicting = self
            .data
            .game
            .as_ref()
            .and_then(|g| g.players.iter().find(|p| p.color == new_color && p.id != player_id))
            .map(|p| p.id);

        // 2. まとめて更新（一貫性を保つ）
        if let Some(game) = &mut self.data.game {
            // ... 必要なら色を交換するなど
        }
        Ok(())
    }
}
```

### 4. Modelの可変メソッドを制限

**問題**: Modelに`pub`な可変メソッドがあると、Viewから直接変更できてしまう

**解決**: Modelの可変メソッドは`pub(crate)`にし、state モジュールからのみアクセス可能にする

```rust
// ✅ Model 側: 可変メソッドは pub(crate)、読み取りは pub
impl Game {
    pub(crate) fn get_wave_mut(&mut self, index: usize) -> Option<&mut Wave> {
        self.waves.get_mut(index)
    }

    pub fn get_wave(&self, index: usize) -> Option<&Wave> {
        self.waves.get(index)
    }
}

// ✅ AppState 側: 状態変更を公開する
impl AppState {
    pub fn add_wave(&mut self) {
        if let Some(game) = &mut self.data.game {
            game.waves.push(Wave::default());
        }
        self.data.current_wave_index += 1;
    }
}
```

**ガイドライン**:
- Modelの可変メソッド（`&mut self`）は`pub(crate)`にする
- 状態を変更する操作は全て`AppState`に実装する
- Modelからは純粋な判定メソッド（`is_dead()`など）のみ`pub`で公開する

## 実装パターン

### パターン1: 単純な読み取り

読み取りは`&self`メソッド、または`Slices`で提供する。

```rust
impl AppState {
    pub fn selected_player_id(&self) -> Option<PlayerId> {
        self.data.selected_player_id
    }

    pub fn erase_mode(&self) -> bool {
        self.data.erase_mode
    }
}
```

### パターン2: 単純な書き込み

単一のフィールドを更新する場合、`&mut self`のセッターを作る。

```rust
impl AppState {
    pub fn set_selected_player_id(&mut self, player_id: Option<PlayerId>) {
        self.data.selected_player_id = player_id;
    }

    pub fn set_erase_mode(&mut self, enabled: bool) {
        self.data.erase_mode = enabled;
    }

    pub fn select_wave(&mut self, index: usize) {
        self.data.current_wave_index = index;
    }
}
```

### パターン3: 複雑な操作（収集 → 更新）

複数フィールドを読んでから更新する場合、先に必要な値を収集してから更新する。`self`は単一の所有者なので、収集と更新を同じメソッド内で素直に書ける。

```rust
impl AppState {
    pub fn update_player_name(
        &mut self,
        player_id: PlayerId,
        new_name: String,
    ) -> Result<(), String> {
        // バリデーション
        if new_name.is_empty() {
            return Err("名前は空にできません".to_string());
        }

        // 重複チェック（読み取り）
        let duplicated = self
            .data
            .game
            .as_ref()
            .map(|g| g.players.iter().any(|p| p.name == new_name && p.id != player_id))
            .unwrap_or(false);
        if duplicated {
            return Err("名前が重複しています".to_string());
        }

        // 更新（書き込み）
        let game = self.data.game.as_mut().ok_or("ゲームが開始されていません")?;
        let player = game
            .players
            .iter_mut()
            .find(|p| p.id == player_id)
            .ok_or("プレイヤーが見つかりません")?;
        player.name = new_name;
        Ok(())
    }
}
```

## Model設計のベストプラクティス

### ✅ DO: シンプルなヘルパーメソッド

コンストラクタ、ゲッター、純粋な判定メソッドは実装する。

```rust
impl Player {
    pub fn new(role: Role, color: Color, name: String) -> Self {
        Self {
            id: PlayerId(Uuid::new_v4()),
            role,
            color,
            name,
            progress: PlayerProgress::default(),
        }
    }

    pub fn is_dead(&self) -> bool {
        self.progress.state == PlayerState::Killed
    }

    pub fn is_ejected(&self) -> bool {
        self.progress.state == PlayerState::Ejected
    }
}
```

### ✅ DO: 可変メソッドはpub(crate)にする

外部から状態を変更できないようにする。

```rust
impl Game {
    pub(crate) fn get_wave_mut(&mut self, index: usize) -> Option<&mut Wave> {
        self.waves.get_mut(index)
    }

    pub fn get_wave(&self, index: usize) -> Option<&Wave> {
        self.waves.get(index)
    }
}
```

### ❌ DON'T: Model内で状態変更ロジックを実装

状態を変更する操作は全てAppStateに実装する。

```rust
// ❌ 悪い例: Model内で状態変更
impl Game {
    pub fn add_wave(&mut self) {
        self.waves.push(Wave::default());
    }
}

impl PlayerState {
    pub fn next(&self) -> Self { /* 状態遷移 */ }
}

// ✅ 良い例: AppStateで状態変更
impl AppState {
    pub fn add_wave(&mut self) {
        if let Some(game) = &mut self.data.game {
            game.waves.push(Wave::default());
        }
        self.data.current_wave_index += 1;
    }
}
```

### ❌ DON'T: Modelに複雑なロジックを持たせる

ビジネスロジックは全てAppStateに集約する。

```rust
// ❌ 悪い例: Model内で複雑なロジック（重複スキップなど）
impl Route {
    pub fn add_point(&mut self, point: Point) { /* 重複チェック等のロジック */ }
}

// ✅ 良い例: AppStateで複雑なロジック
impl AppState {
    pub fn add_route_point(&mut self, player_id: PlayerId, point: Point) {
        let index = self.data.current_wave_index;
        if let Some(game) = &mut self.data.game {
            if let Some(wave) = game.waves.get_mut(index) {
                // 直前のポイントと同じならスキップ、などのロジックをここに実装
            }
        }
    }
}
```

## ベストプラクティス

### ✅ DO: 専用メソッドを作る

意図を明確にし、コードの可読性を高める。

```rust
impl AppState {
    pub fn toggle_erase_mode(&mut self) {
        self.data.erase_mode = !self.data.erase_mode;
    }

    pub fn clear_selection(&mut self) {
        self.data.selected_player_id = None;
        self.data.dragging_player_id = None;
    }
}
```

### ✅ DO: 読み取りと書き込みの境界を意識する

読み取りだけが必要な箇所には`&self` / `Slices`を使い、`&mut self`は実際に変更する操作に限定する。Viewには`Slices`しか渡さない。

### ✅ DO: エラーハンドリングを適切に行う

操作が失敗する可能性がある場合は`Result`を返す。

```rust
impl AppState {
    pub fn update_player_name(
        &mut self,
        player_id: PlayerId,
        name: String,
    ) -> Result<(), String> {
        if name.is_empty() {
            return Err("名前は空にできません".to_string());
        }

        let game = self.data.game.as_mut().ok_or("ゲームが開始されていません")?;
        let player = game
            .players
            .iter_mut()
            .find(|p| p.id == player_id)
            .ok_or("プレイヤーが見つかりません")?;
        player.name = name;
        Ok(())
    }
}
```

### ❌ DON'T: Viewから直接Stateを変更する

Viewはデータを直接変更せず、操作結果（Action 値）を返す。状態変更はFeatureが`Actions`経由で行う。

```rust
// ❌ 悪い例（View内で直接変更）
impl LocationView {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        if ui.button("Select").clicked() {
            state.set_selected_player_id(Some(player_id)); // NG
        }
    }
}

// ✅ 良い例: View は Slices で読み、結果を返す
impl LocationView {
    pub fn render(slices: &Slices<'_>, ui: &mut Ui) -> LocationResult {
        let mut result = LocationResult::default();
        if ui.button("Select").clicked() {
            result.clicked_player = Some(player_id);
        }
        result
    }
}

// Feature 側で Actions に変換して反映する
let mut actions = Actions::new(state);
actions.handle_player(PlayerAction::Select(player_id));
```

## まとめ

### 全体像

```
Model (データ構造 + シンプルなヘルパー)
    ↓ 保持
AppData (純粋なデータコンテナ。フィールドは pub(in crate::state))
    ↓ 所有
AppState (アクセス制御 + ビジネスロジック。AppData を値で保持)
    ├── slices() → Slices（読み取り専用）→ View
    └── Actions（書き込み）→ 状態更新
```

### データフロー

```
Feature層:
    state.slices() → View::render(&slices, ui) → Vec<Action>
                                                    ↓
    Actions::new(state) ← actions.handle_xxx(action)
```

### 設計原則

1. **Model層の責務**:
   - データ構造の定義（シリアライズ可能、egui 非依存）
   - コンストラクタ、ゲッター、純粋な判定メソッド
   - 可変メソッドは`pub(crate)`（外部から状態変更できない）
   - **禁止**: 状態変更ロジック、ビジネスロジック

2. **AppData層の責務**:
   - アプリケーションデータの保持
   - フィールドは`pub(in crate::state)`（外部から直接触れない）
   - **禁止**: メソッド実装（純粋なデータ構造のみ）

3. **AppState層の責務**:
   - `AppData`を所有し、読み取り（`&self`/`Slices`）と書き込み（`&mut self`/`Actions`）を提供
   - **全ての**ビジネスロジックの実装
   - データの一貫性の担保

### 達成される価値

1. **所有権の明確化**: データは`AppState`が値で所有・管理する
2. **&mut の限定**: 変更を`&mut self`メソッドに閉じ込める
3. **カプセル化**: フィールドの可視性とメソッド経由のアクセスで直接変更を防ぐ
4. **一貫性**: 複雑な操作をメソッドにまとめ、データの整合性を保つ
5. **可読性**: 操作の意図がメソッド名で明確になる
6. **保守性**: 変更の影響範囲を限定し、リファクタリングを容易にする

### 実装時のチェックリスト

- [ ] Modelに状態変更ロジックを実装していないか？
- [ ] Modelの可変メソッドは`pub(crate)`になっているか？
- [ ] ビジネスロジックは全て`AppState`に実装されているか？
- [ ] Viewから直接`AppData`や`Model`の状態を変更していないか？（`Slices`読み / `Actions`書き になっているか）
- [ ] 複雑な操作は専用メソッドに切り出しているか？
