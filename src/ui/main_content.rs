use crate::common::CommonTexts;
use crate::features::debug_view::DebugView;
use crate::features::location::{LocationInteraction, LocationView};
use crate::features::route_drawing::{RouteDrawingInteraction, RouteDrawingView};
use crate::i18n::keys::*;
use crate::models::*;
use crate::state::{AppState, DrawingMode};
use egui::*;

pub struct MainContent;

impl MainContent {
    pub fn show(state: &mut AppState, ui: &mut egui::Ui) {
        // テキストをまとめて取得（borrowing conflicts回避）
        let texts = MainContentTexts::get(state);
        let common = CommonTexts::get(state);

        // ゲームがない場合はウェルカムメッセージを表示して早期リターン
        let game = match &mut state.game {
            Some(game) => game,
            None => {
                Self::show_welcome_message(ui, &texts, &common, state);
                return;
            }
        };

        // 先に必要な情報を取得
        let area_name = game.area.name.clone();
        let current_wave_index = state.current_wave_index;

        // マップ表示エリア（ここにエリア画像と軌跡を描画）
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());

        // 位置（出現位置・終了時位置）のドラッグ処理（先に処理）
        let dragging_location_value = state.dragging_location;
        if let Some(dragging_location) = dragging_location_value {
            LocationInteraction::update_dragging_location(
                game,
                current_wave_index,
                dragging_location,
                &response,
                ui.ctx(),
            );
        }

        // ドラッグ開始の検出（gameの不変参照を使用）
        if response.drag_started() {
            let dragging_location = state.dragging_location;
            if dragging_location.is_none() {
                // gameの不変参照を取得（gameの可変借用を一時的に解放）
                let game_ref: &Game = &*game;
                let location =
                    LocationInteraction::detect_drag_start(game_ref, current_wave_index, &response);
                if let Some(location) = location {
                    state.dragging_location = Some(location);
                }
            }
        }

        // ドラッグ&ドロップの処理
        let dragging_user_id = state.dragging_user_id;
        if let Some(dragging_user_id) = dragging_user_id {
            // ユーザーをドロップした位置に終了時位置を設定
            LocationInteraction::handle_user_drop(
                game,
                current_wave_index,
                dragging_user_id,
                &response,
                ui.ctx(),
            );
        }

        // エリアのクリック/ドラッグ処理（選択中のユーザーがいる場合）
        // 位置ドラッグ中でない場合のみ処理
        if state.dragging_location.is_none() {
            let selected_user_id = state.selected_user_id;
            let dragging_user_id_for_area = state.dragging_user_id;
            if let Some(selected_user_id) = selected_user_id {
                // まずgameの可変借用を解放するため、必要な情報を取得
                let needs_spawn_update = response.clicked();
                let needs_drag_tracking = response.drag_started() || response.dragged();

                if needs_spawn_update || needs_drag_tracking {
                    // gameの可変借用を一時的に解放
                    let point_opt = if needs_spawn_update || needs_drag_tracking {
                        response.interact_pointer_pos().map(|pos| {
                            LocationInteraction::screen_to_normalized_point(pos, response.rect)
                        })
                    } else {
                        None
                    };

                    // gameを再度可変借用して更新
                    if needs_spawn_update {
                        if let Some(point) = point_opt.clone() {
                            LocationInteraction::set_spawn_location(
                                game,
                                current_wave_index,
                                selected_user_id,
                                point,
                            );
                        }
                    }

                    // temp_pointsを更新（フリーハンド描画のプレビュー用）
                    if needs_drag_tracking && dragging_user_id_for_area.is_none() {
                        if let Some(point) = point_opt {
                            RouteDrawingInteraction::update_temp_points(
                                &mut state.temp_points,
                                &response,
                                point,
                            );
                        }
                    }
                }
            }
        }

        // ドラッグ終了時にクリア（マウスボタンが離された時）
        if state.dragging_user_id.is_some() && ui.ctx().input(|i| i.pointer.any_released()) {
            state.dragging_user_id = None;
        }
        if state.dragging_location.is_some() && ui.ctx().input(|i| i.pointer.any_released()) {
            state.dragging_location = None;
        }

        // マップ描画（不変参照で描画）
        // 必要な情報を先に取得
        let temp_points = state.temp_points.clone();
        let selected_user_id = state.selected_user_id;
        let show_debug_view = state.show_debug_view;
        let dragging_user_id = state.dragging_user_id;

        if let Some(wave) = game.get_wave(current_wave_index) {
            let painter = ui.painter_at(response.rect);
            Self::draw_map_with_data(
                &painter,
                &response,
                game,
                wave,
                &temp_points,
                selected_user_id,
                state.asset_manager.as_ref(),
            );
        }

        // 先にusersの情報を取得（借用チェッカーの問題を回避）
        let users_info: Vec<(usize, String, bool)> = game
            .users
            .iter()
            .enumerate()
            .map(|(i, u)| (i, u.name.clone(), u.alive))
            .collect();

        // その後、可変参照を取得して編集
        if let Some(wave) = game.get_wave_mut(current_wave_index) {
            ui.heading(format!(
                "{} {} - {}",
                common.label_turn_prefix,
                current_wave_index + 1,
                area_name
            ));

            // 描画モード選択
            ui.horizontal(|ui| {
                ui.label(&texts.drawing_mode);
                ui.radio_value(
                    &mut state.drawing_mode,
                    DrawingMode::None,
                    &texts.drawing_none,
                );
                ui.radio_value(
                    &mut state.drawing_mode,
                    DrawingMode::ClickToLine,
                    &texts.drawing_click_line,
                );
                ui.radio_value(
                    &mut state.drawing_mode,
                    DrawingMode::Freehand,
                    &texts.drawing_freehand,
                );
            });

            ui.separator();

            // マウス操作の処理
            if let Some(user_id) = state.selected_user_id {
                let drawing_mode = state.drawing_mode;
                RouteDrawingInteraction::handle_map_interaction(
                    &response,
                    wave,
                    user_id,
                    drawing_mode,
                );
            }

            ui.separator();

            // 議論ターン情報
            ui.heading(&texts.discussion_info);

            ui.horizontal(|ui| {
                ui.label(&texts.killed_player);
                let mut killed_id = wave.killed;
                egui::ComboBox::from_id_source("killed_player")
                    .selected_text(if let Some(id) = killed_id {
                        if let Some((_, name, alive)) = users_info.get(id) {
                            format!(
                                "{} ({})",
                                name,
                                if *alive {
                                    &common.status_alive
                                } else {
                                    &common.status_dead
                                }
                            )
                        } else {
                            common.select_prompt.clone()
                        }
                    } else {
                        common.select_no_selection.clone()
                    })
                    .show_ui(ui, |ui| {
                        if ui
                            .selectable_label(killed_id.is_none(), &common.select_no_selection)
                            .clicked()
                        {
                            killed_id = None;
                        }
                        for (i, (_, name, alive)) in users_info.iter().enumerate() {
                            if ui
                                .selectable_label(
                                    killed_id == Some(i),
                                    format!(
                                        "{} ({})",
                                        name,
                                        if *alive {
                                            &common.status_alive
                                        } else {
                                            &common.status_dead
                                        }
                                    ),
                                )
                                .clicked()
                            {
                                killed_id = Some(i);
                            }
                        }
                    });
                if killed_id != wave.killed {
                    wave.killed = killed_id;
                }
            });
        } // waveの可変借用を解放

        // 殺害されたプレイヤーを死亡状態にする（waveの借用を解放した後）
        if let Some(wave_ref) = game.get_wave(current_wave_index) {
            if let Some(killed_id) = wave_ref.killed {
                if let Some(user) = game.users.get_mut(killed_id) {
                    if user.alive {
                        user.alive = false;
                        user.death = Some(current_wave_index + 1);
                    }
                }
            }
        }

        // waveを再度取得して続きの処理
        if let Some(wave) = game.get_wave_mut(current_wave_index) {
            ui.horizontal(|ui| {
                ui.label(&texts.kill_location);
                if let Some(loc) = &wave.kill_location {
                    ui.label(format!("({:.2}, {:.2})", loc.x, loc.y));
                } else {
                    ui.label(&texts.location_unset);
                }
                if ui.button(&texts.set_location_btn).clicked() {
                    // マップ上でクリックした位置を殺害場所として設定
                    // これは別のモードとして実装する必要がある
                    ui.label(&texts.location_note);
                }
            });

            ui.label(&common.label_notes);
            ui.text_edit_multiline(&mut wave.notes);
        }

        // デバッグビューの表示（stateの可変借用を解放した後）
        if show_debug_view {
            let selected_user_id = state.selected_user_id;
            DebugView::show(ui.ctx(), dragging_user_id, selected_user_id, game);
        }
    }

    fn show_welcome_message(
        ui: &mut egui::Ui,
        texts: &MainContentTexts,
        common: &CommonTexts,
        state: &mut AppState,
    ) {
        ui.vertical_centered(|ui| {
            ui.heading(&texts.welcome_title);
            ui.label(&texts.welcome_message);
            if ui.button(&common.button_new_game).clicked() {
                state.start_new_game();
            }
        });
    }

    fn draw_map_with_data(
        painter: &egui::Painter,
        response: &egui::Response,
        game: &Game,
        wave: &Wave,
        temp_points: &[Point],
        selected_user_id: Option<usize>,
        asset_manager: Option<&crate::assets::AssetManager>,
    ) {
        let rect = response.rect;

        // エリア画像を背景として描画
        if let Some(asset_manager) = asset_manager {
            if let Some(texture) = asset_manager.get_area_texture(&game.area.id) {
                // 画像の縦横比を維持してセンタリング
                let image_rect = Self::calculate_centered_rect(rect, texture.size_vec2());
                painter.image(
                    texture.id(),
                    image_rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );

                // 画像の外側を黒で塗りつぶし
                if image_rect != rect {
                    Self::fill_outside_area(painter, rect, image_rect);
                }
            } else {
                // テクスチャが見つからない場合はグレーの背景
                painter.rect_filled(rect, 0.0, Color32::from_gray(30));
            }
        } else {
            // AssetManagerが初期化されていない場合はグレーの背景
            painter.rect_filled(rect, 0.0, Color32::from_gray(30));
        }

        // 既存のルートを描画
        for route in &wave.routes {
            if let Some(user) = game.users.get(route.user_id) {
                let color = user.color.to_egui_color();
                let spawn_location = wave.spawn_locations.get(&route.user_id);
                RouteDrawingView::draw_route(painter, route, spawn_location, color, rect);
            }
        }

        // 出現場所・終了時位置を描画
        LocationView::show(painter, game, wave, rect);

        // 描画中の一時的なポイント（選択中のユーザーの色で線を描画）
        if !temp_points.is_empty() {
            let color = if let Some(user_id) = selected_user_id {
                if let Some(user) = game.users.get(user_id) {
                    user.color.to_egui_color()
                } else {
                    Color32::WHITE
                }
            } else {
                Color32::WHITE
            };
            RouteDrawingView::draw_temp_route(painter, temp_points, color, rect);
        }
    }

    // 画像の縦横比を維持して中央配置するための矩形を計算
    fn calculate_centered_rect(container_rect: Rect, image_size: Vec2) -> Rect {
        let container_size = container_rect.size();
        let container_aspect = container_size.x / container_size.y;
        let image_aspect = image_size.x / image_size.y;

        let (scaled_width, scaled_height) = if container_aspect > image_aspect {
            // コンテナが横長の場合、高さに合わせてスケール
            let scaled_width = container_size.y * image_aspect;
            (scaled_width, container_size.y)
        } else {
            // コンテナが縦長の場合、幅に合わせてスケール
            let scaled_height = container_size.x / image_aspect;
            (container_size.x, scaled_height)
        };

        // 中央配置
        let x_offset = (container_size.x - scaled_width) * 0.5;
        let y_offset = (container_size.y - scaled_height) * 0.5;

        Rect::from_min_size(
            container_rect.min + Vec2::new(x_offset, y_offset),
            Vec2::new(scaled_width, scaled_height),
        )
    }

    // 画像の外側の領域を黒で塗りつぶし
    fn fill_outside_area(painter: &egui::Painter, container_rect: Rect, image_rect: Rect) {
        let black = Color32::BLACK;

        // 上側
        if image_rect.min.y > container_rect.min.y {
            painter.rect_filled(
                Rect::from_min_max(
                    container_rect.min,
                    pos2(container_rect.max.x, image_rect.min.y),
                ),
                0.0,
                black,
            );
        }

        // 下側
        if image_rect.max.y < container_rect.max.y {
            painter.rect_filled(
                Rect::from_min_max(
                    pos2(container_rect.min.x, image_rect.max.y),
                    container_rect.max,
                ),
                0.0,
                black,
            );
        }

        // 左側
        if image_rect.min.x > container_rect.min.x {
            painter.rect_filled(
                Rect::from_min_max(
                    pos2(container_rect.min.x, image_rect.min.y),
                    pos2(image_rect.min.x, image_rect.max.y),
                ),
                0.0,
                black,
            );
        }

        // 右側
        if image_rect.max.x < container_rect.max.x {
            painter.rect_filled(
                Rect::from_min_max(
                    pos2(image_rect.max.x, image_rect.min.y),
                    pos2(container_rect.max.x, image_rect.max.y),
                ),
                0.0,
                black,
            );
        }
    }
}

// メインコンテンツ固有のテキスト
struct MainContentTexts {
    pub drawing_mode: String,
    pub drawing_none: String,
    pub drawing_click_line: String,
    pub drawing_freehand: String,
    pub discussion_info: String,
    pub killed_player: String,
    pub kill_location: String,
    pub location_unset: String,
    pub set_location_btn: String,
    pub location_note: String,
    pub welcome_title: String,
    pub welcome_message: String,
}

impl MainContentTexts {
    fn get(state: &AppState) -> Self {
        Self {
            drawing_mode: state.t(MAIN_DRAWING_MODE).to_string(),
            drawing_none: state.t(MAIN_DRAWING_NONE).to_string(),
            drawing_click_line: state.t(MAIN_DRAWING_CLICK_LINE).to_string(),
            drawing_freehand: state.t(MAIN_DRAWING_FREEHAND).to_string(),
            discussion_info: state.t(MAIN_DISCUSSION_INFO).to_string(),
            killed_player: state.t(MAIN_KILLED_PLAYER).to_string(),
            kill_location: state.t(MAIN_KILL_LOCATION).to_string(),
            location_unset: state.t(MAIN_LOCATION_UNSET).to_string(),
            set_location_btn: state.t(MAIN_SET_LOCATION_BTN).to_string(),
            location_note: state.t(MAIN_LOCATION_NOTE).to_string(),
            welcome_title: state.t(MAIN_WELCOME_TITLE).to_string(),
            welcome_message: state.t(MAIN_WELCOME_MESSAGE).to_string(),
        }
    }
}
