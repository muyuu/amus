use crate::game::state::{AppState, DrawingMode};
use crate::models::*;
use egui::*;

pub struct MainContent;

impl MainContent {
    pub fn show(state: &mut AppState, ui: &mut egui::Ui) {
        if let Some(game) = &mut state.game {
            // 先に必要な情報を取得（エリア名など）
            let area_name = game.area.name.clone();
            let current_wave_index = state.current_wave_index;

            // マップ表示エリア（ここにエリア画像と軌跡を描画）
            let response = ui.allocate_response(
                ui.available_size(),
                egui::Sense::click_and_drag(),
            );

            // マップ描画（不変参照で描画）
            // 必要な情報を先に取得
            let temp_points = state.temp_points.clone();
            let selected_user_id = state.selected_user_id;

            if let Some(wave) = game.get_wave(current_wave_index) {
                let painter = ui.painter_at(response.rect);
                Self::draw_map_with_data(&painter, &response, game, wave, &temp_points, selected_user_id);
            }

            // 先にusersの情報を取得（借用チェッカーの問題を回避）
            let users_info: Vec<(usize, String, bool)> = game.users.iter()
                .enumerate()
                .map(|(i, u)| (i, u.name.clone(), u.alive))
                .collect();

            // その後、可変参照を取得して編集
            if let Some(wave) = game.get_wave_mut(current_wave_index) {
                ui.heading(format!("ターン {} - {}", current_wave_index + 1, area_name));

                // 描画モード選択
                ui.horizontal(|ui| {
                    ui.label("描画モード:");
                    ui.radio_value(&mut state.drawing_mode, DrawingMode::None, "なし");
                    ui.radio_value(&mut state.drawing_mode, DrawingMode::ClickToLine, "クリックで直線");
                    ui.radio_value(&mut state.drawing_mode, DrawingMode::Freehand, "フリーハンド");
                });

                ui.separator();

                // マウス操作の処理
                if let Some(user_id) = state.selected_user_id {
                    let drawing_mode = state.drawing_mode;
                    Self::handle_map_interaction(&response, wave, user_id, drawing_mode);
                }

                ui.separator();

                // 議論ターン情報
                ui.heading("議論ターン情報");

                ui.horizontal(|ui| {
                    ui.label("殺害されたプレイヤー:");
                    let mut killed_id = wave.killed;
                    egui::ComboBox::from_id_source("killed_player")
                        .selected_text(
                            if let Some(id) = killed_id {
                                if let Some((_, name, alive)) = users_info.get(id) {
                                    format!("{} ({})", name, if *alive { "生存" } else { "死亡" })
                                } else {
                                    "選択してください".to_string()
                                }
                            } else {
                                "なし".to_string()
                            }
                        )
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(killed_id.is_none(), "なし").clicked() {
                                killed_id = None;
                            }
                            for (i, (_, name, alive)) in users_info.iter().enumerate() {
                                if ui.selectable_label(
                                    killed_id == Some(i),
                                    format!("{} ({})", name, if *alive { "生存" } else { "死亡" })
                                ).clicked() {
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
                    ui.label("殺害場所（証言）:");
                    if let Some(loc) = &wave.kill_location {
                        ui.label(format!("({:.2}, {:.2})", loc.x, loc.y));
                    } else {
                        ui.label("未設定");
                    }
                    if ui.button("マップ上でクリックして設定").clicked() {
                        // マップ上でクリックした位置を殺害場所として設定
                        // これは別のモードとして実装する必要がある
                        ui.label("（実装中: マップ上で右クリックで設定予定）");
                    }
                });

                ui.label("メモ:");
                ui.text_edit_multiline(&mut wave.notes);
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.heading("Among Us 補助ツール");
                ui.label("ゲームを開始してください");
                if ui.button("新規ゲーム").clicked() {
                    state.start_new_game();
                }
            });
        }
    }

    fn draw_map_with_data(
        painter: &egui::Painter,
        response: &egui::Response,
        game: &Game,
        wave: &Wave,
        temp_points: &[Point],
        selected_user_id: Option<usize>,
    ) {
        let rect = response.rect;

        // 背景（後でエリア画像を表示）
        painter.rect_filled(rect, 0.0, Color32::from_gray(30));

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
        if route.points.is_empty() {
            return;
        }

        // スタート地点
        let start_pos = pos2(
            rect.min.x + route.start.x * rect.width(),
            rect.min.y + route.start.y * rect.height(),
        );
        painter.circle_filled(start_pos, 5.0, color);

        // 軌跡を描画
        let mut prev_pos = start_pos;
        for point in &route.points {
            let pos = pos2(
                rect.min.x + point.x * rect.width(),
                rect.min.y + point.y * rect.height(),
            );
            painter.line_segment([prev_pos, pos], (2.0, color));
            prev_pos = pos;
        }
    }

    fn draw_temp_route(painter: &egui::Painter, points: &[Point], color: Color32, rect: Rect) {
        if points.is_empty() {
            return;
        }

        let mut prev_pos = pos2(
            rect.min.x + points[0].x * rect.width(),
            rect.min.y + points[0].y * rect.height(),
        );

        for point in points.iter().skip(1) {
            let pos = pos2(
                rect.min.x + point.x * rect.width(),
                rect.min.y + point.y * rect.height(),
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
                            (pos.x - rect.min.x) / rect.width(),
                            (pos.y - rect.min.y) / rect.height(),
                        );

                        // 既存のルートを探すか、新規作成
                        if let Some(route) = wave.routes.iter_mut()
                            .find(|r| r.user_id == user_id) {
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
                            (pos.x - rect.min.x) / rect.width(),
                            (pos.y - rect.min.y) / rect.height(),
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
                            (pos.x - rect.min.x) / rect.width(),
                            (pos.y - rect.min.y) / rect.height(),
                        );

                        if let Some(route) = wave.routes.iter_mut()
                            .find(|r| r.user_id == user_id) {
                            route.add_point(point);
                        }
                    }
                }
            }
            DrawingMode::None => {}
        }
    }
}