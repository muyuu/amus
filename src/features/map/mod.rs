mod view;

use crate::app_action::AppAction;
use crate::state::Slices;
use egui::{Response, Ui};
use view::MapView;

pub struct MapFeature;

impl MapFeature {
    // 描画のみ（操作なし）だが、規約に合わせ Vec<AppAction>（空）を返す
    pub fn render(slices: &Slices, response: &Response, ui: &mut Ui) -> Vec<AppAction> {
        MapView::render(slices, response, ui);
        Vec::new()
    }
}
