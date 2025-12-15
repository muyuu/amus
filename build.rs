fn main() {
    // ターゲットアーキテクチャを環境変数から取得
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    // Windowsネイティブビルドの場合のみアイコンを埋め込む
    // WASMビルド時はスキップ
    if target_os == "windows" && target_arch != "wasm32" {
        setup_windows_resources();
    }
}

#[cfg(target_os = "windows")]
fn setup_windows_resources() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icons/icon.ico");
    res.compile().unwrap();
}

#[cfg(not(target_os = "windows"))]
fn setup_windows_resources() {
    // 何もしない
}
