//! Embedded assets: fonts and brand graphics.

pub const LOGO_ICON: &[u8] = include_bytes!("../assets/brand/yaams-1b-icon.svg");
pub const APP_ICON_PNG: &[u8] = include_bytes!("../assets/icons/icon-256.png");

pub const FONTS: [&[u8]; 6] = [
    include_bytes!("../assets/fonts/Inter-Regular.ttf"),
    include_bytes!("../assets/fonts/Inter-Bold.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
    include_bytes!("../assets/fonts/SpaceGrotesk-Bold.ttf"),
    crate::icons::FONT_BYTES,
];
