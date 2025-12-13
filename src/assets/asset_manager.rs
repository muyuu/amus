use egui::*;
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};
use std::collections::HashMap;

use crate::models::Area;
use crate::models::Theme;

// include_dir! はビルド時に実行されるため、プロジェクトルートからの相対パスを指定
#[cfg(not(debug_assertions))]
static ASSETS_DIR: Dir = include_dir!("assets");

/// アセット（画像等）のリソース管理
///
/// ## Context経由で保存する理由
///
/// 1. **TextureHandleの依存関係**
///    - `TextureHandle`はeGuiの`Context`に依存しており、Contextが所有するテクスチャへの参照
///    - Contextから独立して存在できないため、Context経由での管理が自然
///
/// 2. **どこからでもアクセス可能**
///    - UIの描画処理（`ui.ctx()`）からどこからでも`AssetManager::get(ctx)`でアクセス可能
///    - グローバル変数やstatic変数を使わずにシングルトンのような挙動を実現
///
/// 3. **ライフタイム管理が簡単**
///    - Contextと同じライフタイムで自動管理される
///    - 明示的な破棄処理が不要
///
/// 4. **eGuiの標準パターン**
///    - `ctx.data()`/`ctx.data_mut()`でカスタムデータを保存するのがeGuiの推奨パターン
///    - 即時モードGUIにおいて、フレーム間で状態を保持する標準的な方法
#[derive(Clone)]
pub struct AssetManager {
    area_images: HashMap<String, TextureHandle>,
}

impl AssetManager {
    /// Context経由でAssetManagerのシングルトンインスタンスを取得または初期化
    pub fn get(ctx: &Context) -> Self {
        ctx.data(|data| {
            data.get_temp::<AssetManager>(Id::new("asset_manager"))
                .unwrap_or_else(|| {
                    let manager = Self::new(ctx);
                    manager
                })
        })
    }

    /// Context経由でAssetManagerを初期化（一度だけ呼ぶ）
    pub fn initialize(ctx: &Context) {
        // data_mutのクロージャ外でAssetManagerを作成（デッドロック回避）
        let should_initialize = ctx.data(|data| {
            data.get_temp::<AssetManager>(Id::new("asset_manager"))
                .is_none()
        });

        if should_initialize {
            let manager = Self::new(ctx);
            ctx.data_mut(|data| {
                data.insert_temp(Id::new("asset_manager"), manager);
            });
        }
    }

    fn new(ctx: &Context) -> Self {
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
        for area in Area::all() {
            let path = format!(
                "assets/images/map/{}_{}.png",
                area.id(),
                Theme::Dark.as_str()
            );
            let normalized_path = Self::normalize_path(&path);

            if let Ok(texture) = Self::load_texture_from_path(ctx, &normalized_path) {
                self.area_images.insert(area.id().to_string(), texture);
            } else {
                // 画像が見つからない場合はプレースホルダーを作成
                let placeholder = Self::create_placeholder_texture(ctx, &area.id());
                self.area_images.insert(area.id().to_string(), placeholder);
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

    #[cfg(target_arch = "wasm32")]
    pub fn load_font_from_assets(path: &str) -> Option<Vec<u8>> {
        #[cfg(debug_assertions)]
        {
            // デバッグビルド: ファイルシステムから読み込む
            std::fs::read(format!("assets/{}", path)).ok()
        }

        #[cfg(not(debug_assertions))]
        {
            // リリースビルド: 埋め込みアセットから読み込む
            #[cfg(target_arch = "wasm32")]
            use web_sys::console;

            match ASSETS_DIR.get_file(path) {
                Some(file) => {
                    let data = file.contents().to_vec();
                    #[cfg(target_arch = "wasm32")]
                    console::log_1(&format!("Font loaded: {} ({} bytes)", path, data.len()).into());
                    Some(data)
                }
                None => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        console::error_1(&format!("Font not found: {}", path).into());
                        console::log_1(&"Available files in ASSETS_DIR:".into());
                        ASSETS_DIR.files().for_each(|file| {
                            console::log_1(&format!("  - {}", file.path().display()).into());
                        });
                    }
                    None
                }
            }
        }
    }
}
