use std::{ops::Range, time::Duration};

use bevy::{color::ColorRange, prelude::*};
use shared_deps::bevy_turborand::{DelegatedRng, RngComponent};

use crate::{
    assets::ParticleAssets,
    color_utils::srgba_u8,
    rand_utils::gen_f32_range,
    velocity::{MovementDirection3D, Speed, VelocityDeltaUpdate},
};

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_particles,
                add_rng_to_spewer,
                update_spewer_assets,
                update_spewer_spawn_timer,
                despawn_particles,
                change_particle_color,
            )
                .run_if(resource_exists::<ParticleAssets>),
        );
    }
}

#[derive(Default, PartialEq, Clone)]
enum ParticleShape {
    #[default]
    Circle,
    Square,
    Custom((String, Vec2)),
}

impl ParticleShape {
    pub fn get_image(
        &self,
        assets: &ParticleAssets,
        asset_server: &AssetServer,
    ) -> (Handle<Image>, Vec2) {
        match self {
            ParticleShape::Circle => (assets.circle.clone(), Vec2::new(5., 5.)),
            ParticleShape::Square => (assets.square.clone(), Vec2::new(5., 5.)),
            ParticleShape::Custom((path, size)) => {
                let image = asset_server.load(path);
                (image, *size)
            }
        }
    }
}

#[derive(Component, Clone)]
pub struct Spewer {
    shape: ParticleShape,
    possible_ranges: Vec<Range<Color>>,
    size: f32,
    direction_min: Vec3,
    direction_max: Vec3,
    speed: Range<f32>,
    lifetime: Duration,
    spawn_interval: Duration,
    spawn_area: Rect,
}

lazy_static! {
    pub static ref CONFETTI: Spewer = {
        Spewer {
            shape: ParticleShape::Square,
            possible_ranges: vec![
                srgba_u8(168, 100, 253),
                srgba_u8(41, 205, 255),
                srgba_u8(120, 255, 68),
                srgba_u8(255, 113, 141),
                srgba_u8(253, 255, 106),
            ]
            .into_iter()
            .map(|color| Color::Srgba(color)..Color::srgba(color.red, color.green, color.blue, 0.))
            .collect(),
            size: 0.5,
            direction_min: Vec3::new(-1.0, -1.0, 0.0),
            direction_max: Vec3::new(1.0, 1.0, 0.0),
            speed: 100.0..150.0,
            lifetime: Duration::from_millis(2000),
            spawn_interval: Duration::from_millis(25),
            spawn_area: Rect::new(-5., -5., 5., 5.),
        }
    };
    pub static ref SPARKS: Spewer = {
        Spewer {
            shape: ParticleShape::Circle,
            possible_ranges: vec![Color::srgb_u8(255, 230, 0)..Color::srgba_u8(255, 0, 0, 0)],
            size: 0.5,
            direction_min: Vec3::new(-1.0, 0.0, 0.0),
            direction_max: Vec3::new(1.0, 1.0, 0.0),
            speed: 50.0..100.0,
            lifetime: Duration::from_millis(1000),
            spawn_interval: Duration::from_millis(50),
            spawn_area: Rect::new(-1., -1., 1., 1.),
        }
    };
}

impl Default for Spewer {
    fn default() -> Self {
        Self {
            shape: ParticleShape::Custom((
                "textures/particles/smiling.png".to_string(),
                Vec2::new(5., 5.),
            )),
            possible_ranges: vec![Color::srgba(1., 0., 0., 1.)..Color::srgba(0., 1., 0., 0.)],
            size: 1.0,
            direction_min: Vec3::new(-1.0, -1.0, 0.0),
            direction_max: Vec3::new(1.0, 1.0, 0.0),
            speed: 50.0..100.0,
            lifetime: Duration::from_millis(1500),
            spawn_interval: Duration::from_millis(50),
            spawn_area: Rect::new(-1., -1., 1., 1.),
        }
    }
}

impl Spewer {
    pub fn with_shape(mut self, shape: ParticleShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn with_possible_color(mut self, color: Range<Color>) -> Self {
        self.possible_ranges.push(color);
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_direction(mut self, direction: Vec3) -> Self {
        self.direction_min = direction;
        self
    }

    pub fn with_speed(mut self, speed: Range<f32>) -> Self {
        self.speed = speed;
        self
    }
}

#[derive(Component)]
struct SpewerAssets {
    image: Handle<Image>,
    size: Vec2,
}

fn update_spewer_assets(
    mut commands: Commands,
    assets: Res<ParticleAssets>,
    asset_server: Res<AssetServer>,
    mut spewer: Query<
        (Entity, &Spewer, Option<&mut SpewerAssets>),
        (Added<Spewer>, Changed<Spewer>),
    >,
) {
    for (entity, spewer, spewer_assets) in &mut spewer {
        let (image, size) = spewer.shape.get_image(&assets, &asset_server);
        match spewer_assets {
            Some(mut spewer_assets) => {
                spewer_assets.image = image;
                spewer_assets.size = size;
            }
            None => {
                commands.entity(entity).insert(SpewerAssets { image, size });
            }
        }
    }
}

#[derive(Component)]
struct SpewerSpawnTimer {
    timer: Timer,
}

fn update_spewer_spawn_timer(
    mut commands: Commands,
    mut spewer: Query<
        (Entity, &Spewer, Option<&mut SpewerSpawnTimer>),
        Or<(Changed<Spewer>, Added<Spewer>)>,
    >,
) {
    for (entity, spewer, spawn_history) in &mut spewer {
        match spawn_history {
            Some(mut spawn_timer) => {
                if spawn_timer.timer.duration() == spewer.spawn_interval {
                    spawn_timer.timer = Timer::from_seconds(
                        spewer.spawn_interval.as_secs_f32(),
                        TimerMode::Repeating,
                    );
                }
            }
            None => {
                let timer =
                    Timer::from_seconds(spewer.spawn_interval.as_secs_f32(), TimerMode::Repeating);
                commands.entity(entity).insert(SpewerSpawnTimer { timer });
            }
        };
    }
}

#[derive(Component)]
struct Particle {
    spawner: Entity,
    alive_for: Duration,
    die_after: Duration,
    color_index: usize,
}

impl Particle {
    pub fn new(spawner: Entity, die_after: Duration, color_index: usize) -> Self {
        Self {
            spawner,
            alive_for: Duration::ZERO,
            die_after,
            color_index,
        }
    }
}

fn add_rng_to_spewer(
    mut commands: Commands,
    spewer: Query<Entity, (With<Spewer>, Without<RngComponent>)>,
) {
    for entity in &spewer {
        commands.entity(entity).insert(RngComponent::new());
    }
}

fn spawn_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut spewer: Query<(
        Entity,
        &Transform,
        &Spewer,
        &SpewerAssets,
        &mut SpewerSpawnTimer,
        &mut RngComponent,
    )>,
) {
    for (entity, global_transform, spewer, spewer_assets, mut spawner_timer, mut rng) in &mut spewer
    {
        spawner_timer.timer.tick(time.delta());
        let rng = rng.as_mut();
        for _ in 0..=spawner_timer.timer.times_finished_this_tick() {
            let direction = Vec3::new(
                gen_f32_range(rng, &(spewer.direction_min.x..spewer.direction_max.x)),
                gen_f32_range(rng, &(spewer.direction_min.y..spewer.direction_max.y)),
                0.,
            );
            let color_index = rng.usize(0..spewer.possible_ranges.len());
            let speed = gen_f32_range(rng, &spewer.speed);
            let location = global_transform.translation
                + Vec3::new(
                    gen_f32_range(rng, &(spewer.spawn_area.min.x..spewer.spawn_area.max.x)),
                    gen_f32_range(rng, &(spewer.spawn_area.min.y..spewer.spawn_area.max.y)),
                    0.,
                );

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: spewer.possible_ranges[color_index].start,
                        custom_size: Some(Vec2::new(spewer_assets.size.x, spewer_assets.size.y)),
                        ..default()
                    },
                    transform: Transform::from_xyz(location.x, location.y, location.z)
                        .with_scale(Vec3::new(spewer.size, spewer.size, 1.)),
                    texture: spewer_assets.image.clone(),
                    ..default()
                },
                Speed(speed),
                Particle::new(entity, spewer.lifetime, color_index),
                MovementDirection3D { direction },
                VelocityDeltaUpdate,
            ));
        }
    }
}

fn despawn_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Particle)>,
) {
    for (entity, mut particle) in &mut particles {
        particle.alive_for += time.delta();
        if particle.alive_for >= particle.die_after {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(Default)]
struct ChangeParticleLocal {
    entity_modulo: u64,
}

fn change_particle_color(
    mut locals: Local<ChangeParticleLocal>,
    mut particles: Query<(Entity, &mut Sprite, &Particle)>,
    spewers: Query<&Spewer>,
) {
    const MODULO_MAX: u64 = 10;

    for (entity, mut sprite, particle) in &mut particles {
        if entity.to_bits() % MODULO_MAX != locals.entity_modulo {
            continue;
        }

        if let Ok(spewer) = spewers.get(particle.spawner) {
            let range = match spewer.possible_ranges.get(particle.color_index) {
                Some(range) => range,
                None => continue,
            };
            let percent_to_death =
                particle.alive_for.as_secs_f32() / particle.die_after.as_secs_f32();

            let new_color = range.at(percent_to_death);
            if sprite.color != new_color {
                sprite.color = new_color;
            }
        }
    }

    locals.entity_modulo = (locals.entity_modulo + 1) % MODULO_MAX;
}
