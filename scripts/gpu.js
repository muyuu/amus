/**
 * GPU ビルドの設定
 *
 * whisper.cpp のバックエンドはコンパイル時に固定されるため、OS ごとに feature が決まる。
 */

const GPU_FEATURE = {
    win32: "gpu-vulkan",
    linux: "gpu-vulkan",
    darwin: "gpu-metal",
};

/** この OS の GPU feature。対応していなければ `null`。 */
export function gpuFeature() {
    return GPU_FEATURE[process.platform] ?? null;
}

/**
 * GPU ビルドで使う target ディレクトリ。Windows 以外は既定のままでよいので `null`。
 *
 * ggml-vulkan はシェーダ生成ツールを入れ子の ExternalProject として建てるため中間
 * ファイルのパスが深くなり、既定の位置では MAX_PATH(260) を超えて cl.exe が pdb を
 * 開けなくなる。OS の LongPathsEnabled は MSVC のツール群に効かないため、短くする
 * 以外の回避手段がない。位置は AMUS_GPU_TARGET_DIR で変えられる。
 */
export function windowsTargetDir() {
    if (process.platform !== "win32") {
        return null;
    }
    return process.env.AMUS_GPU_TARGET_DIR || "C:\\amus-build";
}

/**
 * Vulkan SDK が要るのに無ければ、理由を説明して終了する。
 *
 * 無いまま進むと whisper.cpp のビルドスクリプトが panic するだけで理由が分かりにくい。
 */
export function requireVulkanSdk(feature) {
    if (feature !== "gpu-vulkan" || process.env.VULKAN_SDK) {
        return;
    }

    console.error(
        [
            "VULKAN_SDK が設定されていません。Vulkan のビルドには SDK が必要です。",
            "  Windows: winget install KhronosGroup.VulkanSDK",
            "  Linux:   LunarG の vulkan-sdk パッケージ",
            "インストール直後は、シェルを開き直さないと環境変数が反映されません。",
        ].join("\n"),
    );
    process.exit(1);
}
