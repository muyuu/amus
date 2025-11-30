use crate::{
    components::text::background_label, constants::AppConstants, models::Player, state::AppState,
};
use egui::*;

#[derive(Default, PartialEq)]
pub struct PlayerInfoResult {
    pub any_name_double_clicked: bool,
    pub double_clicked_player_id: Option<usize>,

    pub any_name_changed: bool,
    pub changed_new_name: Option<(String, usize)>,

    pub any_lost_focus: bool,
    pub lost_focus_player_id: Option<usize>,
}

pub struct PlayerInfoView;

impl PlayerInfoView {
    pub fn render(state: &AppState, ui: &mut Ui) -> PlayerInfoResult {
        let mut result = PlayerInfoResult::default();

        ui.vertical(|ui| {
            let players = state.setup_state().players.clone();
            for (player_id, player) in players.iter().enumerate() {
                ui.horizontal(|ui| {
                    Self::render_player_color_box(ui, player);

                    // TODO: ボタンを押したか否かのアイコンを表示

                    let r = Self::render_player_name(state, ui, player_id, player);
                    if r != PlayerInfoResult::default() {
                        result = r;
                    }
                });
            }
        });
        result
    }

    // プレイヤー色の矩形を描画
    fn render_player_color_box(ui: &mut Ui, player: &Player) {
        let rect_size = Vec2::new(
            AppConstants::PLAYER_INFO_COLOR_BOX_SIZE,
            AppConstants::PLAYER_INFO_COLOR_BOX_SIZE,
        );
        let (rect, _) = ui.allocate_exact_size(rect_size, Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, player.color.to_egui_color());
    }

    /// プレイヤー名を描画または編集
    fn render_player_name(
        state: &AppState,
        ui: &mut Ui,
        player_index: usize,
        player: &Player,
    ) -> PlayerInfoResult {
        let mut result = PlayerInfoResult::default();

        if !state.player_name_editing(player_index) {
            let res = background_label(ui, &player.name).double_clicked();
            if res {
                result.any_name_double_clicked = true;
                result.double_clicked_player_id = Some(player_index);
            }
            return result;
        }

        // 編集中
        let mut editing_text = player.name.clone();
        let res = ui.text_edit_singleline(&mut editing_text);
        if res.changed() {
            result.any_name_changed = true;
            result.changed_new_name = Some((editing_text.clone(), player_index));
        }
        if res.lost_focus() {
            result.any_lost_focus = true;
            result.lost_focus_player_id = Some(player_index);
        }
        result
    }
}
