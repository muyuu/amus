#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// components 配下は mod.rs が再エクスポート配線・<name>.rs が実装本体という構成を意図的に採る
#![allow(clippy::module_inception)]

mod app;
mod app_action;
mod common;
mod components;
mod constants;
mod features;
mod i18n;
mod log;
mod models;
mod resources;
mod state;

use crate::i18n::keys::*;
use crate::log::Log;
use app::AmusApp;
use eframe::egui;
use egui::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    Log::init();

    let state = state::AppState::new();
    let app_title = state.t(APP_TITLE);

    // アイコンを事前に読み込み（Context不要なため先に実行）
    let icon = resources::AssetManager::load_icon_static();

    let mut viewport = ViewportBuilder::default()
        .with_inner_size([1200.0, 840.0])
        .with_title(&app_title);

    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        &app_title,
        options,
        Box::new(|cc| {
            // 日本語フォントの設定
            setup_custom_fonts(&cc.egui_ctx);
            setup_panel_bg(&cc.egui_ctx);

            // アセットマネージャーの初期化
            resources::AssetManager::initialize(&cc.egui_ctx);

            Ok(Box::new(AmusApp::new(cc, state)))
        }),
    )
}

fn setup_panel_bg(ctx: &Context) {
    use Style;

    let mut style: Style = (*ctx.style()).clone();
    style.visuals.panel_fill = Color32::BLACK;
    ctx.set_style(style);
}

fn setup_custom_fonts(ctx: &Context) {
    use FontFamily;

    let mut fonts = FontDefinitions::default();

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
                crate::log_info!(
                    "Font",
                    format!("日本語フォントを読み込みました: {}", font_path)
                );
                fonts.font_data.insert(
                    "japanese".to_owned(),
                    FontData::from_owned(font_data).into(),
                );

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
                crate::log_error!(
                    "Font",
                    format!("フォントファイルの読み込みに失敗しました: {}", font_path)
                );
            }
        }
    }

    if !japanese_font_found {
        crate::log_warn!(
            "Font",
            "日本語フォントが見つかりませんでした。日本語が正しく表示されない可能性があります。利用可能なフォントパスを確認してください。"
        );
    }

    ctx.set_fonts(fonts);
}
