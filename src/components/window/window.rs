use super::{base, WindowConf};
use egui::*;

pub fn window<R>(
    ctx: &egui::Context,
    title: impl Into<String>,
    open: Option<&mut bool>,
    default_pos: Option<egui::Pos2>,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<InnerResponse<Option<R>>> {
    let conf = WindowConf {
        title: title.into(),
        open,
        resizable: true,
        collapsible: true,
        movable: true,
        default_size: None,
        min_size: None,
        max_size: None,
        fixed_size: None,
        default_pos,
        anchor: None,
    };
    base(ctx, conf, add_contents)
}
