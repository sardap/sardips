use std::time::Duration;

use bevy::prelude::*;
use bevy_turborand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    age::Age,
    anime::{AnimeBundle, AnimeIndices, AnimeTimer},
    assets::GameImageAssets,
    interaction::Clickable,
    layering,
    name::EntityName,
    simulation::{Simulated, SimulationUpdate},
    sounds::{PlaySoundEffect, SoundEffect},
    text_database::text_keys,
};

use super::hunger::Hunger;

pub struct PoopPlugin;

impl Plugin for PoopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(SimulationUpdate, tick_poopers);
    }
}

#[derive(Component, Default)]
pub struct Poop {
    // Fuck it
    // Probably should define poops in the pet tempalte file I guess I don't know
    pub texture_path: String,
}

#[derive(Bundle, Default)]
pub struct PoopBundle {
    pub poop: Poop,
    pub entity_name: EntityName,
    pub clickable: Clickable,
    pub sprite: SpriteBundle,
    pub simulated: Simulated,
}

#[derive(Component, Default, Serialize, Deserialize, Clone)]
pub struct Cleanliness;

#[derive(Component, Default, Serialize, Deserialize, Clone)]
pub struct Diarrhea;

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Pooper {
    pub interval: Duration,
    pub poop_timer: Timer,
    pub texture: String,
}

impl Pooper {
    pub fn new(poop_interval: Duration, texture: impl ToString) -> Self {
        Self {
            interval: poop_interval,
            poop_timer: Timer::new(poop_interval, TimerMode::Repeating),
            texture: texture.to_string(),
        }
    }
}

pub fn spawn_poop(
    commands: &mut Commands,
    asset_server: &AssetServer,
    game_image_assets: &GameImageAssets,
    scale: f32,
    location: Vec2,
    texture: &str,
) {
    const MAX_SIZE: f32 = 64.;
    let size = MAX_SIZE * scale;
    let size_half = size / 2.;

    let entity_id = commands
        .spawn(PoopBundle {
            poop: Poop {
                texture_path: texture.to_owned(),
            },
            clickable: Clickable::new(
                Vec2::new(-size_half, size_half),
                Vec2::new(-size_half, size_half),
            ),
            sprite: SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    location.x,
                    location.y,
                    layering::view_screen::POOP,
                ))
                .with_scale(Vec3::new(scale, scale, scale)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(MAX_SIZE, MAX_SIZE)),
                    ..default()
                },
                texture: asset_server.load(texture.to_owned()),
                ..default()
            },
            entity_name: EntityName::new(text_keys::POOP),
            ..default()
        })
        .id();

    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(64., 50.)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0., size_half, 0.)),
                texture: game_image_assets.stink_lines.clone(),
                ..default()
            },
            TextureAtlas {
                layout: game_image_assets.stink_line_layout.clone(),
                ..default()
            },
            AnimeBundle {
                timer: AnimeTimer(Timer::new(Duration::from_millis(250), TimerMode::Repeating)),
                indices: AnimeIndices::new(0, 1),
            },
            StinkLines,
            Age::default(),
        ))
        .set_parent(entity_id);
}

pub fn poop_scale(rng: &mut Rng) -> f32 {
    rng.i32(80..100) as f32 / 100.
}

#[derive(Component)]
struct StinkLines;

fn tick_poopers(
    mut commands: Commands,
    mut play_sounds: EventWriter<PlaySoundEffect>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    game_image_assets: Res<GameImageAssets>,
    mut poopers: Query<(
        &mut Pooper,
        &mut RngComponent,
        Option<&Hunger>,
        &Transform,
        Option<&Diarrhea>,
    )>,
) {
    for (mut pooper, mut rng, hunger, transform, diarrhea) in poopers.iter_mut() {
        // Only poop if hunger is over 50% full
        if hunger.map_or(true, |hunger| hunger.filled_percent() > 0.5) {
            let tick_mul = if diarrhea.is_some() { 1.0 } else { 2.0 };

            if pooper
                .poop_timer
                .tick(time.delta().mul_f32(tick_mul))
                .just_finished()
            {
                let scale = poop_scale(&mut rng.fork());

                play_sounds.send(PlaySoundEffect::new(SoundEffect::Poop).with_volume(scale));

                spawn_poop(
                    &mut commands,
                    &asset_server,
                    &game_image_assets,
                    scale,
                    transform.translation.xy(),
                    &pooper.texture,
                );
            }
        }
    }
}
