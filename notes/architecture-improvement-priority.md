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
| 133 | AppData の pub を pub(in crate::state) に絞る | S | 最高 | 1 | Todo |
| 134 | docs と実装の乖離を解消 (RefCell・Phase) | S | 高い | 1 | Todo |
| 135 | テスト基盤の整備 (Actions/AppState ユニット) | S | 高い | 1 | **Done** (PR #147、CI は別途) |
| 136 | Slices/AppState の API 重複整理 (players clone) | A | 高い | 2 | Todo |
| 137 | View→Actions の中央 dispatch 化 | A | 高い | 2 | Todo |
| 138 | Feature の render シグネチャ規約化 | A | 高い | 2 | Todo |
| 139 | Actions 層の費用対効果見直し | A | 低い | 2 | Todo |
| 140 | voice_memo/mod.rs の分割と cfg 整理 | B | 高い | 3 | Todo |
| 141 | エラー型導入 (Result<_,String> → thiserror) | B | 高い | 3 | Todo |
| 142 | i18n の HashMap lookup を enum key + 配列に | B | 高い | 3 | Todo |
| 143 | リソース層の unwrap/Mutex lock を panic 安全化 | B | 高い | 3 | Todo |
| 144 | pixels_per_point 固定値の設定可能化 | C | 低い | - | Todo |
| 145 | 命名規約と公開範囲の整理 | C | 低い | - | Todo |

## 推奨実施順

Project の tier/sprint をベースに、依存とリスクで微調整した「次に着手する順」。

### 最優先（Sprint 1 / Tier S）

1. **#135 テスト基盤の整備** ← まず最初
   - issue 自身が「他の Tier S/A より先に着地させる価値がある。テストが先にあれば
     #133, #134 の変更で何が壊れたか即時に分かる」と明記。
   - ラベルは「高い」だが、#133（最高）の安全網になるため実務的には先行させたい。
   - リグレッション検知ゼロの現状（`#[cfg(test)]` が 0 箇所）を埋めるのが先。

2. **#133 AppData の pub 絞り** ← 最高ラベルの本丸
   - 唯一の未着手「最高」。カプセル化の中核で、直接アクセス箇所の洗い出し→置換と
     影響範囲が広い。だからこそ #135 のテストを先に敷いてから入る。

3. **#134 docs 乖離の解消**
   - 低リスクなドキュメント作業。#133 の実装変更で docs がさらに動くので、
     #133 とセット、または直後に回すのが効率的。

### 次（Sprint 2 / Tier A — 層の意味付け）

4. **#137 中央 dispatch 化** — Sprint 2 の起点。#138/#139 の前提。
5. **#138 Feature render 規約化** — #137 の `AppAction` 統合に乗る形で実施。
6. **#136 Slices/AppState API 重複整理** — #133 と同じ state 層を触るので近接させると良い。
7. **#139 Actions 層の費用対効果** — 低優先。#137 完遂後に「黒字化 or 廃止」を判断。

### その後（Sprint 3 / Tier B — 運用負荷低減）

8. **#141 エラー型導入** — #140/#143 が触る層の `Result<_,String>` を置換するので先行。
9. **#143 unwrap / Mutex panic 安全化** — #141 のエラー型に乗せる。
10. **#140 voice_memo/mod.rs 分割** — #141 と関連（voice_memo の Result も対象）。
11. **#142 i18n 性能改善** — 独立性が高く、いつでも差し込める。

### 低優先（Tier C / Sprint 未割当）

12. **#144 pixels_per_point 設定可能化** — 操作性向上。storage 層（#133 の対象）に依存。
13. **#145 命名規約と公開範囲の整理** — #138/#133 の方針確定後に総仕上げとして。

## メモ

- Project のラベル順だと #133 が最高で先頭だが、#135 の安全網を先に置く方針を推奨。
  ここだけラベル順と入れ替えている（理由は上記）。
- Sprint 内は依存（#137→#138, #141→#143/#140）でほぼ順序が決まる。
- #132 は PR #146 で着地済み（最新コミット）。残る Tier S は #133/#134/#135 の 3 件。
