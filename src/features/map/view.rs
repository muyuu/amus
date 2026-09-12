use crate::constants::AppConstants;
use crate::resources::AssetManager;
use crate::state::Slices;
use egui::*;

pub struct MapView;

impl MapView {
    /// マップ全体を描画（背景画像、ルート、位置）
    pub fn render(slices: &Slices<'_>, response: &Response, ui: &mut Ui) {
        let painter = ui.painter_at(response.rect);
        let rect = response.rect;

        // エリア画像を背景として描画
        let asset_manager = AssetManager::get(ui.ctx());
        let game_slice = slices.game();
        let area = match game_slice.area() {
            Some(area) => area,
            None => return Self::render_default_area(&painter, rect),
        };

        let texture = match asset_manager.get_area_texture(area) {
            Some(texture) => texture,
            None => return Self::render_default_area(&painter, rect),
        };

        // 画像の縦横比を維持してセンタリング
        let image_rect = Self::calculate_centered_rect(rect, texture.size_vec2());

        // 画像は透過なので、下地と画像の外側をまとめて地の色で塗る
        painter.rect_filled(rect, 0.0, AppConstants::BG_COLOR);

        painter.image(
            texture.id(),
            image_rect,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    fn render_default_area(painter: &Painter, rect: Rect) {
        painter.rect_filled(rect, 0.0, AppConstants::BG_COLOR);
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
}
