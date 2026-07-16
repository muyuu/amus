//! native / wasm 共通のアプリシェル初期化。
//!
//! パネル背景と日本語フォントの「ファミリーへのマージ」は両ターゲットで同一。
//! フォントデータの取得元（OS のフォントパス or 埋め込み assets）だけが異なるため、
//! 取得は各エントリーポイント（main.rs / lib.rs）が行い、共通処理はここに寄せる。

use egui::{Color32, Context, FontData, FontDefinitions, FontFamily, Style};

/// パネル背景を黒にする。
pub fn setup_panel_bg(ctx: &Context) {
    let mut style: Style = (*ctx.style()).clone();
    style.visuals.panel_fill = Color32::BLACK;
    ctx.set_style(style);
}

/// 読み込み済みの日本語フォントデータを登録し、Proportional / Monospace の
/// 先頭に差し込む。取得元（OS パス or assets）は呼び出し側が用意する。
pub fn apply_japanese_font(fonts: &mut FontDefinitions, font_data: Vec<u8>) {
    fonts.font_data.insert(
        "japanese".to_owned(),
        FontData::from_owned(font_data).into(),
    );

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let list = fonts
            .families
            .get_mut(&family)
            .expect("font family should exist");
        if !list.contains(&"japanese".to_owned()) {
            list.insert(0, "japanese".to_owned());
        }
    }
}
