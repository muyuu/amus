use crate::{
    components::{background_label, change_color, tile, ChangeColorConf, TileConf},
    features::player_info::constants::PlayerInfoConstants,
    i18n::keys::{PLAYER_INFO_BUTTON_DONE, PLAYER_INFO_BUTTON_NOT_DONE},
    models::{color::Color, Player},
    state::{AppState, PlayerInfoAction},
};
use egui::*;

pub struct PlayerInfoView;

impl PlayerInfoView {
    pub fn render(state: &AppState, ui: &mut Ui) -> Vec<PlayerInfoAction> {
        let mut actions = Vec::new();

        let players = match state.players() {
            Some(p) => p,
            None => return actions,
        };

        ui.vertical(|ui| {
            for player in players.iter() {
                actions.extend(Self::render_player(state, ui, player));
            }
        });
        actions
    }

    fn render_player(state: &AppState, ui: &mut Ui, player: &Player) -> Vec<PlayerInfoAction> {
        let mut actions = Vec::new();
        let texts = PlayerInfoText::get(state);

        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if let Some(color) = Self::render_player_color_box(state, ui, player) {
                actions.push(PlayerInfoAction::ChangeColor(player.id, color));
            }

            if let Some(action) = Self::render_button(ui, player, &texts) {
                actions.push(action);
            }

            actions.extend(Self::render_player_name(state, ui, player));
        });
        actions
    }

    fn render_button(
        ui: &mut Ui,
        player: &Player,
        texts: &PlayerInfoText,
    ) -> Option<PlayerInfoAction> {
        if player.done_button {
            if ui.button(&texts.button_done).clicked() {
                return Some(PlayerInfoAction::ToggleDoneButton(player.id));
            }
            return None;
        }

        let text = RichText::new(&texts.button_not_done)
            .strong()
            .color(Color32::WHITE);
        let button = Button::new(text)
            .fill(Color32::RED)
            .stroke(Stroke::new(2.0, Color32::BROWN));
        if ui.add(button).clicked() {
            return Some(PlayerInfoAction::ToggleDoneButton(player.id));
        }
        None
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
    fn render_player_name(
        state: &AppState,
        ui: &mut Ui,
        player: &Player,
    ) -> Vec<PlayerInfoAction> {
        let mut actions = Vec::new();

        if !state.player_name_editing(player.id) {
            if background_label(ui, &player.name).double_clicked() {
                actions.push(PlayerInfoAction::StartEditingName(player.id));
            }
            return actions;
        }

        // 編集中
        let mut editing_text = player.name.clone();
        let res = ui.text_edit_singleline(&mut editing_text);
        if res.changed() {
            actions.push(PlayerInfoAction::UpdateName(player.id, editing_text));
        }
        if res.lost_focus() {
            actions.push(PlayerInfoAction::StopEditingName(player.id));
        }
        actions
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
