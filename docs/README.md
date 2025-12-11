# 開発ドキュメント

## 概要

このディレクトリには、決定された設計指針と開発ルールが含まれています。

## ドキュメント一覧

1. **[ARCHITECTURE.md](./ARCHITECTURE.md)**: アーキテクチャ設計指針
   - 基本構成とレイヤー構成
   - ディレクトリ構造
   - Featureの粒度とModelの必要性
   - グローバルなModelの更新方法
   - データフロー
   - 命名規則
   - 禁止事項

2. **[STATE_MANAGEMENT.md](./STATE_MANAGEMENT.md)**: 状態管理パターン
   - AppDataとAppStateの関係性
   - 設計意図（所有権、mutの制限、カプセル化）
   - 実装パターンとベストプラクティス
   - RefCellによる借用管理
   - 他の設計パターンとの比較

## 使い方

### チームのオンボーディング用

新しく参加したメンバーは、以下の順序でドキュメントを読むことを推奨します：

1. `ARCHITECTURE.md` - アーキテクチャ設計指針を理解する
2. `STATE_MANAGEMENT.md` - 状態管理パターンを理解する

### 設計に迷った時の参照用

設計判断に迷った場合は、以下を参照してください：

- **アーキテクチャ全般**: `ARCHITECTURE.md`
- **状態管理・データアクセス**: `STATE_MANAGEMENT.md`
- **Featureの粒度**: `ARCHITECTURE.md` の「Feature の粒度」セクション
- **Modelの更新方法**: `ARCHITECTURE.md` の「グローバルなModelの更新」セクション
