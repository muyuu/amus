use egui::*;

use crate::models::Player;

pub struct PlayerRectResult {
    pub clicked: bool,
    pub drag_started: bool,
    pub drag_stopped: bool,
}
pub fn player_rect(
    ui: &mut Ui,
    player: &Player,
    is_selected: bool,
    is_dragging: bool,
) -> PlayerRectResult {
    // ユーザー色の矩形（30x30px）
    let rect_size = Vec2::new(30.0, 30.0);
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

    PlayerRectResult {
        clicked: response.clicked(),
        drag_started: response.drag_started(),
        drag_stopped: response.drag_stopped(),
    }
}
