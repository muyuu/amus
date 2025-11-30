use crate::{
    components::text::background_label, constants::AppConstants, models::User, state::AppState,
};
use egui::*;

#[derive(Default, PartialEq)]
pub struct UserInfoResult {
    pub any_name_double_clicked: bool,
    pub double_clicked_user_id: Option<usize>,

    pub any_name_changed: bool,
    pub changed_new_name: Option<(String, usize)>,

    pub any_lost_focus: bool,
    pub lost_focus_user_id: Option<usize>,
}

pub struct UserInfoView;

impl UserInfoView {
    pub fn render(state: &AppState, ui: &mut Ui) -> UserInfoResult {
        let mut result = UserInfoResult::default();

        ui.vertical(|ui| {
            let players = state.setup_state().players.clone();
            for (user_id, user) in players.iter().enumerate() {
                ui.horizontal(|ui| {
                    Self::render_user_color_box(ui, user);

                    // TODO: ボタンを押したか否かのアイコンを表示

                    let r = Self::render_player_name(state, ui, user_id, user);
                    if r != UserInfoResult::default() {
                        result = r;
                    }
                });
            }
        });
        result
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
    fn render_player_name(
        state: &AppState,
        ui: &mut Ui,
        user_id: usize,
        user: &User,
    ) -> UserInfoResult {
        let mut result = UserInfoResult::default();

        if !state.user_name_editing(user_id) {
            let res = background_label(ui, &user.name).double_clicked();
            if res {
                result.any_name_double_clicked = true;
                result.double_clicked_user_id = Some(user_id);
            }
            return result;
        }

        // 編集中
        let mut editing_text = user.name.clone();
        let res = ui.text_edit_singleline(&mut editing_text);
        if res.changed() {
            result.any_name_changed = true;
            result.changed_new_name = Some((editing_text.clone(), user_id));
        }
        if res.lost_focus() {
            result.any_lost_focus = true;
            result.lost_focus_user_id = Some(user_id);
        }
        result
    }
}
