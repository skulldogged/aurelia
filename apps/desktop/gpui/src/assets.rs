use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
#[include = "*.ttf"]
#[include = "*.txt"]
pub struct CustomAssets;

/// Combined asset source: custom app icons first, then gpui-component bundled icons.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(content) = CustomAssets::get(path) {
            return Ok(Some(content.data));
        }
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut results: Vec<SharedString> = CustomAssets::iter()
            .filter(|p| p.starts_with(path))
            .map(|p| p.into())
            .collect();

        if let Ok(bundled) = gpui_component_assets::Assets.list(path) {
            results.extend(bundled);
        }

        Ok(results)
    }
}
