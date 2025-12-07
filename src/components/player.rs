use egui::*;

use crate::models::Player;

pub struct PlayerRectResult {
    pub clicked: bool,
    pub drag_started: bool,
    pub drag_stopped: bool,
    pub rect: Rect,
    pub response: Response,
}
pub fn player_rect(
    ui: &mut Ui,
    player: &Player,
    is_selected: bool,
    is_dragging: bool,
) -> PlayerRectResult {
    // ユーザー色の矩形（40x40px）
    let rect_size = Vec2::new(40.0, 40.0);
    let (rect, response) = ui.allocate_exact_size(rect_size, Sense::click_and_drag());

    // ユーザーの色を取得
    let player_color = player.color.to_egui_color();

    // 選択中またはドラッグ中の視覚的フィードバック
    let color = if is_dragging || (is_selected && response.hovered()) {
        player_color.linear_multiply(0.7) // 少し暗くする
    } else if is_selected {
        player_color.linear_multiply(0.9) // 選択中は少し暗く
    } else {
        player_color
    };

    // 選択中のユーザーには枠線を表示
    if is_selected {
        ui.painter()
            .rect_stroke(rect, 4.0, (2.0, Color32::WHITE), StrokeKind::Inside);
    }

    // 矩形を描画
    ui.painter().rect_filled(rect, 4.0, color);

    // ユーザー名の最初の3文字を抽出
    let initials: String = player.name.chars().take(3).collect();

    // 矩形の中央に文字を描画
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        initials,
        FontId::proportional(12.0),
        choose_text_color(color), // 背景色に合わせて文字色を調整
    );

    PlayerRectResult {
        clicked: response.clicked(),
        drag_started: response.drag_started(),
        drag_stopped: response.drag_stopped(),
        rect,
        response,
    }
}

fn choose_text_color(bg: Color32) -> Color32 {
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
