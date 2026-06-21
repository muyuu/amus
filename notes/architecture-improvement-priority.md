# アーキテクチャ改善 — 優先順位メモ

Project: https://github.com/users/muyuu/projects/2
更新日: 2026-06-21

GitHub Project に登録された 14 issue を、tier / sprint / 優先度ラベル / 依存関係 /
リスクの観点で並べ直したもの。Project 側のメタ情報を一次ソースとし、それに「何を先に
やると後続が楽か」という実務判断を重ねている。

## 全 issue 一覧（Project の状態）

| # | タイトル | Tier | ラベル | Sprint | Status |
|---|---|---|---|---|---|
| 132 | Model 層から egui 依存を剥がす (color.rs) | S | 最高 | 1 | **Done** (PR #146) |
| 133 | AppData の pub を pub(in crate::state) に絞る | S | 最高 | 1 | **Done** (PR #148) |
| 134 | docs と実装の乖離を解消 (RefCell・Phase) | S | 高い | 1 | **Done** (PR #151) |
| 135 | テスト基盤の整備 (Actions/AppState ユニット) | S | 高い | 1 | **Done** (PR #147 + CI #149/#150) |
| 136 | Slices/AppState の API 重複整理 (players clone) | A | 高い | 2 | **Done** (PR #154) |
| 137 | View→Actions の中央 dispatch 化 | A | 高い | 2 | **Done** (PR #155) |
| 138 | Feature の render シグネチャ規約化 | A | 高い | 2 | **Done** (PR #157) |
| 139 | Actions 層の費用対効果見直し | A | 低い | 2 | **Done** (残す / ADR 0002) |
| 140 | voice_memo/mod.rs の分割と cfg 整理 | B | 高い | 3 | **Done** (PR #153) |
| 141 | エラー型導入 (Result<_,String> → thiserror) | B | 高い | 3 | Todo |
| 142 | i18n の HashMap lookup を enum key + 配列に | B | 高い | 3 | Todo |
| 143 | リソース層の unwrap/Mutex lock を panic 安全化 | B | 高い | 3 | Todo |
| 144 | pixels_per_point 固定値の設定可能化 | C | 低い | - | Todo |
| 145 | 命名規約と公開範囲の整理 | C | 低い | - | Todo |

## 推奨実施順

Project の tier/sprint をベースに、依存とリスクで微調整した「次に着手する順」。

### 最優先（Sprint 1 / Tier S）— **完了**

Sprint 1（#132〜#135）はすべて着地済み。あわせて fmt/clippy のツールチェイン固定と
GitHub Actions（fast: fmt+wasm clippy / native: clippy+test）も整備した。

1. **#135 テスト基盤の整備** — Done（PR #147 でユニットテスト33ケース、CI は #149/#150）
2. **#133 AppData の pub 絞り** — Done（PR #148）
3. **#134 docs 乖離の解消** — Done（PR #151。RefCell/Interaction 記述の刷新、ROADMAP は
   進捗を除き方針のみに）

### 次（Sprint 2 / Tier A — 層の意味付け）— **完了**

4. **#137 中央 dispatch 化** — Done (PR #155)。update→render→action の3段に統一。
5. **#138 Feature render 規約化** — Done (PR #157)。render は `&Slices`＋`Vec<AppAction>`。
6. **#136 Slices/AppState API 重複整理** — Done (PR #154)。読み取りを Slices に一本化。
7. **#139 Actions 層の費用対効果** — Done。「段取り層として残す」と判断（ADR 0002。フック点は handle_actions）。

### その後（Sprint 3 / Tier B — 運用負荷低減）

8. **#141 エラー型導入** — #140/#143 が触る層の `Result<_,String>` を置換するので先行。
9. **#143 unwrap / Mutex panic 安全化** — #141 のエラー型に乗せる。
10. **#140 voice_memo/mod.rs 分割** — #141 と関連（voice_memo の Result も対象）。
11. **#142 i18n 性能改善** — 独立性が高く、いつでも差し込める。

### 低優先（Tier C / Sprint 未割当）

12. **#144 pixels_per_point 設定可能化** — 操作性向上。storage 層（#133 の対象）に依存。
13. **#145 命名規約と公開範囲の整理** — #138/#133 の方針確定後に総仕上げとして。

## メモ

- **Sprint 1（#132〜135）と Sprint 2（#136〜139）は完了**。View→Actions→State の設計アークが閉じた。
  あわせて #96・#140、CI 基盤（fast/native 分割）、fmt/clippy のツールチェイン固定、設計判断の ADR 化（decisions.md）も整備。
- 残るは Tier B/C の運用品質・磨き込みと機能系:
  - **#141 エラー型導入** — #143 が乗る土台なので先行。Sprint 内は #141→#143。
  - **#143 panic 安全化（resources の unwrap/Mutex）** — 局所的。
  - **#142 i18n 性能（HashMap→enum）** — 独立、いつでも差し込める。
  - **#144 pixels_per_point 設定可能化** — 軽量・操作性。
  - **#145 命名・公開範囲の整理** — 総仕上げ。
  - 機能系: #77/#76/#15/#12/#32。
