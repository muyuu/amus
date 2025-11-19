use super::common_texts::CommonTexts;
use super::debug_view::DebugView;
use crate::game::state::{AppState, DrawingMode};
use crate::i18n::keys::*;
use crate::models::*;
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

        // ドラッグ&ドロップの処理：ユーザーをドロップした位置に出現位置を設定
        if let Some(dragging_user_id) = state.dragging_user_id {
            // ドラッグ中の場合、マウス位置を追跡
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                // マップエリア内にマウスがあるかチェック
                if response.rect.contains(pointer_pos) {
                    // ドラッグ中はリアルタイムで位置を更新
                    if let Some(wave) = game.get_wave_mut(current_wave_index) {
                        let rect = response.rect;
                        let point = Point::new(
                            (pointer_pos.x - rect.min.x) / rect.size().x,
                            (pointer_pos.y - rect.min.y) / rect.size().y,
                        );

                        // 既存のルートを探すか、新規作成
                        if let Some(route) = wave
                            .routes
                            .iter_mut()
                            .find(|r| r.user_id == dragging_user_id)
                        {
                            // 既存のルートがある場合は start を更新
                            route.start = point;
                        } else {
                            // 新規ルートを作成
                            let route = Route::new(dragging_user_id, point);
                            wave.routes.push(route);
                        }
                        // ドラッグ中のユーザーを選択状態にする
                        state.selected_user_id = Some(dragging_user_id);
                    }
                }
            }

            // ドラッグ終了時にクリア（マウスボタンが離された時）
            if ui.ctx().input(|i| i.pointer.any_released()) {
                state.dragging_user_id = None;
            }
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
                Self::handle_map_interaction(&response, wave, user_id, drawing_mode);
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
            DebugView::show(ui.ctx(), dragging_user_id);
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
                Self::draw_route(painter, route, color, rect);
            }
        }

        // 描画中の一時的なポイント
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
            Self::draw_temp_route(painter, temp_points, color, rect);
        }
    }

    fn draw_route(painter: &egui::Painter, route: &Route, color: Color32, rect: Rect) {
        // スタート地点を描画（pointsが空でもstartは描画する）
        let start_pos = pos2(
            rect.min.x + route.start.x * rect.size().x,
            rect.min.y + route.start.y * rect.size().y,
        );
        painter.circle_filled(start_pos, 8.0, color); // サイズを大きくして見やすく

        // 軌跡を描画（pointsがある場合のみ）
        if !route.points.is_empty() {
            let mut prev_pos = start_pos;
            for point in &route.points {
                let pos = pos2(
                    rect.min.x + point.x * rect.size().x,
                    rect.min.y + point.y * rect.size().y,
                );
                painter.line_segment([prev_pos, pos], (2.0, color));
                prev_pos = pos;
            }
        }
    }

    fn draw_temp_route(painter: &egui::Painter, points: &[Point], color: Color32, rect: Rect) {
        if points.is_empty() {
            return;
        }

        let mut prev_pos = pos2(
            rect.min.x + points[0].x * rect.size().x,
            rect.min.y + points[0].y * rect.size().y,
        );

        for point in points.iter().skip(1) {
            let pos = pos2(
                rect.min.x + point.x * rect.size().x,
                rect.min.y + point.y * rect.size().y,
            );
            painter.line_segment([prev_pos, pos], (2.0, color));
            prev_pos = pos;
        }
    }

    fn handle_map_interaction(
        response: &egui::Response,
        wave: &mut Wave,
        user_id: usize,
        drawing_mode: DrawingMode,
    ) {
        match drawing_mode {
            DrawingMode::ClickToLine => {
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let rect = response.rect;
                        let point = Point::new(
                            (pos.x - rect.min.x) / rect.size().x,
                            (pos.y - rect.min.y) / rect.size().y,
                        );

                        // 既存のルートを探すか、新規作成
                        if let Some(route) = wave.routes.iter_mut().find(|r| r.user_id == user_id) {
                            // 既存のルートがある場合はポイントを追加
                            route.add_point(point);
                        } else {
                            // 新規ルートの場合は最初のポイントをstartとして設定
                            let route = Route::new(user_id, point.clone());
                            wave.routes.push(route);
                        }
                    }
                }
            }
            DrawingMode::Freehand => {
                if response.drag_started() {
                    // ドラッグ開始時に新しいルートを作成
                    if let Some(pos) = response.interact_pointer_pos() {
                        let rect = response.rect;
                        let point = Point::new(
                            (pos.x - rect.min.x) / rect.size().x,
                            (pos.y - rect.min.y) / rect.size().y,
                        );

                        // 既存のルートがあれば削除して新規作成（フリーハンドは新規描画）
                        wave.routes.retain(|r| r.user_id != user_id);
                        let route = Route::new(user_id, point.clone());
                        wave.routes.push(route);
                    }
                } else if response.dragged() {
                    // ドラッグ中はポイントを追加
                    if let Some(pos) = response.interact_pointer_pos() {
                        let rect = response.rect;
                        let point = Point::new(
                            (pos.x - rect.min.x) / rect.size().x,
                            (pos.y - rect.min.y) / rect.size().y,
                        );

                        if let Some(route) = wave.routes.iter_mut().find(|r| r.user_id == user_id) {
                            route.add_point(point);
                        }
                    }
                }
            }
            DrawingMode::None => {}
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
