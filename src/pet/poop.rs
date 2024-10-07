use std::time::Duration;

use bevy::prelude::*;
use bevy_turborand::prelude::*;
use moonshine_save::save::Save;
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
    view::EntityView,
    GameState,
};

use super::hunger::Hunger;

pub struct PoopPlugin;

impl Plugin for PoopPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Poop>()
            .add_systems(SimulationUpdate, tick_poopers)
            .add_systems(
                Update,
                spawn_poop_view.run_if(in_state(GameState::ViewScreen)),
            );
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Poop {
    // Fuck it
    // Probably should define poops in the pet template file I guess I don't know
    pub texture_path: String,
    pub scale: f32,
}

#[derive(Bundle, Default)]
pub struct PoopBundle {
    pub poop: Poop,
    pub entity_name: EntityName,
    pub simulated: Simulated,
    pub transform: Transform,
    pub save: Save,
}

#[derive(Component, Default)]
pub struct PoopView;

#[derive(Bundle)]
pub struct PoopBundleView {
    pub clickable: Clickable,
    pub sprite: SpriteBundle,
    pub view: EntityView,
    pub poop_view: PoopView,
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

pub fn spawn_poop(commands: &mut Commands, scale: f32, location: Vec2, texture: &str) {
    commands.spawn(PoopBundle {
        poop: Poop {
            texture_path: texture.to_owned(),
            scale,
        },
        entity_name: EntityName::new(text_keys::POOP),
        transform: Transform::from_translation(Vec3::new(
            location.x,
            location.y,
            layering::view_screen::POOP,
        )),
        ..default()
    });
}

pub fn spawn_poop_view(
    mut commands: Commands,
    poop: Query<(Entity, &Transform, &Poop), Added<Poop>>,
    asset_server: Res<AssetServer>,
    game_image_assets: Res<GameImageAssets>,
) {
    for (poop_entity, transform, poop) in poop.iter() {
        const MAX_SIZE: f32 = 64.;
        let size = MAX_SIZE * poop.scale;
        let size_half = size / 2.;

        let entity_id = commands
            .spawn(PoopBundleView {
                clickable: Clickable::new(
                    Vec2::new(-size_half, size_half),
                    Vec2::new(-size_half, size_half),
                ),
                sprite: SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        transform.translation.x,
                        transform.translation.y,
                        layering::view_screen::POOP,
                    ))
                    .with_scale(Vec3::new(poop.scale, poop.scale, poop.scale)),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(MAX_SIZE, MAX_SIZE)),
                        ..default()
                    },
                    texture: asset_server.load(&poop.texture_path),
                    ..default()
                },
                view: EntityView {
                    entity: poop_entity,
                },
                poop_view: PoopView,
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
                    scale,
                    transform.translation.xy(),
                    &pooper.texture,
                );
            }
        }
    }
}
