use egui::*;

use super::base::base;
use super::base::GridConf;
use super::constants::GridConstants;

pub fn grid<R>(ui: &mut Ui, id: impl Into<String>, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let conf = GridConf {
        id: id.into(),
        spacing: GridConstants::SPACING.to_vec(),

        // 制限なし
        num_columns: None,
    };

    base(ui, conf, add_contents)
}
