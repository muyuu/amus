# 状態管理パターン

## 概要

このドキュメントでは、`AppData`と`AppState`の関係性、Model層の役割、設計意図、および実装パターンについて詳しく説明します。

## AppData と AppState の関係性

### 構造

```rust
pub struct AppState {
    data: AppData,
}

pub struct AppData {
    pub current_wave_index: usize,
    pub dragging_location: Option<DraggingLocation>,
    pub game: Option<Game>,
    // ... その他のフィールド
}
```

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

**許可されるメソッド**:
- ✅ `new()` - コンストラクタ
- ✅ `is_empty()`, `is_dead()` - 純粋な判定（状態を変更しない）
- ✅ `get_xxx()` - ゲッター（状態を変更しない）
- ✅ `pub(crate) fn xxx_mut()` - 可変参照の取得（AppStateからのみアクセス可能）

**禁止されるメソッド**:
- ❌ `add_xxx()`, `remove_xxx()` - コレクション操作
- ❌ `toggle_xxx()`, `next()` - 状態遷移
- ❌ `update_xxx()` - 状態更新
- ❌ ビジネスロジック全般

```rust
// Model: データ構造とシンプルなヘルパー
pub struct Player {
    pub id: PlayerId,
    pub state: PlayerState,
    pub name: String,
    // ...
}

impl Player {
    // ✅ コンストラクタはOK
    pub fn new(role: Role, color: Color, name: String) -> Self {
        Self { /* ... */ }
    }

    // ✅ 純粋な判定メソッドはOK
    pub fn is_dead(&self) -> bool {
        self.state == PlayerState::Killed
    }

    // ❌ 状態遷移はNG（AppStateに移動）
    // pub fn kill(&mut self) { ... }
}
```

#### AppData: 純粋なデータコンテナ

**責務**:
- アプリケーションの実際のデータを保持する
- ビジネスロジックを持たない（データ構造のみ）
- シンプルな構造体として定義される

**設計意図**:
- データの構造を明確にする
- テスト時にモックデータを作りやすい
- シリアライズ/デシリアライズが容易
- データの全体像を把握しやすい

```rust
// AppDataは純粋なデータ構造
pub struct AppData {
    pub game: Option<Game>,
    pub selected_player_id: Option<PlayerId>,
    pub erase_mode: bool,
    // ... フィールドのみ、メソッドはない
}
```

#### AppState: データへのアクセス制御とビジネスロジック

**責務**:
- `AppData`への参照（不変・可変）を提供する
- データ操作のためのメソッドを提供する
- **全てのビジネスロジック**を実装する
- データの所有権を明確にする
- `mut`アクセスを限定的にする

**設計意図**:
- データへの直接アクセスを制限し、カプセル化を実現
- 操作の意図を明確にする（メソッド名で何をするか分かる）
- データの一貫性を保つ（バリデーションやロジックを一箇所に集約）
- 借用ルールの管理を簡素化（`RefCell`で内部可変性を提供）

```rust
impl AppState {
    // Slicesを通じた読み取り専用アクセス
    pub fn slices(&self) -> Slices<'_> {
        Slices::new(&self.data)
    }

    // 不変参照でデータを取得
    pub fn game(&self) -> Option<&Game> {
        self.data.game.as_ref()
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

**解決**: `AppState`経由でのみデータにアクセスすることで、所有権が明確になる

```rust
// ❌ 悪い例: 直接AppDataを触る
let mut app_data = get_app_data_somehow();
app_data.selected_player_id = Some(player_id);  // 誰がデータを管理しているか不明確

// ✅ 良い例: AppState経由で操作
state.select_player(Some(player_id));  // AppStateが所有権を持つことが明確
```

### 2. mutを限定的にする

**問題**: データへの可変参照が多いと、予期しない変更が起きやすい

**解決**: `AppState`のメソッドで必要最小限の`mut`アクセスを提供

```rust
impl AppState {
    // 必要な時だけ可変参照を返す
    pub fn data_mut(&self) -> RefMut<'_, AppData> {
        self.data.borrow_mut()
    }

    // 通常は不変参照で済む操作を提供
    pub fn current_wave(&self) -> Option<Wave> {
        let data = self.data.borrow();
        data.game.as_ref()?.waves.get(data.current_wave_index).cloned()
    }
}
```

**ベストプラクティス**:
- `data_mut()`は最終手段として使う
- できる限り専用メソッドを作る（意図が明確になる）
- 可変参照のスコープを最小限に保つ

```rust
// ❌ 避けるべき: data_mut()の長期保持
let mut data = state.data_mut();
data.selected_player_id = Some(player_id);
data.erase_mode = true;
data.current_wave_index = 3;
// ... 多くの変更

// ✅ 推奨: 専用メソッドで意図を明確にする
state.select_player(Some(player_id));
state.set_erase_mode(true);
state.select_wave(3);
```

### 3. データの一貫性

**問題**: 複数のフィールドを同時に変更する必要がある場合、途中で不整合な状態になる可能性がある

**解決**: `AppState`のメソッドでアトミックな操作を提供

```rust
impl AppState {
    // 複数のデータを一貫性を保って更新
    pub fn force_update_player_color(&self, player_id: PlayerId, new_color: Color) -> Result<(), String> {
        // 1. 必要なデータを事前に収集（不変借用）
        let other_player_with_color = {
            let data = self.data.borrow();
            if let Some(game) = &data.game {
                game.players.iter()
                    .find(|p| p.color == new_color && p.id != player_id)
                    .map(|p| (p.id, p.color.clone()))
            } else {
                None
            }
        };

        // 2. 可変借用で一括更新（データの一貫性を保つ）
        let mut data = self.data.borrow_mut();
        if let Some(game) = &mut data.game {
            // 色の交換処理
            if let Some((other_id, old_color)) = other_player_with_color {
                // ... 両方のプレイヤーの色を同時に更新
            }
        }
        Ok(())
    }
}
```

### 4. Modelの可変メソッドを制限

**問題**: Modelに`pub`な可変メソッドがあると、Viewから直接変更できてしまう

```rust
// ❌ 問題のあるModel
impl Game {
    pub fn add_wave(&mut self) {
        self.waves.push(Wave::default());
    }
}

// ❌ Viewから直接変更できてしまう
let mut game = state.game().unwrap();
game.add_wave();  // AppStateを経由せずに変更
```

**解決**: Modelの可変メソッドは`pub(crate)`にし、AppStateからのみアクセス可能にする

```rust
// ✅ 改善されたModel
impl Game {
    // pub(crate)でクレート内（state/モジュール）からのみアクセス可能
    pub(crate) fn get_wave_mut(&mut self, index: usize) -> Option<&mut Wave> {
        self.waves.get_mut(index)
    }
}

// ✅ AppStateで操作を公開
impl AppState {
    pub fn add_wave(&self) {
        let mut data = self.data.borrow_mut();
        if let Some(game) = &mut data.game {
            game.waves.push(Wave::default());
        }
        data.current_wave_index += 1;
    }
}
```

**ガイドライン**:
- Modelの可変メソッド（`&mut self`）は`pub(crate)`にする
- 状態を変更する操作は全て`AppState`に実装する
- Modelからは純粋な判定メソッド（`is_dead()`など）のみ`pub`で公開

### 5. RefCell による借用ルールの管理

**問題**: Rustの借用ルール（可変参照と不変参照は同時に存在できない）が制約になる

**解決**: `RefCell`で内部可変性を提供し、実行時に借用をチェック

```rust
pub struct AppState {
    data: RefCell<AppData>,  // RefCellで内部可変性を提供
}

impl AppState {
    // &self（不変参照）でも内部データを変更できる
    pub fn select_player(&self, player_id: Option<PlayerId>) {
        self.data.borrow_mut().selected_player_id = player_id;
    }
}
```

**注意点**:
- `borrow()`と`borrow_mut()`を同時に保持するとパニックする
- スコープを適切に管理し、借用を早めに解放する

```rust
// ❌ 危険: 同時借用でパニック
let data = state.data.borrow();
let mut data_mut = state.data.borrow_mut();  // パニック！

// ✅ 安全: スコープを分ける
{
    let data = state.data.borrow();
    let value = data.some_field.clone();
}  // dataのスコープ終了
{
    let mut data_mut = state.data.borrow_mut();
    data_mut.some_field = value;
}  // data_mutのスコープ終了
```

## 実装パターン

### パターン1: 単純な読み取り

データを読み取るだけの場合、不変参照を返すメソッドを作る。

```rust
impl AppState {
    pub fn selected_player_id(&self) -> Option<PlayerId> {
        self.data.borrow().selected_player_id
    }

    pub fn erase_mode(&self) -> bool {
        self.data.borrow().erase_mode
    }

    pub fn game(&self) -> Option<Game> {
        self.data.borrow().game.clone()
    }
}
```

### パターン2: 単純な書き込み

単一のフィールドを更新する場合、専用のセッターメソッドを作る。

```rust
impl AppState {
    pub fn set_selected_player_id(&self, player_id: Option<PlayerId>) {
        self.data.borrow_mut().selected_player_id = player_id;
    }

    pub fn set_erase_mode(&self, enabled: bool) {
        self.data.borrow_mut().erase_mode = enabled;
    }

    pub fn select_wave(&self, index: usize) {
        self.data.borrow_mut().current_wave_index = index;
    }
}
```

### パターン3: 複雑な操作（データ収集 → 更新）

複数のフィールドを読み書きする場合、借用を分離する。

```rust
impl AppState {
    pub fn update_with_validation(&self, player_id: PlayerId, new_name: String) -> Result<(), String> {
        // ステップ1: データ収集（不変借用）
        let name_exists = {
            let data = self.data.borrow();
            if let Some(game) = &data.game {
                game.players.iter().any(|p| p.name == new_name && p.id != player_id)
            } else {
                false
            }
        };  // dataのスコープ終了

        // バリデーション
        if name_exists {
            return Err("名前が重複しています".to_string());
        }

        // ステップ2: データ更新（可変借用）
        let mut data = self.data.borrow_mut();
        if let Some(game) = &mut data.game {
            if let Some(player) = game.players.iter_mut().find(|p| p.id == player_id) {
                player.name = new_name;
            }
        }

        Ok(())
    }
}
```

### パターン4: 直接アクセスが必要な場合（data_mut）

複雑な一時的処理や、専用メソッドを作るほどでない場合は`data_mut()`を使う。

```rust
impl AppState {
    pub fn data_mut(&self) -> RefMut<'_, AppData> {
        self.data.borrow_mut()
    }
}

// 使用例
{
    let mut data = state.data_mut();
    data.dragging_location = Some(location);
    data.selected_player_id = Some(player_id);
}  // スコープを明示的に閉じる
```

**使用時の注意**:
- スコープを最小限に保つ（ブロック`{}`で囲む）
- 複雑なロジックは専用メソッドに移動することを検討
- 頻繁に使う操作は専用メソッドを作る

## Model設計のベストプラクティス

### ✅ DO: シンプルなヘルパーメソッド

コンストラクタ、ゲッター、純粋な判定メソッドは実装する。

```rust
impl Player {
    // ✅ コンストラクタ
    pub fn new(role: Role, color: Color, name: String) -> Self {
        Self {
            id: PlayerId(Uuid::new_v4()),
            role,
            color,
            name,
            state: PlayerState::Alive,
            // ...
        }
    }

    // ✅ 純粋な判定（状態を変更しない）
    pub fn is_dead(&self) -> bool {
        self.state == PlayerState::Killed
    }

    pub fn is_ejected(&self) -> bool {
        self.state == PlayerState::Ejected
    }

    // ✅ ゲッター
    pub fn name(&self) -> &str {
        &self.name
    }
}
```

### ✅ DO: 可変メソッドはpub(crate)にする

外部から状態を変更できないようにする。

```rust
impl Game {
    // ✅ pub(crate)でクレート内からのみアクセス可能
    pub(crate) fn get_wave_mut(&mut self, index: usize) -> Option<&mut Wave> {
        self.waves.get_mut(index)
    }

    // ✅ 読み取り専用はpub
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
        self.current_wave_index += 1;  // 状態変更
    }
}

impl PlayerState {
    pub fn next(&self) -> Self {
        match self {
            PlayerState::Alive => PlayerState::Killed,
            PlayerState::Killed => PlayerState::Ejected,
            PlayerState::Ejected => PlayerState::Alive,
        }
    }
}

// ✅ 良い例: AppStateで状態変更
impl AppState {
    pub fn add_wave(&self) {
        let mut data = self.data.borrow_mut();
        if let Some(game) = &mut data.game {
            game.waves.push(Wave::default());
        }
        data.current_wave_index += 1;
    }

    pub fn toggle_player_state(&self, player_id: PlayerId) {
        let mut data = self.data.borrow_mut();
        if let Some(game) = &mut data.game {
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

### ❌ DON'T: Modelに複雑なロジックを持たせる

ビジネスロジックは全てAppStateに集約する。

```rust
// ❌ 悪い例: Model内で複雑なロジック
impl Route {
    pub fn add_point(&mut self, point: Point) {
        let lines = self.lines_mut();
        let last_line = match lines.last_mut() {
            Some(line) => line,
            None => return,
        };

        // 直前のポイントと同じ場合は追加しない（ビジネスロジック）
        let last_point = last_line.last();
        if let Some(lp) = last_point {
            if lp.x == point.x && lp.y == point.y {
                return;
            }
        }
        last_line.push(point);
    }
}

// ✅ 良い例: AppStateで複雑なロジック
impl AppState {
    pub fn add_route_point(&self, player_id: PlayerId, point: Point) {
        let mut data = self.data.borrow_mut();
        if let Some(game) = &mut data.game {
            if let Some(wave) = game.waves.get_mut(data.current_wave_index) {
                if let Some(route) = wave.routes.iter_mut().find(|r| {
                    matches!(r, Route::Draw(d) if d.player_id == player_id)
                }) {
                    // 重複チェックのロジックをここに実装
                    if let Route::Draw(draw) = route {
                        if let Some(last_line) = draw.lines.last_mut() {
                            if let Some(last_point) = last_line.last() {
                                if last_point.x == point.x && last_point.y == point.y {
                                    return;  // 重複をスキップ
                                }
                            }
                            last_line.push(point);
                        }
                    }
                }
            }
        }
    }
}
```

## 他の設計パターンとの比較

### 代替案1: AppDataを直接公開する

```rust
pub struct AppState {
    pub data: RefCell<AppData>,  // publicにする
}

// 使う側
state.data.borrow_mut().selected_player_id = Some(player_id);
```

**デメリット**:
- カプセル化が失われる
- どこからでもデータを変更できるため、追跡が困難
- 意図が不明確（何のための操作か分からない）
- リファクタリング時に影響範囲が広い

### 代替案2: Arcと複数のRefCell

```rust
pub struct AppState {
    game: Arc<RefCell<Option<Game>>>,
    selected_player_id: Arc<RefCell<Option<PlayerId>>>,
    // ... 各フィールドを個別に管理
}
```

**デメリット**:
- 複雑性が増す（各フィールドを個別に管理）
- データの全体像が把握しにくい
- 関連するデータの一貫性を保つのが難しい

### 現在の設計（AppData + AppState）が優れている理由

1. **シンプル**: データと操作が明確に分離されている
2. **カプセル化**: データへのアクセスが制御されている
3. **拡張性**: 新しい操作を追加しやすい
4. **テスト性**: データ構造が単純で、テストデータを作りやすい
5. **保守性**: 変更の影響範囲が限定的

## ベストプラクティス

### ✅ DO: 専用メソッドを作る

意図を明確にし、コードの可読性を高める。

```rust
impl AppState {
    pub fn toggle_erase_mode(&self) {
        let mut data = self.data.borrow_mut();
        data.erase_mode = !data.erase_mode;
    }

    pub fn clear_selection(&self) {
        let mut data = self.data.borrow_mut();
        data.selected_player_id = None;
        data.dragging_player_id = None;
    }
}
```

### ✅ DO: スコープを最小限に保つ

借用は必要最小限の期間だけ保持する。

```rust
// ✅ 良い例
let value = {
    let data = state.data.borrow();
    data.some_field.clone()
};  // dataのスコープ終了
process(value);

// ❌ 悪い例
let data = state.data.borrow();
let value = data.some_field.clone();
process(value);  // dataの借用がまだ続いている
```

### ✅ DO: データ収集と更新を分離

`borrow()`と`borrow_mut()`を同時に使わない。

```rust
// ✅ 良い例
let collected_data = {
    let data = state.data.borrow();
    // データ収集
    data.game.as_ref().map(|g| g.players.len())
};

let mut data = state.data.borrow_mut();
// collected_dataを使って更新

// ❌ 悪い例
let data = state.data.borrow();
let count = data.game.as_ref().map(|g| g.players.len());
let mut data_mut = state.data.borrow_mut();  // パニック！
```

### ✅ DO: エラーハンドリングを適切に行う

操作が失敗する可能性がある場合は`Result`を返す。

```rust
impl AppState {
    pub fn update_player_name(&self, player_id: PlayerId, name: String) -> Result<(), String> {
        if name.is_empty() {
            return Err("名前は空にできません".to_string());
        }

        let mut data = self.data.borrow_mut();
        if let Some(game) = &mut data.game {
            if let Some(player) = game.players.iter_mut().find(|p| p.id == player_id) {
                player.name = name;
                Ok(())
            } else {
                Err("プレイヤーが見つかりません".to_string())
            }
        } else {
            Err("ゲームが開始されていません".to_string())
        }
    }
}
```

### ❌ DON'T: 長期間の可変借用

`data_mut()`を長時間保持しない。

```rust
// ❌ 悪い例
let mut data = state.data_mut();
data.field1 = value1;
// ... 多くの処理
data.field2 = value2;
// ... さらに処理
data.field3 = value3;

// ✅ 良い例
state.set_field1(value1);
// ... 処理
state.set_field2(value2);
// ... 処理
state.set_field3(value3);
```

### ❌ DON'T: AppStateメソッド内で他のAppStateメソッドを呼ぶ（2重借用に注意）

**重要**: 両方のメソッドが`data_mut()`を使う場合、2重借用でパニックする。

```rust
// ❌ 悪い例: 2重借用でパニック
impl AppState {
    pub fn create_game_from_setup(&self) {
        let mut data = self.data_mut();  // 1回目の可変借用
        data.game = Some(Game::new(area, players));

        for _ in 0..20 {
            self.add_wave();  // ← add_wave()内でdata_mut()を再度呼ぶ → パニック！
        }

        data.current_wave_index = 0;  // dataはまだ借用中
    }

    pub fn add_wave(&self) {
        let mut data = self.data_mut();  // 2回目の可変借用 → パニック！
        if let Some(game) = &mut data.game {
            game.waves.push(Wave::default());
        }
    }
}

// ✅ 良い例: 同じdataを使い回す
impl AppState {
    pub fn create_game_from_setup(&self) {
        let mut data = self.data_mut();
        data.game = Some(Game::new(area, players));

        // add_wave()を呼ばず、直接dataを使う
        if let Some(game) = &mut data.game {
            for _ in 0..20 {
                game.waves.push(Wave::default());
            }
        }

        data.current_wave_index = 0;
    }
}
```

**理由**: `data_mut()`は`RefCell::borrow_mut()`を呼ぶため、既に可変借用中の場合はパニックする。

**対策**:
1. 同じ`data`変数を使い回す（推奨）
2. スコープを分けて借用を解放してから次のメソッドを呼ぶ
3. ヘルパー関数を作って`data`を引数で渡す

```rust
// ✅ 対策2: スコープを分ける
impl AppState {
    pub fn create_game_from_setup(&self) {
        {
            let mut data = self.data_mut();
            data.game = Some(Game::new(area, players));
        }  // dataのスコープ終了、借用解放

        // 借用が解放されたので、add_wave()を呼べる
        for _ in 0..20 {
            self.add_wave();
        }

        let mut data = self.data_mut();
        data.current_wave_index = 0;
    }
}

// ✅ 対策3: ヘルパー関数
impl AppState {
    fn add_wave_internal(game: &mut Game) {
        game.waves.push(Wave::default());
    }

    pub fn create_game_from_setup(&self) {
        let mut data = self.data_mut();
        data.game = Some(Game::new(area, players));

        if let Some(game) = &mut data.game {
            for _ in 0..20 {
                Self::add_wave_internal(game);
            }
        }

        data.current_wave_index = 0;
    }
}
```

### ❌ DON'T: Viewから直接data_mutを呼ぶ

Viewはデータを直接変更せず、操作結果を返す。

```rust
// ❌ 悪い例（View内）
impl LocationView {
    pub fn render(state: &AppState, ui: &mut Ui) {
        if ui.button("Select").clicked() {
            state.data_mut().selected_player_id = Some(player_id);  // NG
        }
    }
}

// ✅ 良い例
impl LocationView {
    pub fn render(state: &AppState, ui: &mut Ui) -> LocationResult {
        let mut result = LocationResult::default();
        if ui.button("Select").clicked() {
            result.clicked_player = Some(player_id);
        }
        result
    }
}

impl LocationInteraction {
    pub fn handle(state: &AppState, result: &LocationResult) {
        if let Some(player_id) = result.clicked_player {
            state.set_selected_player_id(Some(player_id));
        }
    }
}
```

## まとめ

この設計パターンは以下の目的を達成します:

### 全体像

```
Model (データ構造 + シンプルなヘルパー)
    ↓ 保持
AppData (純粋なデータコンテナ)
    ↓ 内包
AppState (アクセス制御 + ビジネスロジック)
    ├── slices() → Slices（読み取り専用）→ View
    └── Actions（書き込み）→ 状態更新
```

### データフロー

```
Feature層:
    state.slices() → View::render(slices, ui) → Vec<Action>
                                                    ↓
    Actions::new(state) ← actions.handle_xxx(action)
```

### 設計原則

1. **Model層の責務**:
   - データ構造の定義（シリアライズ可能）
   - コンストラクタ、ゲッター、純粋な判定メソッド
   - 可変メソッドは`pub(crate)`（外部から状態変更できない）
   - **禁止**: 状態変更ロジック、ビジネスロジック

2. **AppData層の責務**:
   - アプリケーションデータの保持
   - **禁止**: メソッド実装（純粋なデータ構造のみ）

3. **AppState層の責務**:
   - データへのアクセス制御
   - **全ての**ビジネスロジックの実装
   - データの所有権管理
   - 可変参照の制限

### 達成される価値

1. **所有権の明確化**: データは`AppState`が所有し、管理する
2. **mutの限定**: 必要最小限の可変参照で済むようにする
3. **カプセル化**: データへの直接アクセスを制限し、メソッド経由で操作する
4. **一貫性**: 複雑な操作をメソッドにまとめ、データの整合性を保つ
5. **可読性**: 操作の意図がメソッド名で明確になる
6. **保守性**: 変更の影響範囲を限定し、リファクタリングを容易にする
7. **安全性**: Modelの可変メソッドを`pub(crate)`にすることで、意図しない変更を防ぐ

### 実装時のチェックリスト

- [ ] Modelに状態変更ロジックを実装していないか？
- [ ] Modelの可変メソッドは`pub(crate)`になっているか？
- [ ] ビジネスロジックは全て`AppState`に実装されているか？
- [ ] Viewから直接`AppData`や`Model`の状態を変更していないか？
- [ ] 複雑な操作は専用メソッドに切り出しているか？
- [ ] `data_mut()`の使用は最小限に抑えているか？
- [ ] **AppStateメソッド内で他のAppStateメソッドを呼ぶ際、2重借用に注意しているか？**

### 重要な注意点

1. **実装側（Feature、View、Interaction）**は**直接`AppData`やModelを変更せず**、必ず**`AppState`経由**でデータにアクセスする。これにより、データの所有権と変更の追跡が容易になる。

2. **AppStateメソッド内で他のAppStateメソッドを呼ぶときは2重借用に注意**する。両方が`data_mut()`を使う場合、パニックする。対策として：
   - 同じ`data`変数を使い回す（推奨）
   - スコープを分けて借用を解放する
   - ヘルパー関数を使う
