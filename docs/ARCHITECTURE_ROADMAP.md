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

---

## Phase 5: Resources層とアプリケーションループの再設計（検討中）

### 背景

現在の設計は egui の即時モードGUI（毎フレーム `update` で描画）を前提としており、Feature は状態を持たない冪等な構造になっている。

しかし、以下のような「リソース」を必要とする機能が出てきた:
- **VoiceMemoFeature**: AudioRecorder（ハードウェア）、WhisperTranscriber（重い初期化）、非同期ダウンロードハンドル

これらは毎フレーム生成/破棄できないため、現状は AppState に入れているが、以下の問題がある:
- AppState の責務が「ゲームデータ」と「リソース管理」で混在
- シリアライズ可能なデータと不可能なリソースが同居

### 現状の問題点

```
現状:
┌─────────────────────────────────────────┐
│ AmusApp                                 │
│  └─ AppState                            │
│      ├─ AppData (シリアライズ可能)      │
│      │   ├─ Game, Player, Wave...       │
│      │   └─ UI状態                      │
│      └─ VoiceMemoFeature (リソース)     │  ← 責務が混在
│          ├─ AudioRecorder              │
│          ├─ WhisperTranscriber         │
│          └─ download_handle            │
└─────────────────────────────────────────┘

update():
  毎フレーム AppState を読み取り → View描画 → 状態更新
  （View内でリソースも触っている）
```

### 提案: Resources層の分離

```
提案:
┌─────────────────────────────────────────────────────────┐
│ main.rs                                                 │
│  └─ アプリ起動のみ                                      │
└─────────────────────────────────────────────────────────┘
          ↓
┌─────────────────────────────────────────────────────────┐
│ AmusApp                                                 │
│  ├─ state: AppState      (データ・シリアライズ可能)     │
│  └─ resources: Resources (リソース・シリアライズ不可)   │
│                                                         │
│  役割:                                                  │
│   - アプリの初期化・設定                                │
│   - リソースの生成・管理                                │
│   - アクションハンドリング                              │
└─────────────────────────────────────────────────────────┘
          ↓
┌─────────────────────────────────────────────────────────┐
│ update() (eframe::App::update)                          │
│  └─ Viewの描画のみ                                      │
│      View → AppActions を返す                           │
└─────────────────────────────────────────────────────────┘
          ↓
┌─────────────────────────────────────────────────────────┐
│ AmusApp::handle_actions()                               │
│  └─ アクション処理                                      │
│      - AppState の更新                                  │
│      - Resources の使用                                 │
└─────────────────────────────────────────────────────────┘
```

### 構造体の分離

```rust
/// データ状態（シリアライズ可能）
pub struct AppState {
    data: AppData,
    // Game, Player, Wave, UI状態など
}

/// リソース（シリアライズ不可）
pub struct Resources {
    #[cfg(not(target_arch = "wasm32"))]
    pub voice_memo: VoiceMemoResources,
    // 将来: pub audio_player: AudioPlayer,
    // 将来: pub network_client: NetworkClient,
}

/// VoiceMemo用リソース
#[cfg(not(target_arch = "wasm32"))]
pub struct VoiceMemoResources {
    pub recorder: Option<AudioRecorder>,
    pub transcriber: Option<WhisperTranscriber>,
    pub download_handle: Option<JoinHandle<Result<(), String>>>,
    pub state: VoiceMemoState,  // UIに必要な状態
}
```

### アプリケーションループの再設計

```rust
pub struct AmusApp {
    state: AppState,
    resources: Resources,
}

impl eframe::App for AmusApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 1. リソースの定期更新（タイマー、非同期完了チェック等）
        self.resources.update(ctx);

        // 2. Viewの描画 → アクションを収集
        let actions = self.render_views(ctx);

        // 3. アクションの処理
        self.handle_actions(actions);
    }
}

impl AmusApp {
    fn render_views(&self, ctx: &Context) -> Vec<AppAction> {
        let mut actions = Vec::new();

        // View は &AppState と &Resources（読み取り専用）を受け取る
        // クリックなどの操作結果を AppAction として返す

        CentralPanel::default().show(ctx, |ui| {
            actions.extend(MainView::render(&self.state, &self.resources, ui));
        });

        actions
    }

    fn handle_actions(&mut self, actions: Vec<AppAction>) {
        for action in actions {
            match action {
                // AppState の更新
                AppAction::SelectPlayer(id) => {
                    self.state.set_selected_player_id(id);
                }
                // Resources の使用
                AppAction::StartRecording => {
                    self.resources.voice_memo.start_recording();
                }
                AppAction::StopRecording => {
                    let result = self.resources.voice_memo.stop_recording();
                    // 結果を AppState に反映
                    self.state.add_voice_memo(result);
                }
            }
        }
    }
}
```

### メリット

1. **責務の明確化**
   - AppState: ゲームデータとUI状態（永続化可能）
   - Resources: ハードウェア・重いリソース（永続化不可）

2. **データフローの明確化**
   ```
   Resources.update() → View描画 → Actions収集 → Actions処理 → State/Resources更新
   ```

3. **テスタビリティ向上**
   - View は純粋な関数（入力 → 出力）
   - Actions はユニットテスト可能

4. **将来の拡張性**
   - ネットワーク通信、オーディオ再生などのリソースを追加しやすい
   - WASM/ネイティブで異なるリソース実装を切り替えやすい

### 移行計画

1. **Resources 構造体の作成**
   - `src/resources/mod.rs` を新設
   - VoiceMemoFeature から リソース部分を分離

2. **AmusApp の分割**
   - `state` と `resources` フィールドに分離
   - `update()` 内でのフローを整理

3. **View の引数変更**
   - `&mut AppState` → `&AppState, &Resources`（読み取り専用）
   - アクションは戻り値で返す

4. **Actions の統一**
   - 全Featureのアクションを `AppAction` enum に統合
   - または Feature ごとの Action を `AmusApp::handle_actions` で dispatch

### 検討事項

- **egui の制約**: `Window::show` のクロージャ内で `&mut self` が必要な場面がある
  - 対策: アクションを収集して後で処理するパターンで回避

- **パフォーマンス**: 毎フレーム `Vec<AppAction>` を生成するコスト
  - 対策: 小規模なので問題にならない見込み。必要なら `SmallVec` を使用

- **既存コードとの互換性**: 段階的に移行可能
  - 最初は VoiceMemo のみ Resources に移動
  - 他の Feature は現状のまま動作

---

## Phase 5.1: ディレクトリ構造の整理（シンプル版）

Phase 5 の前段階として、まずディレクトリ構造だけ整理する。
Action の使い方は現状維持（`Actions::new(state)` を View 内で呼ぶ）。

### 現状の問題

`features/voice_memo/` に view も resources も action も混在している:

```
src/features/voice_memo/
├── mod.rs           # Feature + Resources + Action 混在
├── recorder.rs      # リソース
├── transcriber.rs   # リソース
└── view.rs          # View
```

### 目標

責務ごとにディレクトリを分離:

```
src/
├── features/                    # View とエントリーポイント
│   └── voice_memo/
│       ├── mod.rs               # Feature（エントリーポイント）
│       └── view.rs              # View
├── resources/                   # 新設：ハードウェア・重いリソース
│   ├── mod.rs
│   ├── audio_recorder.rs        # features/voice_memo/recorder.rs から移動
│   ├── whisper_transcriber.rs   # features/voice_memo/transcriber.rs から移動
│   └── model_downloader.rs      # 新設（ダウンロード処理を分離）
└── state/
    └── actions/
        └── voice_memo.rs        # VoiceMemo用アクション（必要なら新設）
```

### 各ディレクトリの責務

| ディレクトリ | 責務 | 例 |
|-------------|------|-----|
| `features/` | View とそのエントリーポイント | VoiceMemoView, VoiceMemoFeature |
| `resources/` | ハードウェア・重いリソース | AudioRecorder, WhisperTranscriber |
| `state/actions/` | 状態変更ロジック | VoiceMemoAction のハンドリング |

### 移行手順

1. `src/resources/` ディレクトリを新設
2. `features/voice_memo/recorder.rs` → `resources/audio_recorder.rs`
3. `features/voice_memo/transcriber.rs` → `resources/whisper_transcriber.rs`
4. ダウンロード処理を `resources/model_downloader.rs` に分離
5. `features/voice_memo/mod.rs` から Resources 関連を削除し、Feature/View に専念

### 備考

- Action の使い方は変えない（`Actions::new(state)` を View 内で呼ぶ）
- Resources は AppState に保持する現状の構造を維持
- 将来 Phase 5 を実施する場合は、この整理が前提になる

## 関連ドキュメント

- [ARCHITECTURE.md](./ARCHITECTURE.md) - 現在のアーキテクチャ
- [STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md) - 現在の状態管理パターン
