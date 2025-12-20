use crate::models::{Color, Player, PlayerId, PlayerState, Role};

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

    pub fn players(&self) -> Option<Vec<Player>> {
        self.game().map(|game| game.players.clone())
    }

    pub fn player(&self, player_id: PlayerId) -> Option<Player> {
        let players = self.players()?;
        players.into_iter().find(|p| p.id == player_id)
    }

    pub(super) fn player_mut(&mut self, player_id: PlayerId) -> Option<&mut Player> {
        let game = self.data.game.as_mut()?;
        game.players.iter_mut().find(|p| p.id == player_id)
    }

    pub fn player_name_editing(&self, player_id: PlayerId) -> bool {
        self.data.editing_name_player_id == Some(player_id)
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
    pub fn try_update_player_color(&mut self, id: PlayerId, color: Color) -> Result<(), String> {
        // setup_state での重複チェック
        let is_duplicate = self
            .players()
            .unwrap_or_default()
            .iter()
            .any(|p| p.id != id && p.color == color);

        if is_duplicate {
            return Err(format!("色 {} は既に使用されています", color.name()));
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
