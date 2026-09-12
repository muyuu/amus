/**
 * 配布用のリリースビルドを作る
 * 引数: gpu | cpu
 *
 * 動かすマシンの OS 向けに作る。whisper.cpp と Vulkan のシェーダを含むため、
 * クロスコンパイルはしない。配布する OS ごとにそのマシンで実行する。
 *
 * 成果物は dist/ へ、どの OS のどの版か分かる名前で置く。
 */

import { spawnSync } from "child_process";
import * as fs from "fs";
import * as path from "path";
import { gpuFeature, requireVulkanSdk, windowsTargetDir } from "./gpu.js";

const OS_LABEL = {
    win32: "windows",
    linux: "linux",
    darwin: "macos",
};

const variant = process.argv[2];
if (variant !== "gpu" && variant !== "cpu") {
    console.error("gpu か cpu を指定してください");
    process.exit(1);
}

const osLabel = OS_LABEL[process.platform];
if (!osLabel) {
    console.error(`対応していない OS です: ${process.platform}`);
    process.exit(1);
}

const feature = variant === "gpu" ? gpuFeature() : null;
if (variant === "gpu" && !feature) {
    console.error(`GPU バックエンドに対応していない OS です: ${process.platform}`);
    process.exit(1);
}
if (feature) {
    requireVulkanSdk(feature);
}

const env = { ...process.env };
// GPU 版だけ target を移す。CPU 版は既定のままでよく、分けておけば互いに温まったまま残る。
const targetDir = variant === "gpu" ? windowsTargetDir() : null;
if (targetDir) {
    env.CARGO_TARGET_DIR = targetDir;
}

console.log(`対象: ${osLabel} / ${variant}${feature ? ` (${feature})` : ""}`);

const args = ["build", "--release"];
if (feature) {
    args.push("--features", feature);
}

const built = spawnSync("cargo", args, {
    stdio: "inherit",
    env,
});
if (built.status !== 0) {
    process.exit(built.status ?? 1);
}

const exeSuffix = process.platform === "win32" ? ".exe" : "";
const from = path.join(
    env.CARGO_TARGET_DIR || "target",
    "release",
    `memongus${exeSuffix}`,
);
const to = path.join("dist", `memongus-${osLabel}-${variant}${exeSuffix}`);

fs.mkdirSync("dist", { recursive: true });
fs.copyFileSync(from, to);
if (process.platform !== "win32") {
    fs.chmodSync(to, 0o755);
}

const sizeMb = (fs.statSync(to).size / (1024 * 1024)).toFixed(0);
console.log(`\n成果物: ${to} (${sizeMb} MB)`);
