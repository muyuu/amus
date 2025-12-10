use egui::*;

/// 与えられた背景色に対して、適切な文字色（黒または白）を選択する関数
pub fn choose_text_color(bg: Color32) -> Color32 {
    // RGB値を0〜255で取得
    let r = bg.r() as f32;
    let g = bg.g() as f32;
    let b = bg.b() as f32;

    // 輝度を計算 (ITU-R BT.601)
    let luminance = 0.299 * r + 0.587 * g + 0.114 * b;

    if luminance > 128.0 {
        Color32::BLACK // 明るい背景なら黒文字
    } else {
        Color32::WHITE // 暗い背景なら白文字
    }
}
