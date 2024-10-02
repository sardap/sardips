use bevy::color::Color;

#[macro_export]
macro_rules! rgb_to_color {
    ($r:expr, $g:expr, $b:expr) => {{
        // Normalize the RGB components to floating-point values in the range [0, 1]
        let normalized_r = $r as f32 / 255.0;
        let normalized_g = $g as f32 / 255.0;
        let normalized_b = $b as f32 / 255.0;

        bevy::color::Color::Srgba(bevy::color::Srgba {
            red: normalized_r,
            green: normalized_g,
            blue: normalized_b,
            alpha: 1.0,
        })
    }};
}

// #CCFFDD
pub const PASTEL_GREEN: Color = rgb_to_color!(204, 255, 221);

// #80FFAA
pub const MEDIUM_SPRING_GREEN: Color = rgb_to_color!(128, 255, 170);

// #E6FFEE
pub const VERY_LIGHT_GREEN: Color = rgb_to_color!(230, 255, 238);

// #FFCCD5
pub const PALE_PINK: Color = rgb_to_color!(255, 204, 213);

// #FFEAEC
pub const VERY_LIGHT_PINK_RED: Color = rgb_to_color!(255, 234, 236);

// #FFB6C1
pub const LIGHT_PINK: Color = rgb_to_color!(255, 182, 193);

// #F5F5F0
pub const OFF_WHITE: Color = rgb_to_color!(245, 245, 240);

// #5A845A
pub const LIGHT_DARK_GREEN: Color = rgb_to_color!(90, 132, 90);

pub mod ui {
    use bevy::color::Color;

    use crate::button_hover::ButtonColorSet;

    pub const BUTTON_SET: ButtonColorSet =
        ButtonColorSet::new(super::PALE_PINK, super::VERY_LIGHT_PINK_RED, Color::WHITE);
    pub const BUTTON_BORDER_SET: ButtonColorSet = ButtonColorSet::new(
        super::LIGHT_PINK,
        Color::WHITE,
        Color::Srgba(bevy::color::palettes::css::LIMEGREEN),
    );
}

pub mod view_screen {
    use bevy::color::Color;

    use crate::button_hover::ButtonColorSet;

    pub const BACKGROUND: Color = super::OFF_WHITE;

    pub const STATUS_BAR: Color = super::PALE_PINK;
    pub const STATUS_BAR_BORDER: Color = super::LIGHT_PINK;

    pub const TOP_UI: Color = super::PALE_PINK;
    pub const TOP_UI_BORDER: Color = super::LIGHT_PINK;

    pub const BUTTON_BORDER_SET: ButtonColorSet = ButtonColorSet::new(
        STATUS_BAR_BORDER,
        super::VERY_LIGHT_PINK_RED,
        Color::Srgba(bevy::color::palettes::css::LIMEGREEN),
    );
}

pub mod minigame_select {
    use bevy::color::Color;

    use crate::button_hover::ButtonColorSet;

    pub const BACKGROUND: Color = super::LIGHT_DARK_GREEN;

    pub const BUTTON_SET: ButtonColorSet = ButtonColorSet::new(
        super::PALE_PINK,
        super::VERY_LIGHT_PINK_RED,
        super::PALE_PINK,
    );
    pub const BUTTON_BORDER_SET: ButtonColorSet =
        ButtonColorSet::new(super::LIGHT_PINK, Color::WHITE, super::LIGHT_PINK);
}

pub mod dipdex_view {
    use bevy::color::Color;

    use crate::button_hover::ButtonColorSet;

    pub const ENTRY_BACKGROUND: Color = super::LIGHT_DARK_GREEN;
    pub const ENTRY_BORDER: Color = super::PALE_PINK;

    pub const BUTTON_ENTRY_SET: ButtonColorSet = ButtonColorSet::new(
        super::PASTEL_GREEN,
        super::MEDIUM_SPRING_GREEN,
        super::PASTEL_GREEN,
    )
    .with_disabled(Color::Srgba(bevy::color::palettes::css::GRAY));
    pub const BUTTON_ENTRY_BORDER_SET: ButtonColorSet = ButtonColorSet::new(
        super::VERY_LIGHT_GREEN,
        Color::WHITE,
        super::VERY_LIGHT_GREEN,
    )
    .with_disabled(Color::Srgba(bevy::color::palettes::css::DARK_GRAY));

    pub const BUTTON_SET: ButtonColorSet = ButtonColorSet::new(
        super::PALE_PINK,
        super::VERY_LIGHT_PINK_RED,
        super::PALE_PINK,
    );
    pub const BUTTON_BORDER_SET: ButtonColorSet =
        ButtonColorSet::new(super::LIGHT_PINK, Color::WHITE, super::LIGHT_PINK);
}
