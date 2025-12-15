use super::PlayerInfoConstants as Constants;
use crate::{
    components::{tile, TileConf},
    models::Player,
    state::AppState,
};
use egui::*;

pub struct PlayerListView;

impl PlayerListView {
    pub fn render(state: &mut AppState, ui: &mut Ui) {
        // 横幅一杯、高さはプレイヤーのタイルサイズ＋マージン分
        // 配置位置は親要素の一番下からタイルの余白分上にオフセット
        let max_rect = Rect::from_min_size(
            pos2(
                ui.min_rect().min.x,
                ui.max_rect().max.y - (Constants::TILE_SIZE + Constants::TILE_GAP * 2.0),
            ),
            vec2(
                ui.max_rect().width(),
                Constants::TILE_SIZE + Constants::TILE_GAP * 2.0,
            ),
        );

        ui.scope_builder(UiBuilder::new().max_rect(max_rect), |ui| {
            // 横スクロール可能なエリアで中央に配置
            ScrollArea::horizontal()
                .scroll_bar_visibility(scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    Self::render_content(state, ui);
                });
        });
    }

    fn render_content(state: &AppState, ui: &mut Ui) {
        // 利用可能なエリア全体を取得
        let available_rect = ui.available_rect_before_wrap();

        // プレイヤー数に基づいてコンテンツ幅を計算
        let players = match state.players() {
            Some(players) => players,
            None => return,
        };

        let player_width = Constants::TILE_SIZE;
        let player_gap = Constants::TILE_GAP;
        let total_content_width =
            players.len() as f32 * player_width + (players.len() as f32 - 1.0) * player_gap;
        let available_width = available_rect.width();

        // 中央揃えのためのパディングを計算
        let padding = if total_content_width < available_width {
            (available_width - total_content_width) / 2.0
        } else {
            0.0
        };

        ui.horizontal(|ui| {
            // 手動でギャップを追加るから自動余白は無効化
            ui.spacing_mut().item_spacing = Vec2 { x: 0.0, y: 0.0 };

            // 中央ぞろえ用に左側にパディングを追加
            if padding > 0.0 {
                ui.add_space(padding);
            }
            Self::render_players(state, ui, &players, player_gap);
        });
    }

    fn render_players(state: &AppState, ui: &mut Ui, players: &[Player], gap: f32) {
        // グローバルなポインター解放を検知してドラッグ終了を判定
        // 注: player_rectの返り値のdrag_stoppedは、その要素の上でマウスを離した時のみ発火する。
        // ドラッグ&ドロップでは要素外（マップ上など）でドロップすることが前提のため、
        // グローバルなポインター解放を検知する必要がある。
        let pointer_released = ui.input(|i| i.pointer.any_released());

        // ドラッグ終了時にクリア（選択状態は保持）
        if pointer_released && state.dragging_player_id().is_some() {
            state.set_dragging_player_id(None);
        }

        // 親要素の長さが足りなくてスクロールが必要になった際に余白がなくなるのを防ぐために
        // 必ず最初と最後にギャップを追加
        ui.add_space(gap);

        for (_, player) in players.iter().enumerate() {
            let is_selected = state.selected_player_id() == Some(player.id);
            let is_dragging = state.dragging_player_id() == Some(player.id);

            let result = tile(
                ui,
                &TileConf {
                    color: player.color.to_egui_color(),
                    label: Some(player.name.clone()),
                    size: Some(Vec2::new(Constants::TILE_SIZE, Constants::TILE_SIZE)),
                    is_selected,
                    is_dragging,
                },
            );

            ui.add_space(gap);

            // クリックまたはドラッグ開始時にユーザーを選択
            if result.clicked || result.drag_started {
                state.set_selected_player_id(Some(player.id));
                state.set_erase_mode(false);
                if result.drag_started {
                    state.set_dragging_player_id(Some(player.id));
                }
            }
        }
    }
}
