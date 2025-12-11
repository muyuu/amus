/**
 * @file wasm-post-build.js
 * WASMビルド後に実行されるスクリプト
 */

import { promises as fs } from "fs";
import * as path from "path";
import { fileURLToPath } from "url";

async function main() {
    const __dirname = path.dirname(fileURLToPath(import.meta.url));
    const WASM_PKG_DIR = path.join(__dirname, "..", "web", "pkg");
    const GITIGNORE_PATH = path.join(WASM_PKG_DIR, ".gitignore");

    await removeGitignore(GITIGNORE_PATH);
    showCompletionMessage(WASM_PKG_DIR);
}

/**
 * .gitignoreファイルを削除する
 * @param {string} gitignorePath - .gitignoreファイルのパス
 */
async function removeGitignore(gitignorePath) {
    // ファイルの存在確認
    // 存在しなければスキップ
    try {
        await fs.access(gitignorePath);
    } catch (err) {
        if (err.code === 'ENOENT') {
            console.log('ℹ️  .gitignoreファイルは存在しません（スキップ）');
            return;
        }
        throw new Error(`ファイルアクセスエラー: ${err.message}`);
    }

    // ファイルの削除
    try {
        await fs.unlink(gitignorePath);
        console.log('✅ .gitignoreを削除しました');
    } catch (err) {
        throw new Error(`.gitignoreファイルの削除に失敗しました: ${err.message}`);
    }
}

/**
 * 完了メッセージを表示する
 * @param {string} wasmPkgDir - WASM出力ディレクトリのパス
 */
function showCompletionMessage(wasmPkgDir) {
    console.log(`🎉 WASMビルドが完了しました: ${wasmPkgDir}/`);
}

await main();
