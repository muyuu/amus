use crate::{
    components::background_label_with_font_size, constants::AppConstants,
    features::location::constants::LocationConstants, models::player, state::AppState,
};
use egui::*;

#[derive(Default)]
pub struct SpawnResult {
    pub drag_start_player_id: Option<player::PlayerId>,
    pub dragging_player_id: Option<player::PlayerId>,
    pub drag_stop_player_id: Option<player::PlayerId>,
    pub click_player_id: Option<player::PlayerId>,
}

#[derive(Default)]
pub struct EndResult {
    pub drag_start_player_id: Option<player::PlayerId>,
    pub dragging_player_id: Option<player::PlayerId>,
    pub drag_stop_player_id: Option<player::PlayerId>,
    pub click_player_id: Option<player::PlayerId>,
}

#[derive(Default)]
pub struct LocationViewResult {
    pub spawn: SpawnResult,
    pub end: EndResult,
}

pub struct LocationView;

impl LocationView {
    /// 出現位置と終了時位置を描画
    pub fn render(state: &AppState, ui: &mut Ui) -> LocationViewResult {
        let mut result = LocationViewResult::default();

        let spawn_result = Self::render_spawn_locations(state, ui);
        let end_result = Self::render_end_locations(state, ui);

        result.spawn = spawn_result;
        result.end = end_result;
        result
    }

    fn render_spawn_locations(state: &AppState, ui: &mut Ui) -> SpawnResult {
        let mut result = SpawnResult::default();

        // 出現場所を描画（四角）
        let spawn_locations = match state.spawn_locations() {
            Some(spawn_locations) => spawn_locations,
            None => return result,
        };

        for (player_id, point) in spawn_locations {
            let player = match state.player(player_id) {
                Some(player) => player,
                None => return result,
            };

            let color = player.color.to_egui_color();
            let ui_rect = ui.max_rect();
            let size = LocationConstants::SPAWN_LOCATION_SIZE;

            // マージンを計算（四方同じサイズ分）
            let margin = size;

            // 中心座標を計算してクランプ
            let center = Self::calculate_clamped_position(&point, ui_rect, margin, margin, margin);

            // 描画領域を確保してResponseを取得
            let rect = Rect::from_center_size(center, Vec2::new(size, size));
            let response = ui.allocate_rect(rect, Sense::click_and_drag());

            // イベント処理
            if response.clicked() {
                result.click_player_id = Some(player_id);
            }
            if response.drag_started() {
                result.drag_start_player_id = Some(player_id);
            }
            if response.dragged() {
                result.dragging_player_id = Some(player_id);
            }
            if response.drag_stopped() {
                result.drag_stop_player_id = Some(player_id);
            }

            // 描画
            let stroke_width = LocationConstants::SPAWN_LOCATION_STROKE_WIDTH;
            let painter = ui.painter();
            painter.rect_filled(rect, stroke_width, color);
            painter.rect_stroke(
                rect,
                stroke_width,
                (stroke_width, Color32::WHITE),
                StrokeKind::Inside,
            );
        }

        result
    }

    fn render_end_locations(state: &AppState, ui: &mut Ui) -> EndResult {
        let mut result = EndResult::default();

        let end_locations = match state.end_locations() {
            Some(end_locations) => end_locations,
            None => return result,
        };

        // 終了時位置を描画（丸）
        for (player_id, point) in end_locations {
            let player = match state.player(player_id) {
                Some(p) => p,
                None => continue,
            };

            let ui_rect = ui.max_rect();
            let size = LocationConstants::END_LOCATION_SIZE;
            let radius = size / 2.0;

            // マージンを計算（上：ラベル分、左右：サイズの倍、下：サイズ分）
            let margin_top = radius
                + LocationConstants::END_LOCATION_LABEL_Y_OFFSET
                + LocationConstants::END_LOCATION_LABEL_FONT_SIZE
                + AppConstants::COM_BG_LABEL_PADDING_Y * 2.0;
            let margin_horizontal = size * 2.0;
            let margin_bottom = size;

            // 中心座標を計算してクランプ
            let center = Self::calculate_clamped_position(
                &point,
                ui_rect,
                margin_top,
                margin_horizontal,
                margin_bottom,
            );

            // 描画領域を確保してResponseを取得
            let rect = Rect::from_center_size(center, Vec2::new(size, size));
            let response = ui.allocate_rect(rect, Sense::click_and_drag());

            // イベント処理
            if response.clicked() {
                result.click_player_id = Some(player_id);
            }
            if response.drag_started() {
                result.drag_start_player_id = Some(player_id);
            }
            if response.dragged() {
                result.dragging_player_id = Some(player_id);
            }
            if response.drag_stopped() {
                result.drag_stop_player_id = Some(player_id);
            }

            let painter = ui.painter();
            Self::render_mark(&player, painter, center, size / 2.0);
            Self::render_dead_mark(&player, painter, center);
            Self::render_ejected_mark(&player, painter, center);
            Self::render_label(ui, &player, center, size / 2.0);
        }

        result
    }

    fn render_mark(player: &player::Player, painter: &Painter, pos: Pos2, radius: f32) {
        let mut color = player.color.to_egui_color();

        // 追放されたら不透明度を半分にする
        if player.is_ejected() {
            color = Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 128);
        }

        let stroke_width = LocationConstants::END_LOCATION_STROKE_WIDTH;

        painter.circle_filled(pos, radius, color);
        painter.circle_stroke(pos, radius, (stroke_width, Color32::WHITE));
    }

    // 死亡している場合はバツ印を描画
    fn render_dead_mark(player: &player::Player, painter: &Painter, pos: Pos2) {
        if !player.is_dead() {
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
            (cross_width, Color32::DARK_RED),
        );

        // 右上から左下への線
        painter.line_segment(
            [
                pos2(pos.x + cross_size, pos.y - cross_size),
                pos2(pos.x - cross_size, pos.y + cross_size),
            ],
            (cross_width, Color32::DARK_RED),
        );
    }

    // 追放されたら中央に横線を描画
    fn render_ejected_mark(player: &player::Player, painter: &Painter, pos: Pos2) {
        if !player.is_ejected() {
            return;
        }

        let cross_size = LocationConstants::END_LOCATION_DEAD_MARK_SIZE;
        let cross_width = LocationConstants::END_LOCATION_DEAD_MARK_STROKE_WIDTH;

        // 中央左から右への線
        painter.line_segment(
            [
                pos2(pos.x - cross_size, pos.y),
                pos2(pos.x + cross_size, pos.y),
            ],
            (cross_width, Color32::BLACK),
        );
    }

    /// 正規化座標から画面上の中心座標を計算し、マージンを考慮してクランプする
    fn calculate_clamped_position(
        point: &crate::models::point::Point,
        ui_rect: Rect,
        margin_top: f32,
        margin_horizontal: f32,
        margin_bottom: f32,
    ) -> Pos2 {
        // 生の中心座標を計算
        let raw_pos = pos2(
            ui_rect.min.x + point.x * ui_rect.size().x,
            ui_rect.min.y + point.y * ui_rect.size().y,
        );

        // マージンを適用してクランプ
        let clamped_x = raw_pos.x.clamp(
            ui_rect.min.x + margin_horizontal,
            ui_rect.max.x - margin_horizontal,
        );
        let clamped_y = raw_pos
            .y
            .clamp(ui_rect.min.y + margin_top, ui_rect.max.y - margin_bottom);
        pos2(clamped_x, clamped_y)
    }

    fn render_label(ui: &mut Ui, player: &player::Player, center: Pos2, radius: f32) {
        let font_size = LocationConstants::END_LOCATION_LABEL_FONT_SIZE;

        // 名前を最大5文字に制限
        let display_name: String = player.name.chars().take(5).collect();

        let galley = ui.painter().layout_no_wrap(
            display_name.clone(),
            FontId::proportional(font_size),
            Color32::WHITE,
        );

        let padding = vec2(
            AppConstants::COM_BG_LABEL_PADDING_X,
            AppConstants::COM_BG_LABEL_PADDING_Y,
        );
        let size = galley.size() + padding * 2.0;

        let label_pos = Pos2::new(
            center.x,
            center.y - radius - LocationConstants::END_LOCATION_LABEL_Y_OFFSET,
        );
        let rect = Rect::from_center_size(label_pos, size);

        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            background_label_with_font_size(ui, &display_name, font_size);
        });
    }
}
