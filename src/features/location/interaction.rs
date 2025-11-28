use crate::models::Point;
use crate::state::{AppState, DraggingLocation, LocationType};
use egui::*;

pub struct LocationInteraction;

impl LocationInteraction {
    pub fn handle_interactions(state: &mut AppState, response: &Response, ctx: &Context) {
        Self::handle_end_location_click(state, response);

        // ユーザードロップ処理（必要な値を先に取得）
        Self::handle_user_drop(state, response, ctx);

        // エリアクリック処理（必要な値を先に取得）
        Self::handle_area_click(state, response);

        // ドラッグ終了時のクリア
        Self::clear_drag_states(state, ctx);

        // ドラッグ開始の検出
        Self::handle_drag_start(state, response, ctx);
    }

    fn handle_drag_start(state: &mut AppState, response: &egui::Response, ctx: &egui::Context) {
        // 位置（出現位置・終了時位置）のドラッグ処理
        if let Some(dragging_location) = state.dragging_location() {
            Self::update_dragging_location(state, dragging_location, response, ctx);
        }

        // ドラッグ開始の検出
        if response.drag_started() && state.dragging_location().is_none() {
            if let Some(location) = Self::detect_drag_start(state, response) {
                state.set_dragging_location(Some(location));
            }
        }
    }

    /// エリアクリック処理（出現位置の設定）
    fn handle_area_click(state: &mut AppState, response: &egui::Response) {
        if state.dragging_location().is_some() {
            return;
        }

        if !response.clicked() {
            return;
        }

        let selected_user_id = match state.selected_user_id() {
            Some(id) => id,
            None => return,
        };

        let pos = match response.interact_pointer_pos() {
            Some(pos) => pos,
            None => return,
        };

        let point = Self::screen_to_normalized_point(pos, response.rect);
        Self::set_spawn_location(state, selected_user_id, point);
    }

    /// ドラッグ終了時の状態クリア
    fn clear_drag_states(state: &mut AppState, ctx: &egui::Context) {
        let has_dragging_user = state.dragging_user_id().is_some();
        let has_dragging_location = state.dragging_location().is_some();
        let pointer_released = ctx.input(|i| i.pointer.any_released());

        if has_dragging_user && pointer_released {
            state.set_dragging_user_id(None);
        }
        if has_dragging_location && pointer_released {
            state.set_dragging_location(None);
        }
    }

    /// スクリーン座標を正規化座標（0.0-1.0）に変換
    pub fn screen_to_normalized_point(screen_pos: egui::Pos2, rect: egui::Rect) -> Point {
        Point::new(
            (screen_pos.x - rect.min.x) / rect.size().x,
            (screen_pos.y - rect.min.y) / rect.size().y,
        )
    }

    /// 終了位置のクリック検出と状態トグル
    pub fn handle_end_location_click(state: &mut AppState, response: &egui::Response) -> bool {
        if !response.clicked() {
            return false;
        }

        let pointer_pos = match response.interact_pointer_pos() {
            Some(pos) => pos,
            None => return false,
        };

        let hit_size = 24.0;

        // 終了時位置をチェックして、該当する user_id を先に取得
        let target_user_id = {
            let wave = match state.current_wave() {
                Ok(wave) => wave,
                Err(_) => return false,
            };

            let mut target_user_id = None;
            for (user_id, end_point) in wave.end_locations.iter() {
                let end_pos = egui::pos2(
                    response.rect.min.x + end_point.x * response.rect.size().x,
                    response.rect.min.y + end_point.y * response.rect.size().y,
                );
                let distance = (pointer_pos - end_pos).length();
                if distance <= hit_size {
                    target_user_id = Some(*user_id);
                    break;
                }
            }
            target_user_id
        };

        match target_user_id {
            None => false,
            Some(user_id) => {
                state.toggle_user_alive(user_id);
                true
            }
        }
    }

    /// ドラッグ開始の検出（出現位置または終了時位置の上でドラッグ開始）
    pub fn detect_drag_start(
        state: &AppState,
        response: &egui::Response,
    ) -> Option<DraggingLocation> {
        let pointer_pos = response.interact_pointer_pos()?;
        let hit_size = 24.0;

        // 出現位置をチェック
        if let Some(location) =
            Self::find_hit_spawn_location(state, pointer_pos, response.rect, hit_size)
        {
            return Some(location);
        }

        // 終了時位置をチェック
        Self::find_hit_end_location(state, pointer_pos, response.rect, hit_size)
    }

    /// 出現位置のヒット判定
    fn find_hit_spawn_location(
        state: &AppState,
        pointer_pos: egui::Pos2,
        rect: egui::Rect,
        hit_size: f32,
    ) -> Option<DraggingLocation> {
        if let Ok(wave) = state.current_wave() {
            for (user_id, spawn_point) in wave.spawn_locations.iter() {
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
        }
        None
    }

    /// 終了時位置のヒット判定
    fn find_hit_end_location(
        state: &AppState,
        pointer_pos: egui::Pos2,
        rect: egui::Rect,
        hit_size: f32,
    ) -> Option<DraggingLocation> {
        if let Ok(wave) = state.current_wave() {
            for (user_id, end_point) in wave.end_locations.iter() {
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
        }
        None
    }

    /// ドラッグ中の位置を更新
    pub fn update_dragging_location(
        state: &mut AppState,
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
                state.add_spawn_location(dragging_location.user_id, point);
            }
            LocationType::End => {
                state.add_end_location(dragging_location.user_id, point);
            }
        }
    }

    /// ユーザーをドラッグ&ドロップした時の処理（終了時位置を設定）
    pub fn handle_user_drop(state: &mut AppState, response: &egui::Response, ctx: &egui::Context) {
        let pointer_pos = match ctx.pointer_latest_pos() {
            Some(pos) => pos,
            None => return,
        };

        if !response.rect.contains(pointer_pos) {
            return;
        }

        let point = Self::screen_to_normalized_point(pointer_pos, response.rect);
        let dragging_user_id = match state.dragging_user_id() {
            Some(user_id) => user_id,
            None => return,
        };

        if let Ok(mut wave) = state.current_wave_mut() {
            wave.end_locations.insert(dragging_user_id, point);
        }
    }

    /// 出現位置を設定
    pub fn set_spawn_location(state: &AppState, user_id: usize, point: Point) {
        if state.spawn_location(user_id).is_none() {
            state.add_spawn_location(user_id, point);
        }
    }
}
