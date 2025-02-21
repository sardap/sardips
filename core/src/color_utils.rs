use bevy::prelude::*;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct HashableColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Default for HashableColor {
    fn default() -> Self {
        Self {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        }
    }
}

impl From<HashableColor> for Color {
    fn from(value: HashableColor) -> Self {
        Color::srgba_u8(value.red, value.green, value.blue, value.alpha)
    }
}

pub fn color_point_from_percent(a_color: Color, b: Color, percent: f32) -> Color {
    assert!((0.0..=1.0).contains(&percent));
    let a = a_color.to_srgba();
    let b = b.to_srgba();

    Color::srgba(
        a.red + ((b.red - a.red) * percent),
        a.green + ((b.green - a.green) * percent),
        a.blue + ((b.blue - a.blue) * percent),
        a.alpha + ((b.alpha - a.alpha) * percent),
    )
}

pub fn srgba_u8(r: u8, g: u8, b: u8) -> Srgba {
    Srgba::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.)
}
