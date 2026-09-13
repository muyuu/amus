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

// winres は Cargo.toml に `[lib]` と `[[bin]]` の両方があるクレートだと
// リソースを lib 側にリンクしてしまい、実行ファイルにアイコンが付かない
// （`cargo:rustc-link-lib=dylib=...` は lib がある場合そちらへ流れる古い方式のため）。
// embed-resource は `cargo:rustc-link-arg-bins=` で bin 側を名指しするため、これを避けられる。
#[cfg(target_os = "windows")]
fn setup_windows_resources() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let icon_path = std::path::Path::new(&manifest_dir).join("assets/icons/icon.ico");
    // rc.exe はプリプロセッサを通すため、バックスラッシュはエスケープ扱いになりうる。
    let icon_path = icon_path.to_string_lossy().replace('\\', "/");

    let version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let major = std::env::var("CARGO_PKG_VERSION_MAJOR").unwrap();
    let minor = std::env::var("CARGO_PKG_VERSION_MINOR").unwrap();
    let patch = std::env::var("CARGO_PKG_VERSION_PATCH").unwrap();

    let rc = format!(
        r#"1 VERSIONINFO
FILEVERSION {major},{minor},{patch},0
PRODUCTVERSION {major},{minor},{patch},0
FILEFLAGSMASK 0x3f
FILEFLAGS 0x0
FILEOS 0x40004
FILETYPE 0x1
{{
BLOCK "StringFileInfo"
{{
BLOCK "000004b0"
{{
VALUE "FileVersion", "{version}"
VALUE "FileDescription", "memongus"
VALUE "ProductName", "memongus"
VALUE "ProductVersion", "{version}"
}}
}}
BLOCK "VarFileInfo" {{ VALUE "Translation", 0x0, 0x04b0 }}
}}
1 ICON "{icon_path}"
"#
    );

    let rc_path = std::path::Path::new(&out_dir).join("app.rc");
    std::fs::write(&rc_path, rc).expect("windows resource script の書き込みに失敗");

    embed_resource::compile(&rc_path, embed_resource::NONE)
        .manifest_required()
        .expect("windows resource（アイコン）のコンパイルに失敗");

    println!("cargo:rerun-if-changed={icon_path}");
}

#[cfg(not(target_os = "windows"))]
fn setup_windows_resources() {
    // 何もしない
}
