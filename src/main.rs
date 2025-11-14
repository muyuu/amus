mod app;
mod models;

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
    
    // 日本語フォントを追加
    // macOSの場合、ヒラギノやNoto Sans CJKなどを試す
    // Windowsの場合、MS GothicやYu Gothicなどを試す
    // Linuxの場合、Noto Sans CJKなどを試す
    
    // システムフォントのパスを試す
    let font_paths = vec![
        // macOS
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/Library/Fonts/AppleGothic.ttf",
        // Windows (例)
        "C:/Windows/Fonts/msgothic.ttc",
        "C:/Windows/Fonts/yugothic.ttf",
        // Linux (例)
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ];
    
    // 利用可能なフォントを探す
    let mut japanese_font_found = false;
    for font_path in font_paths {
        if std::path::Path::new(font_path).exists() {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "japanese".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_insert_with(Vec::new)
                    .insert(0, "japanese".to_owned());
                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_insert_with(Vec::new)
                    .insert(0, "japanese".to_owned());
                japanese_font_found = true;
                break;
            }
        }
    }
    
    // フォントが見つからない場合、Noto Sans CJKをダウンロードして使用
    // または、システムのデフォルトフォントを使用
    if !japanese_font_found {
        // フォールバック: システムのデフォルトフォントを使用
        // この場合、日本語は表示されない可能性があるが、アプリは動作する
        eprintln!("警告: 日本語フォントが見つかりませんでした。日本語が正しく表示されない可能性があります。");
    }
    
    ctx.set_fonts(fonts);
}

