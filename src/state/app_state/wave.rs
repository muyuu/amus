use crate::models::{Point, Route, Wave};

use super::AppState;

// ウェーブとルート管理関連
impl AppState {
    /// 現在の wave への不変参照を取得
    pub fn current_wave(&self) -> Result<&Wave, String> {
        let wave_index = self.data.current_wave_index;
        let game = self.data.game.as_ref().ok_or("Game not found")?;
        game.get_wave(wave_index)
            .ok_or_else(|| "Wave not found".to_string())
    }

    /// 現在の wave への可変参照を取得
    pub(super) fn current_wave_mut(&mut self) -> Result<&mut Wave, String> {
        let wave_index = self.data.current_wave_index;
        let game = self.data.game.as_mut().ok_or("Game not found")?;
        game.get_wave_mut(wave_index)
            .ok_or_else(|| "Wave not found".to_string())
    }

    pub fn routes(&self) -> Option<Vec<Route>> {
        self.current_wave().ok().map(|wave| wave.routes.clone())
    }

    pub fn push_route(&mut self, route: Route) {
        let wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
        };

        wave.routes.push(route);
    }

    pub fn add_point_to_last_route(&mut self, point: Point) {
        let wave = match self.current_wave_mut() {
            Ok(wave) => wave,
            _ => return,
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
