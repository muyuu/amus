mod view;

use egui::Context;
use view::SettingsView;

use crate::app_action::AppAction;
use crate::state::Slices;

pub struct SettingsFeature;

impl SettingsFeature {
    /// ゲーム画面の上に重ねて描画する（画面全体を覆わない）。
    /// 軌跡の太さなど、背後の描画を見ながら変更したい設定があるため。
    pub fn render(slices: &Slices, ctx: &Context) -> Vec<AppAction> {
        SettingsView::render(slices, ctx)
            .into_iter()
            .map(AppAction::Settings)
            .collect()
    }
}
