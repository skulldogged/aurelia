use gpui::Rgba;

pub const BACKGROUND: Rgba = color(0x111318);
pub const SIDEBAR: Rgba = color(0x161920);
pub const SURFACE: Rgba = color(0x191b22);
pub const SURFACE_HIGH: Rgba = color(0x252832);
pub const OUTLINE: Rgba = color(0x343844);
pub const TEXT: Rgba = color(0xf1f0f8);
pub const TEXT_MUTED: Rgba = color(0xa9a7b4);
pub const PRIMARY: Rgba = color(0x9b86ff);
pub const PRIMARY_CONTAINER: Rgba = color(0x302c55);
pub const PINK: Rgba = color(0xff7ca8);

const fn color(hex: u32) -> Rgba {
    Rgba {
        r: ((hex >> 16) & 0xff) as f32 / 255.0,
        g: ((hex >> 8) & 0xff) as f32 / 255.0,
        b: (hex & 0xff) as f32 / 255.0,
        a: 1.0,
    }
}

pub fn translucent(color: Rgba, alpha: f32) -> Rgba {
    color.alpha(alpha)
}
