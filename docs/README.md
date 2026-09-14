![icon](../assets/images/app-icon.png)
![logo](../assets/images/logo.png)


## 概要

このディレクトリには、決定された設計指針と開発ルールが含まれています。

## ドキュメント一覧

1. **[DEVELOPMENT.md](./DEVELOPMENT.md)**: 開発ガイド
   - セットアップ、ビルド・実行（ネイティブ / WASM / GPU / 配布用）
   - push 前ゲート、リポジトリ運用

2. **[ARCHITECTURE.md](./ARCHITECTURE.md)**: アーキテクチャ設計指針
   - 基本構成とレイヤー構成（Model / AppData / AppState / Slices / Actions / Feature / Resources）
   - ディレクトリ構造
   - Featureの粒度
   - データフロー
   - 命名規則
   - 禁止事項

3. **[STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md)**: 状態管理パターン
   - AppDataとAppStateの関係性（読み取り Slices / 書き込み Actions）
   - 設計意図（所有権、&mutの限定、カプセル化）
   - 実装パターンとベストプラクティス

4. **[VOICE_MEMO.md](./VOICE_MEMO.md)**: 音声メモ設計指針
   - 録音・検知・書き起こしの3層構成
   - 常時録音リングバッファとターン区間
   - ホットワードによるターン境界検知
   - 認識精度（プロンプト予算、語彙補正）

5. **[decisions.md](./decisions.md)**: 設計判断記録（ADR）
   - 採用した設計の「なぜ」と却下した案
   - 新たなアーキテクチャ方針は GitHub Issue で検討し、決着したらここに ADR として追記する

## 使い方

### チームのオンボーディング用

新しく参加したメンバーは、以下の順序でドキュメントを読むことを推奨します：

1. `DEVELOPMENT.md` - セットアップ・ビルド・実行の方法を理解する
2. `ARCHITECTURE.md` - アーキテクチャ設計指針を理解する
3. `STATE_MANAGEMENT.md` - 状態管理パターンを理解する

### 設計に迷った時の参照用

設計判断に迷った場合は、以下を参照してください：

- **アーキテクチャ全般**: `ARCHITECTURE.md`
- **状態管理・データアクセス**: `STATE_MANAGEMENT.md`
- **Featureの粒度**: `ARCHITECTURE.md` の「Feature の粒度」セクション
- **読み取り/書き込みの分離（Slices / Actions）**: `STATE_MANAGEMENT.md`
