use egui::*;

use crate::common::choose_text_color;

pub struct TileBaseConf {
    pub color: Color32,
    pub label: Option<String>,
    pub size: Vec2,
    pub is_selected: bool,
    pub is_dragging: bool,
}

#[allow(dead_code)]
pub struct TileResult {
    pub clicked: bool,
    pub drag_started: bool,
    pub drag_stopped: bool,
    pub rect: Rect,
    pub response: Response,
}
pub fn base(ui: &mut Ui, conf: &TileBaseConf) -> TileResult {
    // ユーザー色の矩形
    let rect_size = conf.size;

    let (rect, response) = ui.allocate_exact_size(rect_size, Sense::click_and_drag());

    // ユーザーの色を取得
    let player_color = conf.color;

    // 選択中またはドラッグ中の視覚的フィードバック
    let color = if conf.is_dragging || (conf.is_selected && response.hovered()) {
        player_color.linear_multiply(0.7) // 少し暗くする
    } else if conf.is_selected {
        player_color.linear_multiply(0.9) // 選択中は少し暗く
    } else {
        player_color
    };

    // 選択中のユーザーには枠線を表示
    if conf.is_selected {
        ui.painter()
            .rect_stroke(rect, 4.0, (2.0, Color32::WHITE), StrokeKind::Inside);
    }

    // 矩形を描画
    ui.painter().rect_filled(rect, 4.0, color);

    // ユーザー名の最初の3文字を抽出

    if let Some(label) = &conf.label {
        let initials: String = label.chars().take(3).collect();

        // 矩形の中央に文字を描画
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            initials,
            FontId::proportional(12.0),
            choose_text_color(color), // 背景色に合わせて文字色を調整
        );
    }

    TileResult {
        clicked: response.clicked(),
        drag_started: response.drag_started(),
        drag_stopped: response.drag_stopped(),
        rect,
        response,
    }
}
