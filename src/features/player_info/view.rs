use crate::{
    components::{
        background_label,
        change_color::{change_color, ChangeColorConf},
        tile, TileConf,
    },
    features::player_info::constants::PlayerInfoConstants,
    i18n::keys::{PLAYER_INFO_BUTTON_DONE, PLAYER_INFO_BUTTON_NOT_DONE},
    models::{color::Color, player::PlayerId, Player},
    state::AppState,
};
use egui::*;

#[derive(Default, PartialEq)]
pub struct PlayerInfoResult {
    pub any_name_double_clicked: bool,
    pub double_clicked_player_id: Option<PlayerId>,

    pub any_name_changed: bool,
    pub changed_new_name: Option<(String, PlayerId)>,

    pub any_lost_focus: bool,
    pub lost_focus_player_id: Option<PlayerId>,

    pub any_done_button_clicked: bool,
    pub done_button_clicked_player_id: Option<PlayerId>,
    pub done_button_new_state: Option<bool>,

    pub color_change: Option<(PlayerId, Color)>,
}

pub struct PlayerInfoView;

impl PlayerInfoView {
    pub fn render(state: &AppState, ui: &mut Ui) -> PlayerInfoResult {
        let mut result = PlayerInfoResult::default();

        let players = match state.players() {
            Some(p) => p,
            None => return result,
        };

        ui.vertical(|ui| {
            for player in players.iter() {
                let r = Self::render_player(state, ui, player);
                if r != PlayerInfoResult::default() {
                    result = r;
                }
            }
        });
        result
    }

    fn render_player(state: &AppState, ui: &mut Ui, player: &Player) -> PlayerInfoResult {
        let mut result: PlayerInfoResult = PlayerInfoResult::default();
        let texts = PlayerInfoText::get(state);

        ui.horizontal(|ui| {
            ui.add_space(8.0);
            let color_change = Self::render_player_color_box(state, ui, player);
            if let Some(color) = color_change {
                result.color_change = Some((player.id, color));
            }

            let (clicked, next) = Self::render_button(ui, player, &texts);
            if clicked {
                result.any_done_button_clicked = true;
                result.done_button_clicked_player_id = Some(player.id);
                result.done_button_new_state = Some(next);
            }

            let r = Self::render_player_name(state, ui, player);
            if r != PlayerInfoResult::default() {
                result = r;
            }
        });
        result
    }

    fn render_button(ui: &mut Ui, player: &Player, texts: &PlayerInfoText) -> (bool, bool) {
        if player.done_button {
            let res = ui.button(&texts.button_done).clicked();
            return (res, false);
        }

        let text = RichText::new(&texts.button_not_done)
            .strong()
            .color(Color32::WHITE);
        let button = Button::new(text)
            .fill(Color32::RED)
            .stroke(Stroke::new(2.0, Color32::BROWN));
        let res = ui.add(button).clicked();
        (res, true)
    }

    // プレイヤー色の矩形を描画
    fn render_player_color_box(state: &AppState, ui: &mut Ui, player: &Player) -> Option<Color> {
        let tile_result = tile(
            ui,
            &TileConf {
                color: player.color.to_egui_color(),
                label: None,
                size: Some(Vec2::splat(PlayerInfoConstants::COLOR_BOX_SIZE)),
                is_selected: false,
                is_dragging: false,
            },
        );

        let mut color_change = None;
        tile_result.response.context_menu(|ui| {
            let result = change_color(
                ui,
                state,
                &ChangeColorConf {
                    player_id: player.id,
                    show_delete: false,
                    grid_id_suffix: format!("player_info_{:?}", player.id),
                },
            );

            if let Some(color) = result.color_change {
                color_change = Some(color);
            }
        });

        color_change
    }

    /// プレイヤー名を描画または編集
    fn render_player_name(state: &AppState, ui: &mut Ui, player: &Player) -> PlayerInfoResult {
        let mut result = PlayerInfoResult::default();

        if !state.player_name_editing(player.id) {
            let res = background_label(ui, &player.name).double_clicked();
            if res {
                result.any_name_double_clicked = true;
                result.double_clicked_player_id = Some(player.id);
            }
            return result;
        }

        // 編集中
        let mut editing_text = player.name.clone();
        let res = ui.text_edit_singleline(&mut editing_text);
        if res.changed() {
            result.any_name_changed = true;
            result.changed_new_name = Some((editing_text.clone(), player.id));
        }
        if res.lost_focus() {
            result.any_lost_focus = true;
            result.lost_focus_player_id = Some(player.id);
        }
        result
    }
}

struct PlayerInfoText {
    button_done: String,
    button_not_done: String,
}
impl PlayerInfoText {
    fn get(state: &AppState) -> Self {
        Self {
            button_done: state.t(PLAYER_INFO_BUTTON_DONE).to_string(),
            button_not_done: state.t(PLAYER_INFO_BUTTON_NOT_DONE).to_string(),
        }
    }
}
