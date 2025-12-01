use egui::*;
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};
use std::collections::HashMap;

use crate::models::Area;

// include_dir! はビルド時に実行されるため、プロジェクトルートからの相対パスを指定
#[cfg(not(debug_assertions))]
static ASSETS_DIR: Dir = include_dir!("assets");

#[derive(Clone)]
pub struct AssetManager {
    area_images: HashMap<String, TextureHandle>,
}

impl AssetManager {
    pub fn new(ctx: &Context) -> Self {
        let mut manager = Self {
            area_images: HashMap::new(),
        };

        // エリア画像を読み込み
        manager.load_area_images(ctx);
        manager
    }

    fn normalize_path(path: &str) -> String {
        #[cfg(debug_assertions)]
        {
            // デバッグビルド: 外部ファイルから読み込むため元のパスをそのまま使用
            path.to_string()
        }

        #[cfg(not(debug_assertions))]
        {
            // リリースビルド: assets/ プレフィックスを除去
            // include_dir! は assets/ をルートとして保存するため
            path.strip_prefix("assets/")
                .map(|p| p.to_string())
                .unwrap_or_else(|| path.to_string())
        }
    }

    fn load_area_images(&mut self, ctx: &Context) {
        let area_configs: [(&str, &str); 4] = [
            (&Area::Skeld.id(), "assets/images/map/skeld.png"),
            (&Area::Mira.id(), "assets/images/map/mira.png"),
            (&Area::Polus.id(), "assets/images/map/polus.png"),
            (&Area::AirShip.id(), "assets/images/map/the_airship.png"),
        ];

        for (area_id, path) in &area_configs {
            let normalized_path = Self::normalize_path(path);
            if let Ok(texture) = Self::load_texture_from_path(ctx, &normalized_path) {
                self.area_images.insert(area_id.to_string(), texture);
            } else {
                // 画像が見つからない場合はプレースホルダーを作成
                let placeholder = Self::create_placeholder_texture(ctx, area_id);
                self.area_images.insert(area_id.to_string(), placeholder);
            }
        }
    }

    fn create_texture_from_bytes(
        ctx: &Context,
        path: &str,
        image_bytes: Vec<u8>,
    ) -> Result<TextureHandle, Box<dyn std::error::Error>> {
        let image = image::load_from_memory(&image_bytes)?;
        let rgba_image = image.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = rgba_image.as_flat_samples();

        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        Ok(ctx.load_texture(format!("area_{}", path), color_image, Default::default()))
    }

    // デバッグビルド用のメソッド: 外部ファイルから読み込む
    #[cfg(debug_assertions)]
    fn load_texture_from_path(
        ctx: &Context,
        path: &str,
    ) -> Result<TextureHandle, Box<dyn std::error::Error>> {
        let image_bytes = std::fs::read(path)?;
        Self::create_texture_from_bytes(ctx, path, image_bytes)
    }

    // リリースビルド用のメソッド: バイナリに埋め込まれた画像を使用
    #[cfg(not(debug_assertions))]
    fn load_texture_from_path(
        ctx: &Context,
        path: &str,
    ) -> Result<TextureHandle, Box<dyn std::error::Error>> {
        {
            ASSETS_DIR.files().for_each(|file| {
                eprintln!("  - {}", file.path().display());
            });
        }

        let image_bytes = ASSETS_DIR
            .get_file(path)
            .ok_or_else(|| format!("File not found: {}", path))?
            .contents()
            .to_vec();

        Self::create_texture_from_bytes(ctx, path, image_bytes)
    }

    fn create_placeholder_texture(ctx: &Context, area_name: &str) -> TextureHandle {
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
