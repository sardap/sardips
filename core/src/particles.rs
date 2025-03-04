use std::{ops::Range, time::Duration};

use bevy::{color::ColorRange, prelude::*, utils::hashbrown::HashSet};
use serde::{Deserialize, Serialize};
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
        app.register_type::<ParticleShape>()
            .register_type::<SpewerSize>()
            .register_type::<Spewer>()
            .add_systems(
                Update,
                (
                    spawn_particles,
                    add_rng_to_spewer,
                    add_visibility_to_spewer,
                    update_spewer_assets,
                    update_spewer_spawn_timer,
                    despawn_particles,
                    change_particle_color,
                    despawn_particles_from_hidden_spewer,
                )
                    .run_if(resource_exists::<ParticleAssets>),
            );
    }
}

#[derive(Default, PartialEq, Clone, Serialize, Deserialize, Reflect)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum ParticleShape {
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

#[derive(Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum SpewerSize {
    Uniform(f32),
    UniformRange(Range<f32>),
    NonUniform(Vec2),
    NonUniformRange(Range<Vec2>),
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum SpewerLifetime {
    Uniform(Duration),
    Range(Range<Duration>),
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub struct Spewer {
    pub shapes: Vec<ParticleShape>,
    pub colors: Vec<Range<Color>>,
    pub size: SpewerSize,
    pub direction_min: Vec3,
    pub direction_max: Vec3,
    pub speed: Range<f32>,
    pub lifetime: SpewerLifetime,
    pub spawn_interval: Duration,
    pub spawn_area: Rect,
}

lazy_static! {
    pub static ref CONFETTI: Spewer = {
        Spewer {
            shapes: vec![ParticleShape::Square],
            colors: vec![
                srgba_u8(168, 100, 253),
                srgba_u8(41, 205, 255),
                srgba_u8(120, 255, 68),
                srgba_u8(255, 113, 141),
                srgba_u8(253, 255, 106),
            ]
            .into_iter()
            .map(|color| Color::Srgba(color)..Color::srgba(color.red, color.green, color.blue, 0.))
            .collect(),
            size: SpewerSize::UniformRange(0.5..1.5),
            direction_min: Vec3::new(-1.0, -1.0, 0.0),
            direction_max: Vec3::new(1.0, 1.0, 0.0),
            speed: 100.0..150.0,
            lifetime: SpewerLifetime::Range(
                Duration::from_millis(1800)..Duration::from_millis(2000),
            ),
            spawn_interval: Duration::from_millis(25),
            spawn_area: Rect::new(-5., -5., 5., 5.),
        }
    };
    pub static ref SPARKS: Spewer = {
        Spewer {
            shapes: vec![ParticleShape::Circle],
            colors: vec![Color::srgb_u8(255, 230, 0)..Color::srgba_u8(255, 0, 0, 0)],
            size: SpewerSize::NonUniformRange(Vec2::new(0.3, 0.3)..Vec2::new(0.5, 0.5)),
            direction_min: Vec3::new(-1.0, 0.0, 0.0),
            direction_max: Vec3::new(1.0, 1.0, 0.0),
            speed: 50.0..100.0,
            lifetime: SpewerLifetime::Range(
                Duration::from_millis(800)..Duration::from_millis(1000),
            ),
            spawn_interval: Duration::from_millis(50),
            ..default()
        }
    };
    pub static ref HAPPY: Spewer = {
        Spewer {
            shapes: vec![ParticleShape::Custom((
                "textures/particles/smiling.png".to_string(),
                Vec2::new(5., 5.),
            ))],
            colors: vec![
                Color::srgba(1., 0., 0., 1.)..Color::srgba(0., 0., 1., 0.),
                Color::srgba(0., 1., 0., 1.)..Color::srgba(0., 1., 0., 0.),
                Color::srgba(0., 0., 1., 1.)..Color::srgba(1., 0., 0., 0.),
            ],
            size: SpewerSize::Uniform(1.0),
            direction_min: Vec3::new(-1.0, -1.0, 0.0),
            direction_max: Vec3::new(1.0, 1.0, 0.0),
            speed: 50.0..100.0,
            lifetime: SpewerLifetime::Uniform(Duration::from_millis(1500)),
            spawn_interval: Duration::from_millis(50),
            spawn_area: Rect::new(-5., -5., 5., 5.),
        }
    };
    pub static ref BITS: Spewer = {
        Spewer {
            shapes: vec![
                ParticleShape::Custom(("textures/particles/1.png".to_string(), Vec2::new(5., 5.))),
                ParticleShape::Custom(("textures/particles/0.png".to_string(), Vec2::new(5., 5.))),
            ],
            colors: vec![Color::srgba(0., 1., 0., 0.5)..Color::srgba(0., 1., 0., 0.)],
            size: SpewerSize::Uniform(1.0),
            direction_min: Vec3::new(-1.0, -1.0, 0.0),
            direction_max: Vec3::new(1.0, 0.2, 0.0),
            speed: 20.0..50.0,
            lifetime: SpewerLifetime::Uniform(Duration::from_millis(3000)),
            spawn_interval: Duration::from_millis(500),
            ..default()
        }
    };
}

impl Default for Spewer {
    fn default() -> Self {
        Self {
            shapes: vec![ParticleShape::Circle],
            colors: vec![Color::srgba(1., 0., 0., 1.)..Color::srgba(0., 1., 0., 0.)],
            size: SpewerSize::Uniform(1.0),
            direction_min: Vec3::new(-1.0, -1.0, 0.0),
            direction_max: Vec3::new(1.0, 1.0, 0.0),
            speed: 50.0..100.0,
            lifetime: SpewerLifetime::Uniform(Duration::from_millis(1500)),
            spawn_interval: Duration::from_millis(50),
            spawn_area: Rect::new(-5., -5., 5., 5.),
        }
    }
}

impl Spewer {
    pub fn with_possible_color(mut self, color: Range<Color>) -> Self {
        self.colors.push(color);
        self
    }

    pub fn with_size(mut self, size: SpewerSize) -> Self {
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

    pub fn with_spawn_area(mut self, spawn_area: Rect) -> Self {
        self.spawn_area = spawn_area;
        self
    }
}

#[derive(Component)]
struct SpewerAssets {
    images: Vec<(Handle<Image>, Vec2)>,
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
        let mut images = Vec::new();
        for shape in &spewer.shapes {
            images.push(shape.get_image(&assets, &asset_server));
        }
        match spewer_assets {
            Some(mut spewer_assets) => {
                spewer_assets.images = images;
            }
            None => {
                commands.entity(entity).insert(SpewerAssets { images });
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

fn add_visibility_to_spewer(
    mut commands: Commands,
    spewer: Query<Entity, (With<Spewer>, Without<Visibility>)>,
) {
    for entity in &spewer {
        commands.entity(entity).insert(Visibility::Visible);
    }
}

fn spawn_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut spewer: Query<(
        Entity,
        &GlobalTransform,
        &Visibility,
        &Spewer,
        &SpewerAssets,
        &mut SpewerSpawnTimer,
        &mut RngComponent,
    )>,
) {
    for (entity, transform, visibility, spewer, spewer_assets, mut spawner_timer, mut rng) in
        &mut spewer
    {
        if *visibility != Visibility::Visible {
            continue;
        }

        spawner_timer.timer.tick(time.delta());
        let rng = rng.as_mut();
        for _ in 0..=spawner_timer.timer.times_finished_this_tick() {
            let direction = Vec3::new(
                gen_f32_range(rng, &(spewer.direction_min.x..spewer.direction_max.x)),
                gen_f32_range(rng, &(spewer.direction_min.y..spewer.direction_max.y)),
                0.,
            );
            let color_index = rng.usize(0..spewer.colors.len());
            let speed = gen_f32_range(rng, &spewer.speed);
            let location = transform.translation()
                + Vec3::new(
                    gen_f32_range(rng, &(spewer.spawn_area.min.x..spewer.spawn_area.max.x)),
                    gen_f32_range(rng, &(spewer.spawn_area.min.y..spewer.spawn_area.max.y)),
                    0.,
                );
            let size = match &spewer.size {
                SpewerSize::Uniform(size) => Vec3::new(*size, *size, 1.),
                SpewerSize::UniformRange(range) => {
                    let size = gen_f32_range(rng, range);
                    Vec3::new(size, size, 1.)
                }
                SpewerSize::NonUniform(size) => Vec3::new(size.x, size.y, 1.),
                SpewerSize::NonUniformRange(range) => Vec3::new(
                    gen_f32_range(rng, &(range.start.x..range.end.x)),
                    gen_f32_range(rng, &(range.start.y..range.end.y)),
                    1.,
                ),
            };

            let lifetime = match &spewer.lifetime {
                SpewerLifetime::Uniform(lifetime) => *lifetime,
                SpewerLifetime::Range(range) => {
                    let micros = rng.u128(range.start.as_micros()..range.end.as_micros());
                    Duration::from_micros(micros as u64)
                }
            };

            let (image, image_size) =
                spewer_assets.images[rng.usize(0..spewer_assets.images.len())].clone();

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: spewer.colors[color_index].start,
                        custom_size: Some(image_size),
                        ..default()
                    },
                    transform: Transform::from_xyz(location.x, location.y, location.z)
                        .with_scale(size),
                    texture: image.clone(),
                    ..default()
                },
                Speed(speed),
                Particle::new(entity, lifetime, color_index),
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
            let range = match spewer.colors.get(particle.color_index) {
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

// This should probably not just loop over all particles
fn despawn_particles_from_hidden_spewer(
    mut commands: Commands,
    particles: Query<(Entity, &Particle)>,
    spewer: Query<(Entity, &Visibility), (With<Spewer>, Changed<Visibility>)>,
) {
    let owners_invisible: HashSet<Entity> = spewer
        .iter()
        .filter_map(|(entity, visibility)| {
            if *visibility == Visibility::Hidden {
                Some(entity)
            } else {
                None
            }
        })
        .collect();

    for (entity, particle) in &particles {
        if owners_invisible.contains(&particle.spawner) {
            commands.entity(entity).despawn_recursive();
        }
    }
}
