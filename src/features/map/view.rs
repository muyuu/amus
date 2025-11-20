use crate::features::location::LocationView;
use crate::features::route_drawing::RouteDrawingView;
use crate::models::{Game, Point, Wave};
use egui::*;

pub struct MapView;

impl MapView {
    /// マップ全体を描画（背景画像、ルート、位置、一時的なポイント）
    pub fn draw(
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

    /// 画像の縦横比を維持して中央配置するための矩形を計算
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

    /// 画像の外側の領域を黒で塗りつぶし
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
