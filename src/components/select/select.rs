use egui::*;

use super::base::{base, SelectConf};

/// ベクタ版
pub fn select_with_options<T: PartialEq + Clone>(
    ui: &mut Ui,
    id: impl Into<String>,
    selected: &mut T,
    options: Vec<(T, String)>,
) -> Option<T> {
    let selected_label = options
        .iter()
        .find(|(val, _)| *val == *selected)
        .map(|(_, label)| label.clone())
        .unwrap_or_else(|| "<未選択>".to_string());

    let mut changed_value: Option<T> = None;

    base(
        ui,
        SelectConf {
            id: id.into(),
            selected_text: selected_label,
            width: None,
        },
        |ui| {
            for (val, label) in &options {
                if ui.selectable_value(selected, val.clone(), label).changed() {
                    changed_value = Some(val.clone());
                }
            }
        },
    );

    changed_value
}

/// クロージャ版
pub fn select_with_contents<R>(
    ui: &mut Ui,
    id: impl Into<String>,
    selected_text: impl Into<String>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    base(
        ui,
        SelectConf {
            id: id.into(),
            selected_text: selected_text.into(),
            width: None,
        },
        add_contents,
    )
}
