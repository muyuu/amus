use egui::{ColorImage, TextureHandle};
use std::collections::HashMap;

use crate::models::Area;

pub struct AssetManager {
    area_images: HashMap<String, TextureHandle>,
}

impl AssetManager {
    pub fn new(ctx: &egui::Context) -> Self {
        let mut manager = Self {
            area_images: HashMap::new(),
        };

        // エリア画像を読み込み
        manager.load_area_images(ctx);
        manager
    }

    fn load_area_images(&mut self, ctx: &egui::Context) {
        let area_configs: [(&str, &str); 4] = [
            ("skeld", "assets/images/map/skeld.png"),
            ("mira", "assets/images/map/mira.png"),
            ("polus", "assets/images/map/polus.png"),
            ("airship", "assets/images/map/airship.png"),
        ];

        for (area_id, path) in &area_configs {
            if let Ok(texture) = Self::load_texture_from_path(ctx, path) {
                self.area_images.insert(area_id.to_string(), texture);
            } else {
                // 画像が見つからない場合はプレースホルダーを作成
                let placeholder = Self::create_placeholder_texture(ctx, area_id);
                self.area_images.insert(area_id.to_string(), placeholder);
            }
        }
    }

    fn load_texture_from_path(
        ctx: &egui::Context,
        path: &str,
    ) -> Result<TextureHandle, Box<dyn std::error::Error>> {
        let image_bytes = std::fs::read(path)?;
        let image = image::load_from_memory(&image_bytes)?;
        let rgba_image = image.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = rgba_image.as_flat_samples();

        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        Ok(ctx.load_texture(format!("area_{}", path), color_image, Default::default()))
    }

    fn create_placeholder_texture(ctx: &egui::Context, area_name: &str) -> TextureHandle {
        // プレースホルダー画像を作成（グレーの背景に文字）
        let size = [400, 300];
        let mut pixels = vec![100u8; size[0] * size[1] * 4]; // グレーの背景

        // 簡単なパターンを追加（チェッカーボード）
        for y in 0..size[1] {
            for x in 0..size[0] {
                let idx = (y * size[0] + x) * 4;
                if (x / 20 + y / 20) % 2 == 0 {
                    pixels[idx] = 120; // R
                    pixels[idx + 1] = 120; // G
                    pixels[idx + 2] = 120; // B
                } else {
                    pixels[idx] = 80; // R
                    pixels[idx + 1] = 80; // G
                    pixels[idx + 2] = 80; // B
                }
                pixels[idx + 3] = 255; // A
            }
        }

        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);

        ctx.load_texture(
            format!("placeholder_{}", area_name),
            color_image,
            Default::default(),
        )
    }

    pub fn get_area_texture(&self, area: &Area) -> Option<&TextureHandle> {
        self.area_images.get(&area.id())
    }
}
