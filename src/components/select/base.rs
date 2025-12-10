use egui::*;

pub struct SelectConf {
    pub id: String,
    pub selected_text: String,
    pub width: Option<f32>,
}

/// 共通処理: ComboBox を生成して show_ui を呼ぶ
pub fn base<R>(
    ui: &mut Ui,
    conf: SelectConf,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let mut combo = ComboBox::from_id_salt(conf.id).selected_text(conf.selected_text);

    if let Some(w) = conf.width {
        combo = combo.width(w);
    }

    combo.show_ui(ui, add_contents).inner
}
