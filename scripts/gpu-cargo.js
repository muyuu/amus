/**
 * GPU バックエンド付きで cargo を実行する
 * 引数: cargo のサブコマンド（build / run）以降をそのまま渡す
 *
 * バックエンドは OS ごとに決まる。Windows / Linux は Vulkan、macOS は Metal。
 *
 * Windows では target ディレクトリを短いパスへ移す。ggml-vulkan はシェーダ生成ツールを
 * 入れ子の ExternalProject として建てるため中間ファイルのパスが深くなり、既定の位置では
 * MAX_PATH(260) を超えて cl.exe が pdb を開けなくなる。OS の LongPathsEnabled は
 * MSVC のツール群に効かないため、短くする以外の回避手段がない。
 * 位置は AMUS_GPU_TARGET_DIR で変えられる。
 */

import { spawnSync } from "child_process";

const GPU_FEATURE = {
    win32: "gpu-vulkan",
    linux: "gpu-vulkan",
    darwin: "gpu-metal",
};

const WINDOWS_TARGET_DIR_FALLBACK = "C:\\amus-build";

const feature = GPU_FEATURE[process.platform];
if (!feature) {
    console.error(`GPU バックエンドに対応していない OS です: ${process.platform}`);
    process.exit(1);
}

const args = process.argv.slice(2);
if (args.length === 0) {
    console.error("cargo のサブコマンドを渡してください（例: build / run）");
    process.exit(1);
}

// Vulkan はビルドに SDK が要る（実行側はドライバ同梱のローダーで足りる）。
// 無いまま進むと whisper.cpp のビルドスクリプトが panic するだけで理由が分かりにくい。
if (feature === "gpu-vulkan" && !process.env.VULKAN_SDK) {
    console.error(
        [
            "VULKAN_SDK が設定されていません。Vulkan SDK のビルドには必要です。",
            "  Windows: winget install KhronosGroup.VulkanSDK",
            "  Linux:   ディストロの vulkan-headers / shaderc パッケージ",
            "インストール直後は、シェルを開き直さないと環境変数が反映されません。",
        ].join("\n"),
    );
    process.exit(1);
}

const env = { ...process.env };
if (process.platform === "win32") {
    env.CARGO_TARGET_DIR =
        env.AMUS_GPU_TARGET_DIR || WINDOWS_TARGET_DIR_FALLBACK;
}

// CPU ビルドとは feature が違うので、同じ target を共有すると切り替えるたびに
// 全体が再ビルドされる。分けておけば両方が温まったまま残る。
console.log(`バックエンド: ${feature}`);
if (env.CARGO_TARGET_DIR) {
    console.log(`ビルド先: ${env.CARGO_TARGET_DIR}`);
}

const result = spawnSync("cargo", [args[0], "--features", feature, ...args.slice(1)], {
    stdio: "inherit",
    env,
    shell: process.platform === "win32",
});

process.exit(result.status ?? 1);
