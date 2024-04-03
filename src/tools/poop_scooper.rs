use bevy::prelude::*;

use crate::{
    assets::GameImageAssets,
    interaction::Hovering,
    layering,
    pet::poop::Poop,
    sounds::{PlaySoundEffect, SoundEffect},
};

use super::{Tool, TOOL_SIZE};

pub struct PoopScooperPlugin;

impl Plugin for PoopScooperPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, click_up_poop);
    }
}

#[derive(Bundle)]
pub struct PoopScooperBundle {
    pub poop_scooper: PoopScooper,
    pub tool: Tool,
    pub sprite: SpriteBundle,
}

#[derive(Component)]
pub struct PoopScooper;

pub fn create_poop_scooper(commands: &mut Commands, game_image_assets: &GameImageAssets) {
    commands.spawn(PoopScooperBundle {
        poop_scooper: PoopScooper,
        tool: Tool,
        sprite: SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, layering::view_screen::TOOL),
            sprite: Sprite {
                custom_size: Some(TOOL_SIZE),
                ..default()
            },
            texture: game_image_assets.poop_scooper.clone(),
            ..default()
        },
    });
}

pub fn click_up_poop(
    mut commands: Commands,
    mut sounds: EventWriter<PlaySoundEffect>,
    poop_scooper: Query<Entity, With<PoopScooper>>,
    poops: Query<Entity, (With<Poop>, With<Hovering>)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if poop_scooper.iter().count() == 0 {
        return;
    }

    if buttons.just_pressed(MouseButton::Left) {
        let mut picked_up = false;

        for poop in poops.iter() {
            picked_up = true;
            commands.entity(poop).despawn_recursive();
        }

        sounds.send(PlaySoundEffect::new(if picked_up {
            SoundEffect::Scoop
        } else {
            SoundEffect::Error
        }));
    }
}
