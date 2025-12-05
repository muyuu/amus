pub mod app;
pub mod assets;
pub mod common;
pub mod components;
pub mod constants;
pub mod features;
pub mod i18n;
pub mod models;
pub mod state;

#[cfg(target_arch = "wasm32")]
use app::AmusApp;
#[cfg(target_arch = "wasm32")]
use eframe::egui;
#[cfg(target_arch = "wasm32")]
use egui::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // コンソールにログを出力するための設定
    console_error_panic_hook::set_once();

    // DOMからcanvas要素を取得
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window not found"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("document not found"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str(&format!("canvas element '{}' not found", canvas_id)))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| JsValue::from_str("element is not a canvas"))?;

    let options = eframe::WebOptions::default();

    eframe::WebRunner::new()
        .start(
            canvas,
            options,
            Box::new(|cc| {
                // 日本語フォントの設定
                setup_japanese_fonts_wasm(&cc.egui_ctx);
                setup_panel_bg(&cc.egui_ctx);
                Ok(Box::new(AmusApp::default()))
            }),
        )
        .await
        .map_err(|err| {
            // eframe::ErrorをJsValueに変換（Debugフォーマットを使用）
            let err_msg = format!("{:?}", err);
            JsValue::from_str(&err_msg)
        })
}

#[cfg(target_arch = "wasm32")]
fn setup_japanese_fonts_wasm(ctx: &Context) {
    use FontFamily;

    let mut fonts = FontDefinitions::default();

    // assets.rsのメソッドを使ってフォントを読み込む
    // リリースビルドではinclude_dir!から、デバッグビルドではファイルシステムから読み込む
    use web_sys::console;

    let font_path = "fonts/NotoSansJP-Regular.ttf";
    if let Some(font_data) = crate::assets::AssetManager::load_font_from_assets(font_path) {
        console::log_1(
            &format!(
                "Successfully loaded font: {} ({} bytes)",
                font_path,
                font_data.len()
            )
            .into(),
        );
        fonts.font_data.insert(
            "japanese".to_owned(),
            FontData::from_owned(font_data).into(),
        );

        // プロポーショナルフォントファミリーに日本語フォントを追加
        let proportional = fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .expect("Proportional font family should exist");
        if !proportional.contains(&"japanese".to_owned()) {
            proportional.insert(0, "japanese".to_owned());
        }

        // 等幅フォントファミリーに日本語フォントを追加
        let monospace = fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .expect("Monospace font family should exist");
        if !monospace.contains(&"japanese".to_owned()) {
            monospace.insert(0, "japanese".to_owned());
        }
    } else {
        console::warn_1(&format!("Warning: Failed to load font: {}", font_path).into());
    }

    ctx.set_fonts(fonts);
}

#[cfg(target_arch = "wasm32")]
fn setup_panel_bg(ctx: &Context) {
    use Style;

    let mut style: Style = (*ctx.style()).clone();
    style.visuals.panel_fill = Color32::BLACK;
    ctx.set_style(style);
}
