fn main() {
    // ビルドターゲットを環境変数から取得
    let target = std::env::var("TARGET").unwrap_or_default();

    // Windowsネイティブビルドの場合のみアイコンを埋め込む
    // WASMビルド時はスキップ
    if target.contains("windows") && !target.contains("wasm32") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icons/icon.ico");
        res.compile().unwrap();
    }
}
