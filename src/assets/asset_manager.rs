use egui::*;
#[cfg(not(debug_assertions))]
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::sync::Arc;

use crate::models::Area;
use crate::models::Theme;

// アセットディレクトリのパスを一元管理
macro_rules! assets_dir {
    () => {
        "assets"
    };
}

macro_rules! assets_path {
    ($path:expr) => {
        concat!("assets/", $path)
    };
}

// include_dir! はビルド時に実行されるため、プロジェクトルートからの相対パスを指定
// 注: include_dir!はネストしたマクロをサポートしないため、直接文字列リテラルを指定
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
    player_images: HashMap<String, TextureHandle>,
}

impl AssetManager {
    /// Context経由でAssetManagerのシングルトンインスタンスを取得または初期化
    pub fn get(ctx: &Context) -> Self {
        // 既存のインスタンスがあればそれを返す
        if let Some(manager) =
            ctx.data(|data| data.get_temp::<AssetManager>(Id::new("asset_manager")))
        {
            return manager;
        }

        // 存在しない場合は初期化して保存
        let manager = Self::new(ctx);
        ctx.data_mut(|data| {
            data.insert_temp(Id::new("asset_manager"), manager.clone());
        });
        manager
    }

    pub fn initialize(ctx: &Context) {
        Self::get(ctx);
    }

    fn new(ctx: &Context) -> Self {
        let mut manager = Self {
            area_images: HashMap::new(),
            player_images: HashMap::new(),
        };

        // エリア画像を読み込み
        manager.load_area_images(ctx);
        // プレイヤー画像を読み込み
        manager.load_player_images(ctx);
        manager
    }

    fn load_area_images(&mut self, ctx: &Context) {
        for area in Area::all() {
            let path = format!(
                assets_path!("images/map/{}_{}.png"),
                area.id(),
                Theme::Dark.as_str()
            );

            if let Ok(texture) = Self::load_texture_from_path(ctx, &path) {
                self.area_images.insert(area.id().to_string(), texture);
            } else {
                // 画像が見つからない場合はプレースホルダーを作成
                let placeholder = Self::create_placeholder_texture(ctx, &area.id());
                self.area_images.insert(area.id().to_string(), placeholder);
            }
        }
    }

    fn load_texture_from_path(
        ctx: &Context,
        path: &str,
    ) -> Result<TextureHandle, Box<dyn std::error::Error>> {
        let image_bytes = Self::load_file_bytes(path)?;
        let image = image::load_from_memory(&image_bytes)?;
        let rgba_image = image.to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = rgba_image.as_flat_samples();

        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        Ok(ctx.load_texture(format!("area_{}", path), color_image, Default::default()))
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

    fn load_player_images(&mut self, ctx: &Context) {
        use crate::models::Color;

        for color in Color::all() {
            let color_name = color.name_lowercase();

            // live画像
            let live_path = format!(assets_path!("images/char/{}-live.png"), color_name);
            if let Ok(texture) = Self::load_texture_from_path(ctx, &live_path) {
                self.player_images.insert(format!("{}-live", color_name), texture);
            }

            // dead画像
            let dead_path = format!(assets_path!("images/char/{}-dead.png"), color_name);
            if let Ok(texture) = Self::load_texture_from_path(ctx, &dead_path) {
                self.player_images.insert(format!("{}-dead", color_name), texture);
            }
        }
    }

    pub fn get_player_texture(&self, color: &crate::models::Color, is_dead: bool) -> Option<&TextureHandle> {
        let color_name = color.name_lowercase();
        let state = if is_dead { "dead" } else { "live" };
        self.player_images.get(&format!("{}-{}", color_name, state))
    }

    /// アイコンを静的に読み込み（main.rs等で使用）
    pub fn load_icon_static() -> Option<Arc<egui::IconData>> {
        Self::load_icon()
    }

    /// アイコンファイルを読み込み
    fn load_icon() -> Option<Arc<egui::IconData>> {
        let icon_path = assets_path!("icons/icon.ico");
        let icon_bytes = Self::load_file_bytes(&icon_path).ok()?;

        // ICO形式を読み込んでIconDataに変換
        let image = image::load_from_memory(&icon_bytes).ok()?;
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();

        Some(Arc::new(egui::IconData {
            rgba: rgba.into_raw(),
            width: width as u32,
            height: height as u32,
        }))
    }

    /// ファイルをバイト列として読み込み
    /// デバッグ/リリースビルドで切り替え
    fn load_file_bytes(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        {
            // デバッグビルド: ファイルシステムから読み込む
            Ok(std::fs::read(path)?)
        }

        #[cfg(not(debug_assertions))]
        {
            // リリースビルド: 埋め込みアセットから読み込む
            let relative_path = path.strip_prefix(concat!(assets_dir!(), "/")).unwrap_or(path);
            ASSETS_DIR
                .get_file(relative_path)
                .ok_or_else(|| format!("File not found: {}", path).into())
                .map(|file| file.contents().to_vec())
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn load_font_from_assets(path: &str) -> Option<Vec<u8>> {
        #[cfg(debug_assertions)]
        {
            // デバッグビルド: ファイルシステムから読み込む
            std::fs::read(format!(concat!(assets_dir!(), "/{}"), path)).ok()
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
