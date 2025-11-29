use egui::*;

use crate::constants::AppConstants;

/// 背景付きラベルを描画し、クリック状態を返す関数
pub fn background_label(ui: &mut Ui, text: &str) -> Response {
    let font_id = FontId::proportional(AppConstants::DEFAULT_FONT_SIZE);
    let ctx = ui.ctx();

    // 文字サイズを計算
    let galley = ctx.fonts(|f| f.layout_no_wrap(text.to_owned(), font_id.clone(), Color32::WHITE));

    let padding = egui::vec2(
        AppConstants::COM_BG_LABEL_PADDING_X,
        AppConstants::COM_BG_LABEL_PADDING_Y,
    );
    let size = galley.size() + padding * 2.0;

    // レイアウトに従って領域を確保
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

    // 背景描画
    let bg_color = Color32::BLACK;
    ui.painter().rect_filled(rect, 4.0, bg_color);

    // テキスト描画
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font_id,
        Color32::WHITE,
    );

    response
}
