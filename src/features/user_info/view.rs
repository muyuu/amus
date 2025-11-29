use crate::{
    components::text::background_label, constants::AppConstants, models::User, state::AppState,
};
use egui::*;

pub struct UserInfoView;

impl UserInfoView {
    pub fn render(state: &AppState, ui: &mut Ui) {
        ui.vertical(|ui| {
            let players = state.setup_state().players.clone();
            for (user_id, user) in players.iter().enumerate() {
                ui.horizontal(|ui| {
                    Self::render_user_color_box(ui, user);

                    // TODO: ボタンを押したか否かのアイコンを表示

                    Self::render_player_name(state, ui, user_id, user);
                });
            }
        });
    }

    // ユーザー色の矩形を描画
    fn render_user_color_box(ui: &mut Ui, user: &User) {
        let rect_size = Vec2::new(
            AppConstants::USER_INFO_COLOR_BOX_SIZE,
            AppConstants::USER_INFO_COLOR_BOX_SIZE,
        );
        let (rect, _) = ui.allocate_exact_size(rect_size, Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, user.color.to_egui_color());
    }

    /// ユーザー名を描画または編集
    fn render_player_name(state: &AppState, ui: &mut Ui, user_id: usize, user: &User) {
        if !state.user_name_editing(user_id) {
            let res = background_label(ui, &user.name).double_clicked();
            if res {
                state.toggle_user_name_editing(user_id);
            }
            return;
        }

        // 編集中
        let mut editing_text = user.name.clone();
        let res = ui.text_edit_singleline(&mut editing_text);
        if res.changed() {
            state.update_player_name(user_id, editing_text.clone());
        }
        if res.lost_focus() {
            state.toggle_user_name_editing(user_id);
        }
    }
}
