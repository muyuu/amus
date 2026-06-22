use crate::models::{Color, Player, PlayerId, PlayerState, Role};
use crate::state::error::ColorError;

use super::AppState;

// プレイヤー操作関連
impl AppState {
    pub fn toggle_player_state(&mut self, player_id: PlayerId) {
        if let Some(game) = self.data.game.as_mut() {
            game.players.iter_mut().for_each(|p| {
                if p.id == player_id {
                    let next = match p.state {
                        PlayerState::Alive => PlayerState::Killed,
                        PlayerState::Killed => PlayerState::Ejected,
                        PlayerState::Ejected => PlayerState::Alive,
                    };
                    p.state = next;
                }
            });
        }
    }

    pub fn adjust_player_count(&mut self, new_count: usize) {
        let old_count = self.data.setup_state.players.len();
        self.data.setup_state.player_count = new_count;

        if new_count > old_count {
            // プレイヤーを追加
            let available_colors = [
                Color::Red,
                Color::Blue,
                Color::Green,
                Color::Pink,
                Color::Orange,
                Color::Yellow,
                Color::Black,
                Color::White,
                Color::Purple,
                Color::Brown,
                Color::Cyan,
                Color::Lime,
                Color::Maroon,
                Color::Rose,
                Color::Banana,
                Color::Gray,
                Color::Tan,
                Color::Coral,
            ];

            for _i in old_count..new_count {
                // 現在のプレイヤーが使ってない色を選択
                let used_colors = self
                    .data
                    .setup_state
                    .players
                    .iter()
                    .map(|p| p.color.clone())
                    .collect::<Vec<Color>>();
                let color = available_colors
                    .iter()
                    .find(|c| !used_colors.contains(c))
                    .cloned()
                    .unwrap_or(Color::Red);

                self.data
                    .setup_state
                    .players
                    .push(Player::new(Role::Crew, color, "".to_string()));
            }
        }
    }

    pub(super) fn player_mut(&mut self, player_id: PlayerId) -> Option<&mut Player> {
        let game = self.data.game.as_mut()?;
        game.players.iter_mut().find(|p| p.id == player_id)
    }

    pub fn toggle_player_name_editing(&mut self, player_id: PlayerId) {
        if self.data.editing_name_player_id == Some(player_id) {
            self.data.editing_name_player_id = None;
        } else {
            self.data.editing_name_player_id = Some(player_id);
        }
    }

    pub fn update_player_name(&mut self, id: PlayerId, name: String) {
        // setup_stateを更新
        self.data.setup_state.players.iter_mut().for_each(|p| {
            if p.id == id {
                p.name = name.clone();
            }
        });

        // gameを更新
        if let Some(game) = &mut self.data.game {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.name = name.clone();
                }
            });
        }
    }

    #[allow(dead_code)]
    /// 色の変更を試みる（重複している場合は変更を拒否）
    pub fn try_update_player_color(&mut self, id: PlayerId, color: Color) -> Result<(), ColorError> {
        // 既存プレイヤーとの色重複チェック
        let is_duplicate = self
            .data
            .game
            .as_ref()
            .map(|g| g.players.iter().any(|p| p.id != id && p.color == color))
            .unwrap_or(false);

        if is_duplicate {
            return Err(ColorError::DuplicateColor(color.name().to_string()));
        }

        // 重複がない場合は更新
        self.data.setup_state.players.iter_mut().for_each(|p| {
            if p.id == id {
                p.color = color.clone();
            }
        });

        // game が存在する場合も更新
        if let Some(game) = &mut self.data.game {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.color = color.clone();
                }
            });
        }

        Ok(())
    }

    /// 色を強制的に変更する（重複している場合は他のプレイヤーと色をスワップ）
    pub fn force_update_player_color(&mut self, id: PlayerId, color: Color) {
        // 対象の色を使っている他のプレイヤーを探す
        let other_player_id = self
            .data
            .setup_state
            .players
            .iter()
            .find(|p| p.id != id && p.color == color)
            .map(|p| p.id);

        // 対象プレイヤーの現在の色を取得
        let current_color = self
            .data
            .setup_state
            .players
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.color.clone());

        // setup_state と game を更新
        self.data.setup_state.players.iter_mut().for_each(|p| {
            if p.id == id {
                // 対象プレイヤーの色を変更
                p.color = color.clone();
            } else if Some(p.id) == other_player_id {
                // 重複していたプレイヤーの色を対象プレイヤーの元の色に変更
                if let Some(ref swap_color) = current_color {
                    p.color = swap_color.clone();
                }
            }
        });

        // game が存在する場合も更新
        if let Some(game) = &mut self.data.game {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.color = color.clone();
                } else if Some(p.id) == other_player_id {
                    if let Some(ref swap_color) = current_color {
                        p.color = swap_color.clone();
                    }
                }
            });
        }
    }

    pub fn update_player_button(&mut self, id: PlayerId, done: bool) {
        // これはリセット時に初期化したいので setup_state は更新しない
        if let Some(game) = &mut self.data.game {
            game.players.iter_mut().for_each(|p| {
                if p.id == id {
                    p.done_button = done;
                }
            });
        }
    }

    pub fn toggle_comms(&mut self, id: PlayerId) {
        if let Some(player) = self.player_mut(id) {
            player.resolved_comms = !player.resolved_comms;
        }
    }

    pub fn toggle_lights(&mut self, id: PlayerId) {
        if let Some(player) = self.player_mut(id) {
            player.resolved_lights = !player.resolved_lights;
        }
    }

    pub fn toggle_o2(&mut self, id: PlayerId) {
        if let Some(player) = self.player_mut(id) {
            player.resolved_o2 = !player.resolved_o2;
        }
    }

    pub fn toggle_reactor(&mut self, id: PlayerId) {
        if let Some(player) = self.player_mut(id) {
            player.resolved_reactor = !player.resolved_reactor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ゲームを開始し、最初のプレイヤーの id を返すヘルパー
    fn game_with_players() -> (AppState, PlayerId) {
        let mut state = AppState::new();
        state.create_game_from_setup();
        let id = state.slices().player().players().expect("players")[0].id;
        (state, id)
    }

    #[test]
    fn toggle_player_state_cycles_alive_killed_ejected() {
        let (mut state, id) = game_with_players();
        assert_eq!(
            state.slices().player().player(id).unwrap().state,
            PlayerState::Alive
        );

        state.toggle_player_state(id);
        assert_eq!(
            state.slices().player().player(id).unwrap().state,
            PlayerState::Killed
        );

        state.toggle_player_state(id);
        assert_eq!(
            state.slices().player().player(id).unwrap().state,
            PlayerState::Ejected
        );

        state.toggle_player_state(id);
        assert_eq!(
            state.slices().player().player(id).unwrap().state,
            PlayerState::Alive
        );
    }

    #[test]
    fn update_player_name_updates_game_and_setup() {
        let (mut state, id) = game_with_players();

        state.update_player_name(id, "あお".to_string());

        assert_eq!(state.slices().player().player(id).unwrap().name, "あお");
        // setup_state 側も同じ id のプレイヤーが更新される
        let slices = state.slices();
        let in_setup = slices
            .setup()
            .players()
            .iter()
            .find(|p| p.id == id)
            .expect("setup にも同じ id が居る");
        assert_eq!(in_setup.name, "あお");
    }

    #[test]
    fn try_update_player_color_rejects_duplicate() {
        let (mut state, id) = game_with_players();
        // 別プレイヤーが既に使っている色を取得
        let other_color = state.slices().player().players().unwrap()[1].color.clone();

        let result = state.try_update_player_color(id, other_color.clone());

        assert!(matches!(result, Err(ColorError::DuplicateColor(_))));
        // 拒否されたので色は変わらない
        assert_ne!(
            state.slices().player().player(id).unwrap().color,
            other_color
        );
    }
}
