//! Embedded assets: fonts and brand graphics.

pub const LOGO_ICON: &[u8] = include_bytes!("../assets/brand/yaams-1b-icon.svg");
pub const APP_ICON_PNG: &[u8] = include_bytes!("../assets/icons/icon-256.png");

pub const FONTS: [&[u8]; 7] = [
    include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf"),
    include_bytes!("../assets/fonts/IBMPlexSans-SemiBold.ttf"),
    include_bytes!("../assets/fonts/IBMPlexSansCondensed-SemiBold.ttf"),
    include_bytes!("../assets/fonts/IBMPlexSansCondensed-Bold.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Regular.ttf"),
    include_bytes!("../assets/fonts/IBMPlexMono-Bold.ttf"),
    crate::icons::FONT_BYTES,
];
