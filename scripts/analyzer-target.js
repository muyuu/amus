/**
 * amus.code-workspace を編集して rust-analyzer のオプションを変更する
 * 引数: release or debug (default: debug)
 */ 

import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "url";
import JSON5 from "json5";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const ALLOWED_TARGETS = ["release", "debug"];

function getTargetArg() {
    const arg = process.argv[2] || "debug";

    if (!ALLOWED_TARGETS.includes(arg)) {
        console.error("ターゲット指定が正しくないです。 'release' か 'debug' で指定して。");
        process.exit(1);
    }

    return arg === 'debug' ? null : `--${arg}`;
}

try {
    const target = getTargetArg();
    const workspacePath = path.join(__dirname, "..", "amus.code-workspace");
    const workspaceJson = JSON5.parse(fs.readFileSync(workspacePath, "utf-8"));

    // settings に rust-analyzer.cargo.extraArgs を追加または更新
    if (!workspaceJson.settings && target !== null) {
        workspaceJson.settings = { "rust-analyzer.cargo.extraArgs": [target] };
    }

    if (workspaceJson.settings && target === null) {
        delete workspaceJson.settings["rust-analyzer.cargo.extraArgs"];
    }

    if (workspaceJson.settings && target !== null) {
        workspaceJson.settings["rust-analyzer.cargo.extraArgs"] = [target];
    }

    // 変更をファイルに書き戻す
    fs.writeFileSync(workspacePath, JSON.stringify(workspaceJson, null, 4), "utf-8");
    console.log("Updated rust-analyzer.cargo.extraArgs in amus.code-workspace");
} catch (error) {
    console.error("Error updating amus.code-workspace:", error);
}
