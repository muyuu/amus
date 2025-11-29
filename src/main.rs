#![windows_subsystem = "windows"]

mod app;
mod assets;
mod common;
mod features;
mod i18n;
mod models;
mod state;

use app::AmusApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Among Us 補助ツール"),
        ..Default::default()
    };

    eframe::run_native(
        "Among Us 補助ツール",
        options,
        Box::new(|cc| {
            // 日本語フォントの設定
            setup_custom_fonts(&cc.egui_ctx);
            Box::new(AmusApp::default())
        }),
    )
}

fn setup_custom_fonts(ctx: &egui::Context) {
    use egui::FontFamily;

    let mut fonts = egui::FontDefinitions::default();

    // 日本語フォントのパス（優先順位順）
    let font_paths = vec![
        // macOS - ヒラギノ角ゴシック（標準でインストールされている）
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/ヒラギノ角ゴ ProN W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        // macOS - その他の日本語フォント
        "/System/Library/Fonts/AppleGothic.ttf",
        // Windows
        "C:/Windows/Fonts/msgothic.ttc",
        "C:/Windows/Fonts/yugothic.ttf",
        "C:/Windows/Fonts/meiryo.ttc",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ];

    // 利用可能なフォントを探す
    let mut japanese_font_found = false;
    for font_path in font_paths {
        let path = std::path::Path::new(font_path);
        if path.exists() {
            if let Ok(font_data) = std::fs::read(path) {
                println!("日本語フォントを読み込みました: {}", font_path);
                fonts
                    .font_data
                    .insert("japanese".to_owned(), egui::FontData::from_owned(font_data));

                // 既存のフォントファミリーを取得して、日本語フォントを先頭に追加
                // プロポーショナルフォントファミリー
                let proportional = fonts
                    .families
                    .get_mut(&FontFamily::Proportional)
                    .expect("Proportional font family should exist");
                if !proportional.contains(&"japanese".to_owned()) {
                    proportional.insert(0, "japanese".to_owned());
                }

                // 等幅フォントファミリー
                let monospace = fonts
                    .families
                    .get_mut(&FontFamily::Monospace)
                    .expect("Monospace font family should exist");
                if !monospace.contains(&"japanese".to_owned()) {
                    monospace.insert(0, "japanese".to_owned());
                }

                japanese_font_found = true;
                break;
            } else {
                eprintln!("フォントファイルの読み込みに失敗しました: {}", font_path);
            }
        }
    }

    if !japanese_font_found {
        eprintln!("警告: 日本語フォントが見つかりませんでした。日本語が正しく表示されない可能性があります。");
        eprintln!("利用可能なフォントパスを確認してください。");
    }

    ctx.set_fonts(fonts);
}
