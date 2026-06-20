use crate::{
    components::{background_label, change_color, tile, ChangeColorConf, TileConf},
    features::player_info::constants::PlayerInfoConstants,
    i18n::keys::{PLAYER_INFO_BUTTON_DONE, PLAYER_INFO_BUTTON_NOT_DONE},
    models::{color::Color, Player},
    state::{PlayerInfoAction, Slices},
};
use egui::*;
use crate::common::ui_color_adapter::to_egui_color;

pub struct PlayerInfoView;

impl PlayerInfoView {
    pub fn render(slices: &Slices<'_>, ui: &mut Ui) -> Vec<PlayerInfoAction> {
        let mut actions = Vec::new();

        let player_slice = slices.player();
        let players = match player_slice.players() {
            Some(p) => p,
            None => return actions,
        };

        let texts = PlayerInfoText::get(slices);

        ui.vertical(|ui| {
            for player in players.iter() {
                actions.extend(Self::render_player(slices, ui, player, &texts));
            }
        });
        actions
    }

    fn render_player(
        slices: &Slices<'_>,
        ui: &mut Ui,
        player: &Player,
        texts: &PlayerInfoText,
    ) -> Vec<PlayerInfoAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            ui.add_space(8.0);
            if let Some(color) = Self::render_player_color_box(slices, ui, player) {
                actions.push(PlayerInfoAction::ChangeColor(player.id, color));
            }

            if let Some(action) = Self::render_button(ui, player, texts) {
                actions.push(action);
            }

            actions.extend(Self::render_player_name(slices, ui, player));
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
    fn render_player_color_box(slices: &Slices<'_>, ui: &mut Ui, player: &Player) -> Option<Color> {
        let tile_result = tile(
            ui,
            &TileConf {
                color: to_egui_color(&player.color),
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
                slices,
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
        slices: &Slices<'_>,
        ui: &mut Ui,
        player: &Player,
    ) -> Vec<PlayerInfoAction> {
        let mut actions = Vec::new();

        if !slices.player().is_editing_name(player.id) {
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
    fn get(slices: &Slices<'_>) -> Self {
        Self {
            button_done: slices.t(PLAYER_INFO_BUTTON_DONE),
            button_not_done: slices.t(PLAYER_INFO_BUTTON_NOT_DONE),
        }
    }
}
