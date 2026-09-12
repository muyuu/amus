/**
 * GPU バックエンド付きで cargo を実行する
 * 引数: cargo のサブコマンド（build / run）以降をそのまま渡す
 */

import { spawnSync } from "child_process";
import { gpuFeature, requireVulkanSdk, windowsTargetDir } from "./gpu.js";

const feature = gpuFeature();
if (!feature) {
    console.error(`GPU バックエンドに対応していない OS です: ${process.platform}`);
    process.exit(1);
}

const args = process.argv.slice(2);
if (args.length === 0) {
    console.error("cargo のサブコマンドを渡してください（例: build / run）");
    process.exit(1);
}

requireVulkanSdk(feature);

const env = { ...process.env };
const targetDir = windowsTargetDir();
if (targetDir) {
    env.CARGO_TARGET_DIR = targetDir;
}

// CPU ビルドとは feature が違うので、同じ target を共有すると切り替えるたびに
// 全体が再ビルドされる。分けておけば両方が温まったまま残る。
console.log(`バックエンド: ${feature}`);
if (targetDir) {
    console.log(`ビルド先: ${targetDir}`);
}

const result = spawnSync("cargo", [args[0], "--features", feature, ...args.slice(1)], {
    stdio: "inherit",
    env,
});

process.exit(result.status ?? 1);
