use crate::models::{Point, User, Wave};
use crate::state::{DraggingLocation, LocationType};

pub struct LocationInteraction;

impl LocationInteraction {
    /// スクリーン座標を正規化座標（0.0-1.0）に変換
    pub fn screen_to_normalized_point(screen_pos: egui::Pos2, rect: egui::Rect) -> Point {
        Point::new(
            (screen_pos.x - rect.min.x) / rect.size().x,
            (screen_pos.y - rect.min.y) / rect.size().y,
        )
    }

    /// 終了位置のクリック検出と状態トグル
    pub fn handle_end_location_click(
        users: &mut [User],
        wave: &Wave,
        response: &egui::Response,
    ) -> bool {
        if !response.clicked() {
            return false;
        }

        let pointer_pos = match response.interact_pointer_pos() {
            Some(pos) => pos,
            None => return false,
        };

        let hit_size = 24.0;

        // 終了時位置をチェック
        for (user_id, end_point) in &wave.end_locations {
            let end_pos = egui::pos2(
                response.rect.min.x + end_point.x * response.rect.size().x,
                response.rect.min.y + end_point.y * response.rect.size().y,
            );
            let distance = (pointer_pos - end_pos).length();
            if distance <= hit_size {
                // ユーザーの生存状態をトグル
                if let Some(user) = users.get_mut(*user_id) {
                    user.alive = !user.alive;
                    return true;
                }
            }
        }

        false
    }

    /// ドラッグ開始の検出（出現位置または終了時位置の上でドラッグ開始）
    pub fn detect_drag_start(wave: &Wave, response: &egui::Response) -> Option<DraggingLocation> {
        let pointer_pos = response.interact_pointer_pos()?;
        let hit_size = 24.0;

        // 出現位置をチェック
        if let Some(location) =
            Self::find_hit_spawn_location(wave, pointer_pos, response.rect, hit_size)
        {
            return Some(location);
        }

        // 終了時位置をチェック
        Self::find_hit_end_location(wave, pointer_pos, response.rect, hit_size)
    }

    /// 出現位置のヒット判定
    fn find_hit_spawn_location(
        wave: &Wave,
        pointer_pos: egui::Pos2,
        rect: egui::Rect,
        hit_size: f32,
    ) -> Option<DraggingLocation> {
        for (user_id, spawn_point) in &wave.spawn_locations {
            let spawn_pos = egui::pos2(
                rect.min.x + spawn_point.x * rect.size().x,
                rect.min.y + spawn_point.y * rect.size().y,
            );
            let distance = (pointer_pos - spawn_pos).length();
            if distance <= hit_size {
                return Some(DraggingLocation {
                    location_type: LocationType::Spawn,
                    user_id: *user_id,
                });
            }
        }
        None
    }

    /// 終了時位置のヒット判定
    fn find_hit_end_location(
        wave: &Wave,
        pointer_pos: egui::Pos2,
        rect: egui::Rect,
        hit_size: f32,
    ) -> Option<DraggingLocation> {
        for (user_id, end_point) in &wave.end_locations {
            let end_pos = egui::pos2(
                rect.min.x + end_point.x * rect.size().x,
                rect.min.y + end_point.y * rect.size().y,
            );
            let distance = (pointer_pos - end_pos).length();
            if distance <= hit_size {
                return Some(DraggingLocation {
                    location_type: LocationType::End,
                    user_id: *user_id,
                });
            }
        }
        None
    }

    /// ドラッグ中の位置を更新
    pub fn update_dragging_location(
        wave: &mut Wave,
        dragging_location: DraggingLocation,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        let pointer_pos = match ctx.pointer_latest_pos() {
            Some(pos) => pos,
            None => return,
        };

        if !response.rect.contains(pointer_pos) {
            return;
        }

        let point = Self::screen_to_normalized_point(pointer_pos, response.rect);

        match dragging_location.location_type {
            LocationType::Spawn => {
                wave.spawn_locations
                    .insert(dragging_location.user_id, point);
            }
            LocationType::End => {
                wave.end_locations.insert(dragging_location.user_id, point);
            }
        }
    }

    /// ユーザーをドラッグ&ドロップした時の処理（終了時位置を設定）
    pub fn handle_user_drop(
        wave: &mut Wave,
        dragging_user_id: usize,
        response: &egui::Response,
        ctx: &egui::Context,
    ) {
        let pointer_pos = match ctx.pointer_latest_pos() {
            Some(pos) => pos,
            None => return,
        };

        if !response.rect.contains(pointer_pos) {
            return;
        }

        let point = Self::screen_to_normalized_point(pointer_pos, response.rect);
        wave.end_locations.insert(dragging_user_id, point);
    }

    /// 出現位置を設定
    pub fn set_spawn_location(wave: &mut Wave, user_id: usize, point: Point) {
        // すでに出現位置を持ってる場合は何もしない
        // すでに置いた出現位置を削除する必要はないし、位置変更はドラッグのみで実施
        if wave.spawn_locations.contains_key(&user_id) {
            return;
        }

        wave.spawn_locations.insert(user_id, point);
    }
}
