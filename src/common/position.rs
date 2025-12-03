use egui::*;

use crate::models::Point;

pub fn current_position_with_ctx(ctx: &Context) -> Option<Pos2> {
    ctx.pointer_latest_pos()
}

pub fn current_position_with_response(response: &Response) -> Option<Pos2> {
    response.interact_pointer_pos()
}

pub fn current_point_with_ui(ui: &mut Ui) -> Option<Point> {
    current_position_with_ctx(ui.ctx()).map(|pos| screen_to_normalized_point(pos, ui.max_rect()))
}

pub fn current_point_with_response(response: &Response, rect: Rect) -> Option<Point> {
    current_position_with_response(response).map(|pos| screen_to_normalized_point(pos, rect))
}

pub fn screen_to_normalized_point(screen_pos: Pos2, rect: Rect) -> Point {
    Point::new(
        (screen_pos.x - rect.min.x) / rect.size().x,
        (screen_pos.y - rect.min.y) / rect.size().y,
    )
}
