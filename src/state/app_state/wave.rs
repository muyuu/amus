use crate::models::{Point, Route, Wave};

use super::AppState;

// ウェーブとルート管理関連
impl AppState {
    /// 現在の wave への可変参照を取得（game 未作成・wave 未選択なら `None`）
    pub(super) fn current_wave_mut(&mut self) -> Option<&mut Wave> {
        let wave_index = self.data.current_wave_index;
        let game = self.data.game.as_mut()?;
        game.get_wave_mut(wave_index)
    }

    pub fn push_route(&mut self, route: Route) {
        let Some(wave) = self.current_wave_mut() else {
            return;
        };

        wave.routes.push(route);
    }

    pub fn add_point_to_last_route(&mut self, point: Point) {
        let Some(wave) = self.current_wave_mut() else {
            return;
        };

        let last_route = match wave.routes.last_mut() {
            Some(r) => r,
            None => return,
        };

        let lines = last_route.lines_mut();
        let last_line = match lines.last_mut() {
            Some(line) => line,
            None => return,
        };

        // 直前のポイントと同じ場合は追加しない
        let last_point = last_line.last();
        if let Some(lp) = last_point {
            if lp.x == point.x && lp.y == point.y {
                return;
            }
        }
        last_line.push(point);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::route::Draw;

    #[test]
    fn current_wave_errors_without_game() {
        let state = AppState::new();
        assert!(state.slices().wave().current_wave().is_none());
    }

    #[test]
    fn push_route_appends_to_current_wave() {
        let mut state = AppState::new();
        state.create_game_from_setup();
        let player_id = state.slices().player().players().unwrap()[0].id;
        assert_eq!(state.slices().wave().routes().unwrap().len(), 0);

        state.push_route(Route::Draw(Draw::new(player_id)));

        assert_eq!(state.slices().wave().routes().unwrap().len(), 1);
    }

    #[test]
    fn push_route_targets_selected_wave() {
        let mut state = AppState::new();
        state.create_game_from_setup();
        let player_id = state.slices().player().players().unwrap()[0].id;

        state.select_wave(2);
        state.push_route(Route::Draw(Draw::new(player_id)));

        // wave 2 にだけ route が入る
        assert_eq!(state.slices().wave().routes().unwrap().len(), 1);
        state.select_wave(0);
        assert_eq!(state.slices().wave().routes().unwrap().len(), 0);
    }

    #[test]
    fn add_point_to_last_route_skips_duplicate_point() {
        let mut state = AppState::new();
        state.create_game_from_setup();
        let player_id = state.slices().player().players().unwrap()[0].id;
        state.push_route(Route::Draw(Draw::new(player_id)));

        state.add_point_to_last_route(Point::new(1.0, 1.0));
        // 直前と同じ点は追加されない
        state.add_point_to_last_route(Point::new(1.0, 1.0));
        state.add_point_to_last_route(Point::new(2.0, 2.0));

        let slices = state.slices();
        let routes = slices.wave().routes().unwrap();
        let Route::Draw(draw) = &routes[0] else {
            panic!("Draw のはず");
        };
        // 最初の line に 2 点だけ入る
        assert_eq!(draw.lines[0].len(), 2);
    }
}
