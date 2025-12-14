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

    // ユーザー名の最初の4文字を抽出して、2文字以降は改行
    // 文字は上下左右中央に配置
    if let Some(label) = &conf.label {
        let initials: Vec<char> = label.chars().take(4).collect();
        let font_id = FontId::proportional(12.0);
        let text_color = choose_text_color(color);

        if initials.len() > 2 {
            // 2文字ずつに分割して各行を個別に描画
            let first: String = initials.iter().take(2).collect();
            let rest: String = initials.iter().skip(2).collect();

            // 行の高さを計算
            let line_height = font_id.size * 1.2;
            let total_height = line_height * 2.0;
            let center = rect.center();

            // 1行目
            ui.painter().text(
                pos2(center.x, center.y - total_height / 4.0),
                Align2::CENTER_CENTER,
                first,
                font_id.clone(),
                text_color,
            );

            // 2行目
            ui.painter().text(
                pos2(center.x, center.y + total_height / 4.0),
                Align2::CENTER_CENTER,
                rest,
                font_id,
                text_color,
            );
        } else {
            // 2文字以下の場合は1行で中央に描画
            let text: String = initials.iter().collect();
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                text,
                font_id,
                text_color,
            );
        }
    }

    TileResult {
        clicked: response.clicked(),
        drag_started: response.drag_started(),
        drag_stopped: response.drag_stopped(),
        rect,
        response,
    }
}
