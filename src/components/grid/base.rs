use egui::*;

/// Grid の設定をまとめた構造体
pub struct GridConf {
    /// グリッドの識別子（必須）
    pub id: String,

    /// 行・列間のスペース
    pub spacing: Vec<f32>,

    /// グリッドの最大列数（None なら制限なし）
    pub num_columns: Option<usize>,
}

/// フル指定版
pub fn base<R>(ui: &mut Ui, conf: GridConf, add_contents: impl FnOnce(&mut Ui) -> R) -> R {
    let spacing = Vec2::new(conf.spacing[0], conf.spacing[1]);

    Grid::new(conf.id)
        .spacing(spacing)
        .num_columns(conf.num_columns.unwrap_or(0)) // 0なら制限なし
        .show(ui, add_contents)
        .inner
}
