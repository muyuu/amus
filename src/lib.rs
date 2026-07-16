#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code, unused_imports))]
// components 配下は mod.rs が再エクスポート配線・<name>.rs が実装本体という構成を意図的に採る
#![allow(clippy::module_inception)]

// wasm の公開 API は `start()` のみ。各モジュールは crate 内部利用に閉じる
// （外部に晒さないことで API 表面を狭め、内部リファクタを自由にする）。
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
mod shell;
mod state;

use crate::log::Log;

use app::AmusApp;
use eframe::egui;
use egui::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    // コンソールにログを出力するための設定
    console_error_panic_hook::set_once();
    Log::init();

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

    let state = state::AppState::new();

    eframe::WebRunner::new()
        .start(
            canvas,
            options,
            Box::new(|cc| {
                // 日本語フォントの設定
                setup_japanese_fonts_wasm(&cc.egui_ctx);
                shell::setup_panel_bg(&cc.egui_ctx);
                Ok(Box::new(AmusApp::new(cc, state)))
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
    use web_sys::console;

    let mut fonts = FontDefinitions::default();

    // assets.rs のメソッドでフォントを読み込む
    // （リリースは include_dir! から、デバッグはファイルシステムから）
    let font_path = "fonts/NotoSansJP-Regular.ttf";
    if let Some(font_data) = crate::resources::AssetManager::load_font_from_assets(font_path) {
        console::log_1(
            &format!(
                "Successfully loaded font: {} ({} bytes)",
                font_path,
                font_data.len()
            )
            .into(),
        );
        shell::apply_japanese_font(&mut fonts, font_data);
    } else {
        console::warn_1(&format!("Warning: Failed to load font: {}", font_path).into());
    }

    ctx.set_fonts(fonts);
}
