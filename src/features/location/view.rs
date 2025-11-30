use crate::{
    components::background_label, constants::AppConstants,
    features::location::constants::LocationConstants, models::player, state::AppState,
};
use egui::*;

pub struct LocationView;

impl LocationView {
    /// 出現位置と終了時位置を描画
    pub fn render(state: &AppState, ui: &mut Ui) {
        Self::render_spawn_locations(state, ui);
        Self::render_end_locations(state, ui);
    }

    fn render_spawn_locations(state: &AppState, ui: &mut Ui) {
        // 出現場所を描画（四角）
        let spawn_locations = match state.spawn_locations() {
            Some(spawn_locations) => spawn_locations,
            None => return,
        };

        for (player_id, point) in spawn_locations {
            let player = match state.player(player_id) {
                Some(player) => player,
                None => return,
            };

            let color = player.color.to_egui_color();
            let rect = ui.max_rect();
            let pos = pos2(
                rect.min.x + point.x * rect.size().x,
                rect.min.y + point.y * rect.size().y,
            );
            let size = LocationConstants::SPAWN_LOCATION_SIZE;
            let stroke_width = LocationConstants::SPAWN_LOCATION_STROKE_WIDTH;
            let painter = ui.painter();
            painter.rect_filled(
                Rect::from_center_size(pos, Vec2::new(size, size)),
                stroke_width,
                color,
            );
            painter.rect_stroke(
                Rect::from_center_size(pos, Vec2::new(size, size)),
                stroke_width,
                (stroke_width, Color32::WHITE),
                StrokeKind::Inside,
            );
        }
    }

    fn render_end_locations(state: &AppState, ui: &mut Ui) {
        let end_locations = match state.end_locations() {
            Some(end_locations) => end_locations,
            None => return,
        };

        // 終了時位置を描画（丸）
        for (player_id, point) in end_locations {
            let player = match state.player(player_id) {
                Some(p) => p,
                None => continue,
            };

            let color = player.color.to_egui_color();
            let rect = ui.max_rect();
            let pos = pos2(
                rect.min.x + point.x * rect.size().x,
                rect.min.y + point.y * rect.size().y,
            );

            // サイズ定義は1辺の長さなので半径にするため2で割る
            let radius = LocationConstants::END_LOCATION_SIZE / 2.0;
            let stroke_width = LocationConstants::END_LOCATION_STROKE_WIDTH;

            let painter = ui.painter();
            painter.circle_filled(pos, radius, color);
            painter.circle_stroke(pos, radius, (stroke_width, Color32::WHITE));

            Self::render_dead_mark(&player, painter, pos);
            Self::render_label(ui, &player, pos, radius);
        }
    }

    // 死亡している場合はバツ印を描画
    fn render_dead_mark(player: &player::Player, painter: &Painter, pos: Pos2) {
        if player.alive {
            return;
        }

        let cross_size = LocationConstants::END_LOCATION_DEAD_MARK_SIZE;
        let cross_width = LocationConstants::END_LOCATION_DEAD_MARK_STROKE_WIDTH;

        // 左上から右下への線
        painter.line_segment(
            [
                pos2(pos.x - cross_size, pos.y - cross_size),
                pos2(pos.x + cross_size, pos.y + cross_size),
            ],
            (cross_width, Color32::BLACK),
        );

        // 右上から左下への線
        painter.line_segment(
            [
                pos2(pos.x + cross_size, pos.y - cross_size),
                pos2(pos.x - cross_size, pos.y + cross_size),
            ],
            (cross_width, Color32::BLACK),
        );
    }

    fn render_label(ui: &mut Ui, player: &player::Player, center: Pos2, radius: f32) {
        let galley = ui.painter().layout_no_wrap(
            player.name.to_owned(),
            FontId::proportional(16.0),
            Color32::WHITE,
        );

        let padding = vec2(
            AppConstants::COM_BG_LABEL_PADDING_X,
            AppConstants::COM_BG_LABEL_PADDING_Y,
        );
        let size = galley.size() + padding * 2.0;

        let label_pos = Pos2::new(center.x, center.y - radius - 20.0);
        let rect = Rect::from_center_size(label_pos, size);
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            background_label(ui, &player.name);
        });
    }
}
