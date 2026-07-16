use std::sync::{OnceLock, RwLock};

use gpui::{rgb, rgba, BorrowAppContext, Hsla, WindowAppearance};
use gpui_component::{Colorize as _, Theme, ThemeMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccentColorName {
    Blue,
    Green,
    Orange,
    Pink,
    Purple,
    Red,
    Yellow,
}

impl AccentColorName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Orange => "orange",
            Self::Pink => "pink",
            Self::Purple => "purple",
            Self::Red => "red",
            Self::Yellow => "yellow",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Blue => "Blue",
            Self::Green => "Green",
            Self::Orange => "Orange",
            Self::Pink => "Pink",
            Self::Purple => "Purple",
            Self::Red => "Red",
            Self::Yellow => "Yellow",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "blue" => Some(Self::Blue),
            "green" => Some(Self::Green),
            "orange" => Some(Self::Orange),
            "pink" => Some(Self::Pink),
            "purple" => Some(Self::Purple),
            "red" => Some(Self::Red),
            "yellow" => Some(Self::Yellow),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorSchemeName {
    CatppuccinFrappe,
    CatppuccinLatte,
    CatppuccinMacchiato,
    CatppuccinMocha,
    DefaultDark,
    DefaultLight,
    GruvboxDark,
    GruvboxLight,
}

impl ColorSchemeName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CatppuccinFrappe => "catppuccin-frappe",
            Self::CatppuccinLatte => "catppuccin-latte",
            Self::CatppuccinMacchiato => "catppuccin-macchiato",
            Self::CatppuccinMocha => "catppuccin-mocha",
            Self::DefaultDark => "default-dark",
            Self::DefaultLight => "default-light",
            Self::GruvboxDark => "gruvbox-dark",
            Self::GruvboxLight => "gruvbox-light",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::CatppuccinFrappe => "Catppuccin Frappe",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::CatppuccinMacchiato => "Catppuccin Macchiato",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::DefaultDark => "Default Dark",
            Self::DefaultLight => "Default Light",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::GruvboxLight => "Gruvbox Light",
        }
    }

    pub fn is_dark(self) -> bool {
        !matches!(
            self,
            Self::DefaultLight | Self::CatppuccinLatte | Self::GruvboxLight
        )
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "catppuccin-frappe" => Some(Self::CatppuccinFrappe),
            "catppuccin-latte" => Some(Self::CatppuccinLatte),
            "catppuccin-macchiato" => Some(Self::CatppuccinMacchiato),
            "catppuccin-mocha" => Some(Self::CatppuccinMocha),
            "default-dark" => Some(Self::DefaultDark),
            "default-light" => Some(Self::DefaultLight),
            "gruvbox-dark" => Some(Self::GruvboxDark),
            "gruvbox-light" => Some(Self::GruvboxLight),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AccentColor {
    pub foreground: u32,
    pub hex: u32,
    pub name: AccentColorName,
}

#[derive(Clone, Copy, Debug)]
pub struct SchemeColors {
    pub accent: u32,
    pub accent_foreground: u32,
    pub background: u32,
    pub background_dark: u32,
    pub border: u32,
    pub card: u32,
    pub card_foreground: u32,
    pub destructive: u32,
    pub destructive_foreground: u32,
    pub foreground: u32,
    pub input: u32,
    pub muted: u32,
    pub muted_foreground: u32,
    pub popover: u32,
    pub popover_foreground: u32,
    pub primary: u32,
    pub primary_foreground: u32,
    pub ring: u32,
    pub secondary: u32,
    pub secondary_foreground: u32,
    pub sidebar: u32,
    pub success: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct ColorScheme {
    pub accent_colors: &'static [AccentColor],
    pub colors: SchemeColors,
    pub name: ColorSchemeName,
}

#[derive(Clone, Copy, Debug)]
pub struct AppPalette {
    pub accent: AccentColor,
    pub scheme: &'static ColorScheme,
}

impl AppPalette {
    pub fn is_dark(self) -> bool {
        self.scheme.name.is_dark()
    }

    pub fn background(self) -> u32 {
        self.scheme.colors.background
    }
    pub fn background_dark(self) -> u32 {
        self.scheme.colors.background_dark
    }
    pub fn border(self) -> u32 {
        self.scheme.colors.border
    }
    pub fn card(self) -> u32 {
        self.scheme.colors.card
    }
    pub fn foreground(self) -> u32 {
        self.scheme.colors.foreground
    }
    pub fn muted_foreground(self) -> u32 {
        self.scheme.colors.muted_foreground
    }
    pub fn sidebar(self) -> u32 {
        self.scheme.colors.sidebar
    }
    pub fn sidebar_item_hover(self) -> u32 {
        self.scheme.colors.secondary
    }
    pub fn player_bg(self) -> u32 {
        self.scheme.colors.background_dark
    }
    pub fn accent_rgb(self) -> u32 {
        self.accent.hex
    }
    pub fn accent_foreground_rgb(self) -> u32 {
        self.accent.foreground
    }
    pub fn rgba(self, rgb_hex: u32, alpha: u8) -> u32 {
        (rgb_hex << 8) | alpha as u32
    }

    pub fn accent_alpha(self, alpha: u8) -> u32 {
        self.rgba(self.accent.hex, alpha)
    }
    pub fn background_alpha(self, alpha: u8) -> u32 {
        self.rgba(self.scheme.colors.background, alpha)
    }
}

const DEFAULT_LIGHT_ACCENTS: [AccentColor; 7] = [
    AccentColor {
        foreground: 0xfafafa,
        hex: 0xef4444,
        name: AccentColorName::Red,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0xf97316,
        name: AccentColorName::Orange,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0xf59e0b,
        name: AccentColorName::Yellow,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0x10b981,
        name: AccentColorName::Green,
    },
    AccentColor {
        foreground: 0xfafafa,
        hex: 0x3b82f6,
        name: AccentColorName::Blue,
    },
    AccentColor {
        foreground: 0xfafafa,
        hex: 0x8b5cf6,
        name: AccentColorName::Purple,
    },
    AccentColor {
        foreground: 0xfafafa,
        hex: 0xec4899,
        name: AccentColorName::Pink,
    },
];

const DEFAULT_DARK_ACCENTS: [AccentColor; 7] = [
    AccentColor {
        foreground: 0x18181b,
        hex: 0xef4444,
        name: AccentColorName::Red,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0xf97316,
        name: AccentColorName::Orange,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0xf59e0b,
        name: AccentColorName::Yellow,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0x10b981,
        name: AccentColorName::Green,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0x3b82f6,
        name: AccentColorName::Blue,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0x8b5cf6,
        name: AccentColorName::Purple,
    },
    AccentColor {
        foreground: 0x18181b,
        hex: 0xec4899,
        name: AccentColorName::Pink,
    },
];

const CATPPUCCIN_LATTE_ACCENTS: [AccentColor; 7] = [
    AccentColor {
        foreground: 0xeff1f5,
        hex: 0xd20f39,
        name: AccentColorName::Red,
    },
    AccentColor {
        foreground: 0x4c4f69,
        hex: 0xfe640b,
        name: AccentColorName::Orange,
    },
    AccentColor {
        foreground: 0x4c4f69,
        hex: 0xdf8e1d,
        name: AccentColorName::Yellow,
    },
    AccentColor {
        foreground: 0x4c4f69,
        hex: 0x40a02b,
        name: AccentColorName::Green,
    },
    AccentColor {
        foreground: 0xeff1f5,
        hex: 0x1e66f5,
        name: AccentColorName::Blue,
    },
    AccentColor {
        foreground: 0xeff1f5,
        hex: 0x8839ef,
        name: AccentColorName::Purple,
    },
    AccentColor {
        foreground: 0x4c4f69,
        hex: 0xea76cb,
        name: AccentColorName::Pink,
    },
];

const CATPPUCCIN_FRAPPE_ACCENTS: [AccentColor; 7] = [
    AccentColor {
        foreground: 0x303446,
        hex: 0xe78284,
        name: AccentColorName::Red,
    },
    AccentColor {
        foreground: 0x303446,
        hex: 0xf2a66f,
        name: AccentColorName::Orange,
    },
    AccentColor {
        foreground: 0x303446,
        hex: 0xe5c890,
        name: AccentColorName::Yellow,
    },
    AccentColor {
        foreground: 0x303446,
        hex: 0xa6d189,
        name: AccentColorName::Green,
    },
    AccentColor {
        foreground: 0x303446,
        hex: 0x8caaee,
        name: AccentColorName::Blue,
    },
    AccentColor {
        foreground: 0x303446,
        hex: 0xca9ee6,
        name: AccentColorName::Purple,
    },
    AccentColor {
        foreground: 0x303446,
        hex: 0xf4b8e4,
        name: AccentColorName::Pink,
    },
];

const CATPPUCCIN_MACCHIATO_ACCENTS: [AccentColor; 7] = [
    AccentColor {
        foreground: 0x24273a,
        hex: 0xed8796,
        name: AccentColorName::Red,
    },
    AccentColor {
        foreground: 0x24273a,
        hex: 0xf5a97f,
        name: AccentColorName::Orange,
    },
    AccentColor {
        foreground: 0x24273a,
        hex: 0xeed49f,
        name: AccentColorName::Yellow,
    },
    AccentColor {
        foreground: 0x24273a,
        hex: 0xa6da95,
        name: AccentColorName::Green,
    },
    AccentColor {
        foreground: 0x24273a,
        hex: 0x8aadf4,
        name: AccentColorName::Blue,
    },
    AccentColor {
        foreground: 0x24273a,
        hex: 0xc6a0f6,
        name: AccentColorName::Purple,
    },
    AccentColor {
        foreground: 0x24273a,
        hex: 0xf5bde6,
        name: AccentColorName::Pink,
    },
];

const CATPPUCCIN_MOCHA_ACCENTS: [AccentColor; 7] = [
    AccentColor {
        foreground: 0x1e1e2e,
        hex: 0xf38ba8,
        name: AccentColorName::Red,
    },
    AccentColor {
        foreground: 0x1e1e2e,
        hex: 0xfab387,
        name: AccentColorName::Orange,
    },
    AccentColor {
        foreground: 0x1e1e2e,
        hex: 0xf9e2af,
        name: AccentColorName::Yellow,
    },
    AccentColor {
        foreground: 0x1e1e2e,
        hex: 0xa6e3a1,
        name: AccentColorName::Green,
    },
    AccentColor {
        foreground: 0x1e1e2e,
        hex: 0x89b4fa,
        name: AccentColorName::Blue,
    },
    AccentColor {
        foreground: 0x1e1e2e,
        hex: 0xcba6f7,
        name: AccentColorName::Purple,
    },
    AccentColor {
        foreground: 0x1e1e2e,
        hex: 0xf5c2e7,
        name: AccentColorName::Pink,
    },
];

const GRUVBOX_LIGHT_ACCENTS: [AccentColor; 7] = [
    AccentColor {
        foreground: 0xfbf1c7,
        hex: 0xcc241d,
        name: AccentColorName::Red,
    },
    AccentColor {
        foreground: 0xfbf1c7,
        hex: 0xd65d0e,
        name: AccentColorName::Orange,
    },
    AccentColor {
        foreground: 0x3c3836,
        hex: 0xd79921,
        name: AccentColorName::Yellow,
    },
    AccentColor {
        foreground: 0xfbf1c7,
        hex: 0x98971a,
        name: AccentColorName::Green,
    },
    AccentColor {
        foreground: 0xfbf1c7,
        hex: 0x458588,
        name: AccentColorName::Blue,
    },
    AccentColor {
        foreground: 0xfbf1c7,
        hex: 0xb16286,
        name: AccentColorName::Purple,
    },
    AccentColor {
        foreground: 0x3c3836,
        hex: 0xd3869b,
        name: AccentColorName::Pink,
    },
];

const GRUVBOX_DARK_ACCENTS: [AccentColor; 7] = [
    AccentColor {
        foreground: 0x282828,
        hex: 0xfb4934,
        name: AccentColorName::Red,
    },
    AccentColor {
        foreground: 0x282828,
        hex: 0xfe8019,
        name: AccentColorName::Orange,
    },
    AccentColor {
        foreground: 0x282828,
        hex: 0xfabd2f,
        name: AccentColorName::Yellow,
    },
    AccentColor {
        foreground: 0x282828,
        hex: 0xb8bb26,
        name: AccentColorName::Green,
    },
    AccentColor {
        foreground: 0x282828,
        hex: 0x83a598,
        name: AccentColorName::Blue,
    },
    AccentColor {
        foreground: 0x282828,
        hex: 0xd3869b,
        name: AccentColorName::Purple,
    },
    AccentColor {
        foreground: 0x282828,
        hex: 0xd3869b,
        name: AccentColorName::Pink,
    },
];

const COLOR_SCHEMES: [ColorScheme; 8] = [
    ColorScheme {
        accent_colors: &DEFAULT_LIGHT_ACCENTS,
        colors: SchemeColors {
            accent: 0x64748b,
            accent_foreground: 0xfafafa,
            background: 0xffffff,
            background_dark: 0xf8f8f8,
            border: 0xe4e4e7,
            card: 0xffffff,
            card_foreground: 0x09090b,
            destructive: 0xe7000b,
            destructive_foreground: 0xfafafa,
            foreground: 0x09090b,
            input: 0xe4e4e7,
            muted: 0xf4f4f5,
            muted_foreground: 0x71717b,
            popover: 0xffffff,
            popover_foreground: 0x09090b,
            primary: 0x18181b,
            primary_foreground: 0xfafafa,
            ring: 0x9f9fa9,
            secondary: 0xf4f4f5,
            secondary_foreground: 0x18181b,
            sidebar: 0xcdcdd4,
            success: 0x10b981,
        },
        name: ColorSchemeName::DefaultLight,
    },
    ColorScheme {
        accent_colors: &DEFAULT_DARK_ACCENTS,
        colors: SchemeColors {
            accent: 0x64748b,
            accent_foreground: 0xfafafa,
            background: 0x09090b,
            background_dark: 0x030304,
            border: 0x27272a,
            card: 0x09090b,
            card_foreground: 0xfafafa,
            destructive: 0xef4444,
            destructive_foreground: 0x09090b,
            foreground: 0xfafafa,
            input: 0x27272a,
            muted: 0x27272a,
            muted_foreground: 0x9f9fa9,
            popover: 0x09090b,
            popover_foreground: 0xfafafa,
            primary: 0xfafafa,
            primary_foreground: 0x18181b,
            ring: 0x52525c,
            secondary: 0x27272a,
            secondary_foreground: 0xfafafa,
            sidebar: 0x000000,
            success: 0x10b981,
        },
        name: ColorSchemeName::DefaultDark,
    },
    ColorScheme {
        accent_colors: &CATPPUCCIN_LATTE_ACCENTS,
        colors: SchemeColors {
            accent: 0xdc8a78,
            accent_foreground: 0xeff1f5,
            background: 0xeff1f5,
            background_dark: 0xe6e9ef,
            border: 0x9ca0b0,
            card: 0xeff1f5,
            card_foreground: 0x4c4f69,
            destructive: 0xd20f39,
            destructive_foreground: 0xeff1f5,
            foreground: 0x4c4f69,
            input: 0x9ca0b0,
            muted: 0xbcc0cc,
            muted_foreground: 0x6c6f85,
            popover: 0xeff1f5,
            popover_foreground: 0x4c4f69,
            primary: 0x1e66f5,
            primary_foreground: 0xeff1f5,
            ring: 0x1e66f5,
            secondary: 0xacb0be,
            secondary_foreground: 0x4c4f69,
            sidebar: 0xd1d5db,
            success: 0x40a02b,
        },
        name: ColorSchemeName::CatppuccinLatte,
    },
    ColorScheme {
        accent_colors: &CATPPUCCIN_FRAPPE_ACCENTS,
        colors: SchemeColors {
            accent: 0xf2d5cf,
            accent_foreground: 0x303446,
            background: 0x303446,
            background_dark: 0x1a1a2e,
            border: 0x626880,
            card: 0x303446,
            card_foreground: 0xc6d0f5,
            destructive: 0xe78284,
            destructive_foreground: 0x303446,
            foreground: 0xc6d0f5,
            input: 0x626880,
            muted: 0x737994,
            muted_foreground: 0x949cbb,
            popover: 0x303446,
            popover_foreground: 0xc6d0f5,
            primary: 0x8caaee,
            primary_foreground: 0x303446,
            ring: 0x8caaee,
            secondary: 0x626880,
            secondary_foreground: 0xc6d0f5,
            sidebar: 0x292c3c,
            success: 0xa6d189,
        },
        name: ColorSchemeName::CatppuccinFrappe,
    },
    ColorScheme {
        accent_colors: &CATPPUCCIN_MACCHIATO_ACCENTS,
        colors: SchemeColors {
            accent: 0xf4dbd6,
            accent_foreground: 0x24273a,
            background: 0x24273a,
            background_dark: 0x1a1a2e,
            border: 0x5b6078,
            card: 0x24273a,
            card_foreground: 0xcad3f5,
            destructive: 0xed8796,
            destructive_foreground: 0x24273a,
            foreground: 0xcad3f5,
            input: 0x5b6078,
            muted: 0x6e738d,
            muted_foreground: 0xa5adcb,
            popover: 0x24273a,
            popover_foreground: 0xcad3f5,
            primary: 0x8aadf4,
            primary_foreground: 0x24273a,
            ring: 0x8aadf4,
            secondary: 0x5b6078,
            secondary_foreground: 0xcad3f5,
            sidebar: 0x1e1e2e,
            success: 0xa6da95,
        },
        name: ColorSchemeName::CatppuccinMacchiato,
    },
    ColorScheme {
        accent_colors: &CATPPUCCIN_MOCHA_ACCENTS,
        colors: SchemeColors {
            accent: 0xf5e0dc,
            accent_foreground: 0x1e1e2e,
            background: 0x1e1e2e,
            background_dark: 0x11111b,
            border: 0x585b70,
            card: 0x1e1e2e,
            card_foreground: 0xcdd6f4,
            destructive: 0xf38ba8,
            destructive_foreground: 0x1e1e2e,
            foreground: 0xcdd6f4,
            input: 0x585b70,
            muted: 0x6c7086,
            muted_foreground: 0xa6adc8,
            popover: 0x1e1e2e,
            popover_foreground: 0xcdd6f4,
            primary: 0x89b4fa,
            primary_foreground: 0x1e1e2e,
            ring: 0x89b4fa,
            secondary: 0x585b70,
            secondary_foreground: 0xcdd6f4,
            sidebar: 0x181825,
            success: 0xa6e3a1,
        },
        name: ColorSchemeName::CatppuccinMocha,
    },
    ColorScheme {
        accent_colors: &GRUVBOX_LIGHT_ACCENTS,
        colors: SchemeColors {
            accent: 0xd79921,
            accent_foreground: 0xfbf1c7,
            background: 0xfbf1c7,
            background_dark: 0xf2e6b6,
            border: 0xbdae93,
            card: 0xfbf1c7,
            card_foreground: 0x3c3836,
            destructive: 0xcc241d,
            destructive_foreground: 0xfbf1c7,
            foreground: 0x3c3836,
            input: 0xbdae93,
            muted: 0xd5c4a1,
            muted_foreground: 0x665c54,
            popover: 0xfbf1c7,
            popover_foreground: 0x3c3836,
            primary: 0x458588,
            primary_foreground: 0xfbf1c7,
            ring: 0x458588,
            secondary: 0xebdbb2,
            secondary_foreground: 0x3c3836,
            sidebar: 0xd5c4a1,
            success: 0x98971a,
        },
        name: ColorSchemeName::GruvboxLight,
    },
    ColorScheme {
        accent_colors: &GRUVBOX_DARK_ACCENTS,
        colors: SchemeColors {
            accent: 0xfabd2f,
            accent_foreground: 0x282828,
            background: 0x282828,
            background_dark: 0x1a1a1a,
            border: 0x504945,
            card: 0x282828,
            card_foreground: 0xebdbb2,
            destructive: 0xfb4934,
            destructive_foreground: 0x282828,
            foreground: 0xebdbb2,
            input: 0x504945,
            muted: 0x665c54,
            muted_foreground: 0xa89984,
            popover: 0x282828,
            popover_foreground: 0xebdbb2,
            primary: 0x83a598,
            primary_foreground: 0x282828,
            ring: 0x83a598,
            secondary: 0x504945,
            secondary_foreground: 0xebdbb2,
            sidebar: 0x1d2021,
            success: 0xb8bb26,
        },
        name: ColorSchemeName::GruvboxDark,
    },
];

const DEFAULT_PALETTE: AppPalette = AppPalette {
    accent: AccentColor {
        foreground: 0x18181b,
        hex: 0x3b82f6,
        name: AccentColorName::Blue,
    },
    scheme: &COLOR_SCHEMES[1],
};

static CURRENT_PALETTE: OnceLock<RwLock<AppPalette>> = OnceLock::new();

fn palette_store() -> &'static RwLock<AppPalette> {
    CURRENT_PALETTE.get_or_init(|| RwLock::new(DEFAULT_PALETTE))
}

pub fn schemes() -> &'static [ColorScheme] {
    &COLOR_SCHEMES
}

pub fn scheme(name: ColorSchemeName) -> &'static ColorScheme {
    COLOR_SCHEMES
        .iter()
        .find(|scheme| scheme.name == name)
        .unwrap_or(&COLOR_SCHEMES[1])
}

pub fn default_scheme_for_appearance(appearance: WindowAppearance) -> ColorSchemeName {
    match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => ColorSchemeName::DefaultLight,
        WindowAppearance::Dark | WindowAppearance::VibrantDark => ColorSchemeName::DefaultDark,
    }
}

pub fn resolve_palette(scheme_name: ColorSchemeName, accent_name: AccentColorName) -> AppPalette {
    let scheme = scheme(scheme_name);
    let accent = scheme
        .accent_colors
        .iter()
        .copied()
        .find(|color| color.name == accent_name)
        .unwrap_or_else(|| scheme.accent_colors[4]);

    AppPalette { accent, scheme }
}

pub fn current_palette() -> AppPalette {
    *palette_store()
        .read()
        .expect("theme palette read lock poisoned")
}

pub fn set_palette(palette: AppPalette) {
    *palette_store()
        .write()
        .expect("theme palette write lock poisoned") = palette;
}

pub fn background() -> u32 {
    current_palette().background()
}
pub fn background_dark() -> u32 {
    current_palette().background_dark()
}
pub fn border() -> u32 {
    current_palette().border()
}
pub fn card() -> u32 {
    current_palette().card()
}
pub fn foreground() -> u32 {
    current_palette().foreground()
}
pub fn muted_foreground() -> u32 {
    current_palette().muted_foreground()
}
pub fn sidebar() -> u32 {
    current_palette().sidebar()
}
pub fn sidebar_item_hover() -> u32 {
    current_palette().sidebar_item_hover()
}
pub fn player_bg() -> u32 {
    current_palette().player_bg()
}
pub fn accent() -> u32 {
    current_palette().accent_rgb()
}
pub fn accent_foreground() -> u32 {
    current_palette().accent_foreground_rgb()
}
pub fn accent_alpha(alpha: u8) -> u32 {
    current_palette().accent_alpha(alpha)
}
pub fn background_alpha(alpha: u8) -> u32 {
    current_palette().background_alpha(alpha)
}
pub fn is_dark() -> bool {
    current_palette().is_dark()
}

fn hsla_to_rgb(color: Hsla) -> u32 {
    let rgba: gpui::Rgba = color.into();
    (u32::from(rgba)) >> 8
}

pub fn accent_hover() -> u32 {
    hsla_to_rgb(hover_color(to_hsla(accent()), is_dark()))
}

pub fn accent_active() -> u32 {
    hsla_to_rgb(active_color(to_hsla(accent()), is_dark()))
}

fn to_hsla(hex: u32) -> Hsla {
    rgb(hex).into()
}

fn to_hsla_alpha(hex: u32) -> Hsla {
    rgba(hex).into()
}

fn hover_color(color: Hsla, dark: bool) -> Hsla {
    if dark {
        color.lighten(0.08)
    } else {
        color.darken(0.08)
    }
}

fn active_color(color: Hsla, dark: bool) -> Hsla {
    if dark {
        color.lighten(0.16)
    } else {
        color.darken(0.16)
    }
}

pub fn apply_theme(cx: &mut impl BorrowAppContext, palette: AppPalette) {
    set_palette(palette);

    let dark = palette.is_dark();
    let accent = to_hsla(palette.accent.hex);
    let accent_fg = to_hsla(palette.accent.foreground);
    let background = to_hsla(palette.scheme.colors.background);
    let background_dark = to_hsla(palette.scheme.colors.background_dark);
    let border = to_hsla(palette.scheme.colors.border);
    let card = to_hsla(palette.scheme.colors.card);
    let card_fg = to_hsla(palette.scheme.colors.card_foreground);
    let destructive = to_hsla(palette.scheme.colors.destructive);
    let destructive_fg = to_hsla(palette.scheme.colors.destructive_foreground);
    let foreground = to_hsla(palette.scheme.colors.foreground);
    let input = to_hsla(palette.scheme.colors.input);
    let muted = to_hsla(palette.scheme.colors.muted);
    let muted_fg = to_hsla(palette.scheme.colors.muted_foreground);
    let popover = to_hsla(palette.scheme.colors.popover);
    let popover_fg = to_hsla(palette.scheme.colors.popover_foreground);
    let secondary = to_hsla(palette.scheme.colors.secondary);
    let secondary_fg = to_hsla(palette.scheme.colors.secondary_foreground);
    let sidebar = to_hsla(palette.scheme.colors.sidebar);
    let success = to_hsla(palette.scheme.colors.success);
    let neutral_accent = to_hsla(palette.scheme.colors.accent);
    let neutral_accent_fg = to_hsla(palette.scheme.colors.accent_foreground);
    let primary = to_hsla(palette.scheme.colors.primary);
    let primary_fg = to_hsla(palette.scheme.colors.primary_foreground);
    let ring = to_hsla(palette.scheme.colors.ring);

    cx.update_global(|theme: &mut Theme, _cx| {
        theme.mode = if dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        theme.font_family = "Rubik".into();
        theme.background = background;
        theme.border = border;
        theme.foreground = foreground;
        theme.input = input;
        theme.muted = muted;
        theme.muted_foreground = muted_fg;
        theme.popover = popover;
        theme.popover_foreground = popover_fg;
        theme.secondary = secondary;
        theme.secondary_foreground = secondary_fg;
        theme.secondary_hover = hover_color(secondary, dark);
        theme.secondary_active = active_color(secondary, dark);
        theme.accent = accent;
        theme.accent_foreground = accent_fg;
        theme.primary = primary;
        theme.primary_foreground = primary_fg;
        theme.primary_hover = hover_color(primary, dark);
        theme.primary_active = active_color(primary, dark);
        theme.ring = ring;
        theme.caret = accent;
        theme.selection = to_hsla_alpha(palette.accent_alpha(if dark { 110 } else { 90 }));
        theme.colors.list = card;
        theme.list_hover = secondary;
        theme.list_active = to_hsla_alpha(palette.accent_alpha(if dark { 46 } else { 32 }));
        theme.list_active_border = accent;
        theme.table = card;
        theme.table_head = background_dark;
        theme.table_head_foreground = foreground;
        theme.table_hover = secondary;
        theme.table_active = to_hsla_alpha(palette.accent_alpha(if dark { 46 } else { 32 }));
        theme.table_active_border = accent;
        theme.table_row_border = border;
        theme.sidebar = sidebar;
        theme.sidebar_foreground = foreground;
        theme.sidebar_border = border;
        theme.sidebar_accent = to_hsla_alpha(palette.accent_alpha(if dark { 42 } else { 30 }));
        theme.sidebar_accent_foreground = accent;
        theme.sidebar_primary = accent;
        theme.sidebar_primary_foreground = accent_fg;
        theme.title_bar = background;
        theme.title_bar_border = border;
        theme.window_border = border;
        theme.progress_bar = accent;
        theme.slider_bar = muted;
        theme.slider_thumb = accent;
        theme.switch = muted;
        theme.switch_thumb = primary_fg;
        theme.scrollbar = background_dark;
        theme.scrollbar_thumb = to_hsla_alpha(palette.accent_alpha(if dark { 150 } else { 128 }));
        theme.scrollbar_thumb_hover = accent;
        theme.danger = destructive;
        theme.danger_foreground = destructive_fg;
        theme.danger_hover = hover_color(destructive, dark);
        theme.danger_active = active_color(destructive, dark);
        theme.success = success;
        theme.success_foreground = if dark { background } else { card };
        theme.success_hover = hover_color(success, dark);
        theme.success_active = active_color(success, dark);
        theme.info = to_hsla(palette.scheme.colors.primary);
        theme.info_foreground = to_hsla(palette.scheme.colors.primary_foreground);
        theme.info_hover = hover_color(theme.info, dark);
        theme.info_active = active_color(theme.info, dark);
        theme.warning = neutral_accent;
        theme.warning_foreground = neutral_accent_fg;
        theme.warning_hover = hover_color(neutral_accent, dark);
        theme.warning_active = active_color(neutral_accent, dark);
        theme.tab = background_dark;
        theme.tab_active = to_hsla_alpha(palette.accent_alpha(if dark { 46 } else { 32 }));
        theme.tab_active_foreground = accent;
        theme.tab_bar = background;
        theme.tab_bar_segmented = background_dark;
        theme.tab_foreground = muted_fg;
        theme.group_box = card;
        theme.group_box_foreground = card_fg;
        theme.description_list_label = background_dark;
        theme.description_list_label_foreground = muted_fg;
        theme.accordion = card;
        theme.accordion_hover = secondary;
        theme.skeleton = muted;
        theme.overlay = to_hsla_alpha(if dark { 0x00000088 } else { 0x00000055 });
        theme.link = accent;
        theme.link_hover = hover_color(accent, dark);
        theme.link_active = active_color(accent, dark);
        theme.tokens = (&theme.colors).into();
    });
}
