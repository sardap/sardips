use bevy::prelude::*;
use text_keys::BACK;

use crate::{
    assets::FontAssets,
    button_hover::{ButtonColorSet, ButtonHover},
    text_translation::KeyText,
};

pub fn spawn_back_button<T: Component + Default>(
    parent: &mut ChildBuilder,
    font_assets: &FontAssets,
    background: &ButtonColorSet,
    border: &ButtonColorSet,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(200.),
                    height: Val::Px(50.),
                    margin: UiRect::top(Val::Px(20.)),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            ButtonHover::default()
                .with_background(*background)
                .with_border(*border),
            T::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section("", TextStyle {
                        font_size: 40.0,
                        color: Color::BLACK,
                        font: font_assets.main_font.clone(),
                    }),
                    ..default()
                },
                KeyText::new().with(0, BACK),
            ));
        });
}
