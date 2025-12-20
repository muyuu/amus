use crate::{
    components::{
        grid::grid,
        tile::{tile, TileConf},
    },
    models::{color::Color, player::PlayerId},
    state::Slices,
};
use egui::*;

pub struct ChangeColorConf {
    pub player_id: PlayerId,
    pub show_delete: bool,
    pub grid_id_suffix: String,
}

pub struct ChangeColorResult {
    pub color_change: Option<Color>,
    pub delete: bool,
}

/// 色変更UIを描画（コンテキストメニュー内で使用）
pub fn change_color(ui: &mut Ui, slices: &Slices<'_>, conf: &ChangeColorConf) -> ChangeColorResult {
    let mut result = ChangeColorResult {
        color_change: None,
        delete: false,
    };

    // 削除ボタン（オプション）
    if conf.show_delete {
        if ui.button("削除").clicked() {
            result.delete = true;
            ui.close();
            return result;
        }
        ui.separator();
    }

    ui.label("色を変更");
    ui.separator();

    // 使われていない色を取得
    let all_colors = Color::all();
    let player_slice = slices.player();
    let used_colors: Vec<_> = player_slice
        .players()
        .map(|players| {
            players
                .iter()
                .filter(|p| p.id != conf.player_id)
                .map(|p| p.color.clone())
                .collect()
        })
        .unwrap_or_default();
    let available_colors: Vec<_> = all_colors
        .into_iter()
        .filter(|c| !used_colors.contains(c))
        .collect();

    // 1行4列でタイルを描画
    let tile_size = 30.0;

    // Gridコンポーネントを使用（4列固定）
    grid(ui, format!("color_grid_{}", conf.grid_id_suffix), |ui| {
        for (index, available_color) in available_colors.iter().enumerate() {
            let tile_result = tile(
                ui,
                &TileConf {
                    color: available_color.to_egui_color(),
                    label: None,
                    size: Some(Vec2::new(tile_size, tile_size)),
                    is_selected: false,
                    is_dragging: false,
                },
            );

            // ホバーエフェクト
            if tile_result.response.hovered() {
                ui.painter().rect_stroke(
                    tile_result.rect,
                    2.0,
                    (3.0, Color32::YELLOW),
                    StrokeKind::Outside,
                );
            }

            // クリックで色を変更
            if tile_result.clicked {
                result.color_change = Some(available_color.clone());
                ui.close();
            }

            // 4列ごとに改行
            if (index + 1) % 4 == 0 {
                ui.end_row();
            }
        }
    });

    result
}
