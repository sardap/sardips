#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
use std::ops::Range;
use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::utils::hashbrown::HashSet;
use maplit::hashmap;
use sardips_core::{
    assets::{EndlessShooterAssets, FontAssets},
    button_hover::{ButtonColorSet, ButtonHover},
    interaction::{MouseCamera, MoveTowardsCursor},
    minigames_core::{MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType, Playing},
    mood_core::{AutoSetMoodImage, MoodCategory, MoodImageIndexes},
    shrink::Shrinking,
    text_translation::{KeyString, KeyText},
    velocity::Speed,
};
use sardips_core::{rgb_to_color, VaryingTimer};
use shared_deps::avian2d::prelude::{
    Collider, ColliderDensity, CollidingEntities, CollisionLayers, GravityScale, LinearVelocity,
    Mass, PhysicsLayer, RigidBody,
};
use shared_deps::bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};

use text_keys::{
    MINIGAME_ENDLESS_SHOOTER_CLUSTER_GUN, MINIGAME_ENDLESS_SHOOTER_CLUSTER_GUN_COOLDOWN,
    MINIGAME_ENDLESS_SHOOTER_COOLDOWN, MINIGAME_ENDLESS_SHOOTER_MINIGUN,
    MINIGAME_ENDLESS_SHOOTER_MINIGUN_COOLDOWN, MINIGAME_ENDLESS_SHOOTER_PISTOL,
    MINIGAME_ENDLESS_SHOOTER_PISTOL_COOLDOWN, MINIGAME_ENDLESS_SHOOTER_QUIT,
    MINIGAME_ENDLESS_SHOOTER_ROCKET, MINIGAME_ENDLESS_SHOOTER_ROCKET_COOLDOWN,
    MINIGAME_ENDLESS_SHOOTER_SCORE, MINIGAME_ENDLESS_SHOOTER_TURNCOAT_GUN,
    MINIGAME_ENDLESS_SHOOTER_TURNCOAT_GUN_COOLDOWN, MINIGAME_ENDLESS_SHOOTER_WAVE_GUN,
    MINIGAME_ENDLESS_SHOOTER_WAVE_GUN_COOLDOWN,
};

extern crate maplit;

pub struct EndlessShooterPlugin;

impl Plugin for EndlessShooterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(EndlessShooterState::default())
            .add_event::<BulletSpawnEvent>()
            .add_systems(
                OnEnter(MiniGameState::PlayingEndlessShooter),
                (
                    setup_camera_and_ui,
                    setup_shooter,
                    setup_walls,
                    setup_spawners,
                ),
            )
            .add_systems(OnExit(MiniGameState::PlayingEndlessShooter), teardown)
            .add_systems(
                FixedUpdate,
                (
                    queue_spawn_bullets,
                    spawn_bullets,
                    tick_bullets,
                    spawn_walkers,
                    update_walker_direction,
                    spawn_gate,
                    update_gate_text,
                )
                    .run_if(in_state(EndlessShooterState::Playing)),
            )
            .add_systems(
                Update,
                (
                    bullet_hit,
                    shooter_hit,
                    walker_hit,
                    update_mood,
                    temp_mood_added,
                    update_gate_color,
                    lock_shooter_y,
                    update_damage_fade,
                    update_tint_turncoat,
                    kill_walkers,
                    check_game_over,
                    update_shooter_shield,
                    tick_score,
                    tick_invincibility,
                    toggle_walker_bodies,
                )
                    .run_if(in_state(EndlessShooterState::Playing)),
            )
            .add_systems(OnEnter(EndlessShooterState::GameOver), setup_game_over)
            .add_systems(
                Update,
                (
                    update_walker_direction_game_over,
                    spawn_walkers,
                    despawn_walkers_game_over,
                    quit_button_pressed,
                )
                    .run_if(in_state(EndlessShooterState::GameOver)),
            )
            .add_systems(OnEnter(EndlessShooterState::Exit), send_complete);
    }
}

// #FFCCD5
pub const PALE_PINK: Color = rgb_to_color!(255, 204, 213);

// #FFEAEC
pub const VERY_LIGHT_PINK_RED: Color = rgb_to_color!(255, 234, 236);

// #FFB6C1
pub const LIGHT_PINK: Color = rgb_to_color!(255, 182, 193);

#[derive(PhysicsLayer, Default)]
enum ColLayer {
    #[default]
    Default,
    Bullet,
    Walker,
    Gate,
    Shooter,
    Wall,
}

#[derive(Copy, Clone)]
enum ZLayer {
    Background = 1,
    DeadWalker,
    Walker,
    Pet,
    Bullet,
    Gate,
    GateText,
}

impl ZLayer {
    fn to_f32(self) -> f32 {
        self as u8 as f32
    }
}

const GAME_WIDTH: i32 = 300;
const GAME_HEIGHT: i32 = 700;
const GAME_TOP_Y: i32 = GAME_HEIGHT / 2;
const GAME_BOTTOM_Y: i32 = -GAME_HEIGHT / 2;

const SHOOTER_Y: f32 = -255.;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum EndlessShooterState {
    #[default]
    None,
    Playing,
    GameOver,
    Exit,
}

#[derive(Component)]
struct EndlessShooter;

#[derive(Component)]
struct EndlessShooterPet;

#[derive(Component)]
struct EndlessShooterCamera;

struct OwnedGun {
    last_shot: VaryingTimer,
    gun: Gun,
}

impl OwnedGun {
    fn new<T: DelegatedRng>(gun: Gun, rng: &mut T) -> Self {
        Self {
            last_shot: VaryingTimer::new(gun.cooldown.clone(), rng),
            gun,
        }
    }
}

#[derive(Component)]
struct Shooter {
    guns: Vec<OwnedGun>,
    multiplier: i32,
    damage: f32,
    y_speed_bonus: f32,
    health: i32,
}

impl Shooter {
    fn new<T: DelegatedRng>(gun: Gun, rng: &mut T) -> Self {
        Self {
            multiplier: 1,
            damage: 0.,
            y_speed_bonus: 0.,
            guns: vec![OwnedGun::new(gun, rng)],
            health: 2,
        }
    }
}

#[derive(Clone, PartialEq)]
struct Gun {
    cooldown: Range<Duration>,
    count: i32,
    x_arc: i32,
    y_speed: Range<i32>,
    size: f32,
    damage: Range<i32>,
    text_key: &'static str,
}

const PISTOL: Gun = Gun {
    cooldown: Duration::from_millis(300)..Duration::from_millis(400),
    count: 1,
    x_arc: 10,
    y_speed: 90..110,
    size: 0.2,
    damage: 10..20,
    text_key: MINIGAME_ENDLESS_SHOOTER_PISTOL,
};

const MINIGUN: Gun = Gun {
    cooldown: Duration::from_millis(50)..Duration::from_millis(100),
    count: 3,
    x_arc: 50,
    y_speed: 200..400,
    size: 0.15,
    damage: 5..10,
    text_key: MINIGAME_ENDLESS_SHOOTER_MINIGUN,
};

const WAVE_GUN: Gun = Gun {
    cooldown: Duration::from_millis(500)..Duration::from_millis(1000),
    count: 20,
    x_arc: 40,
    y_speed: 50..60,
    size: 0.08,
    damage: 1..5,
    text_key: MINIGAME_ENDLESS_SHOOTER_WAVE_GUN,
};

const ROCKET: Gun = Gun {
    cooldown: Duration::from_millis(800)..Duration::from_millis(1300),
    count: 1,
    x_arc: 1,
    y_speed: 400..800,
    size: 1.5,
    damage: 90..200,
    text_key: MINIGAME_ENDLESS_SHOOTER_ROCKET,
};

const CLUSTER_GUN: Gun = Gun {
    cooldown: Duration::from_millis(1500)..Duration::from_millis(2000),
    count: 1,
    x_arc: 5,
    y_speed: 30..40,
    size: 0.6,
    damage: 1..2,
    text_key: MINIGAME_ENDLESS_SHOOTER_CLUSTER_GUN,
};

const TURNCOAT_GUN: Gun = Gun {
    cooldown: Duration::from_millis(500)..Duration::from_millis(600),
    count: 1,
    x_arc: 10,
    y_speed: 100..200,
    size: 0.3,
    damage: 1..2,
    text_key: MINIGAME_ENDLESS_SHOOTER_TURNCOAT_GUN,
};

#[derive(Component, Default)]
struct Bullet {
    damage: f32,
}

#[derive(Component)]
struct ClusterBullet {
    shells: i32,
}

#[derive(Component)]
struct TurnCoatBullet;

#[derive(Component)]
struct TurnCoat;

#[derive(Component)]
struct Walker {
    health: f32,
}

struct CooldownGate {
    starting: i32,
    gun_index: usize,
    max: i32,
}

struct GunGate {
    lock_threshold: i32,
    gun: Gun,
}

#[derive(Component)]
enum GateKind {
    Cooldown(CooldownGate),
    Gun(GunGate),
}

impl GateKind {
    fn text_key(&self, shooter: &Shooter) -> &'static str {
        match self {
            GateKind::Cooldown(cooldown) => match shooter.guns.get(cooldown.gun_index) {
                Some(owned_gun) => match owned_gun.gun {
                    PISTOL => MINIGAME_ENDLESS_SHOOTER_PISTOL_COOLDOWN,
                    MINIGUN => MINIGAME_ENDLESS_SHOOTER_MINIGUN_COOLDOWN,
                    WAVE_GUN => MINIGAME_ENDLESS_SHOOTER_WAVE_GUN_COOLDOWN,
                    ROCKET => MINIGAME_ENDLESS_SHOOTER_ROCKET_COOLDOWN,
                    CLUSTER_GUN => MINIGAME_ENDLESS_SHOOTER_CLUSTER_GUN_COOLDOWN,
                    TURNCOAT_GUN => MINIGAME_ENDLESS_SHOOTER_TURNCOAT_GUN_COOLDOWN,
                    _ => MINIGAME_ENDLESS_SHOOTER_COOLDOWN,
                },
                None => MINIGAME_ENDLESS_SHOOTER_COOLDOWN,
            },
            GateKind::Gun(val) => val.gun.text_key,
        }
    }
}

#[derive(Component, Default)]
struct GateBarrier {
    pre_collided_bullets: HashSet<Entity>,
    damage_taken: f32,
}

impl GateBarrier {
    pub fn score(&self) -> i32 {
        (self.damage_taken / 10.).ceil() as i32
    }
}

#[derive(Component)]
struct GatePair {
    connected: Vec<Entity>,
}

fn setup_camera_and_ui(
    mut commands: Commands,
    assets: Res<EndlessShooterAssets>,
    font_assets: Res<FontAssets>,
) {
    let ui_camera: Entity = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 0,
                    ..default()
                },
                ..default()
            },
            EndlessShooter,
        ))
        .id();

    commands
        .spawn((
            TargetCamera(ui_camera),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },

                ..default()
            },
            EndlessShooter,
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                image: UiImage::new(assets.background.clone()),
                ..default()
            });
        });

    commands
        .spawn((
            TargetCamera(ui_camera),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },

                ..default()
            },
            EndlessShooter,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    style: Style {
                        margin: UiRect::left(Val::Percent(85.)),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    text: Text::from_sections(vec![
                        TextSection::new(
                            "",
                            TextStyle {
                                font: font_assets.main_font.clone(),
                                font_size: 20.,
                                color: Color::BLACK,
                            },
                        ),
                        TextSection::new(
                            ":\n",
                            TextStyle {
                                font: font_assets.main_font.clone(),
                                font_size: 20.,
                                color: Color::BLACK,
                            },
                        ),
                        TextSection::new(
                            "",
                            TextStyle {
                                font: font_assets.main_font.clone(),
                                font_size: 20.,
                                color: Color::BLACK,
                            },
                        ),
                    ]),
                    ..default()
                },
                KeyText::new().with(0, MINIGAME_ENDLESS_SHOOTER_SCORE),
                ScoreText,
            ));
        });

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        MouseCamera,
        EndlessShooter,
        EndlessShooterCamera,
    ));
}

fn setup_shooter(
    mut commands: Commands,
    mut state: ResMut<NextState<EndlessShooterState>>,
    mut rng: ResMut<GlobalRng>,
    assets: Res<EndlessShooterAssets>,
    pet_sheet: Query<(&Handle<Image>, &TextureAtlas, &Sprite, &MoodImageIndexes), With<Playing>>,
) {
    state.set(EndlessShooterState::Playing);

    let (image, atlas, sprite, mood_images) = pet_sheet.single();

    const SPRITE_SIZE: f32 = 50.;

    let size = sprite.custom_size.unwrap();

    let size_x: f32;
    let size_y: f32;

    if size.x > size.y {
        let x = SPRITE_SIZE;
        let ratio = x / size.x;
        let y = size.y * ratio;
        size_x = x;
        size_y = y;
    } else {
        let y = SPRITE_SIZE;
        let ratio = y / size.y;
        let x = size.x * ratio;
        size_x = x;
        size_y = y;
    }

    commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    0.,
                    SHOOTER_Y,
                    ZLayer::Pet.to_f32(),
                )),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(size_x, size_y)),
                    ..default()
                },
                texture: image.clone(),
                ..default()
            },
            atlas.clone(),
            *mood_images,
            MoodCategory::Neutral,
            AutoSetMoodImage,
            MoveTowardsCursor::new().with_x(true),
            RngComponent::from(&mut *rng),
            Shooter::new(PISTOL, &mut *rng),
            EndlessShooterPet,
            EndlessShooter,
        ))
        .insert((
            LinearVelocity(Vec2::new(0., 0.)),
            Speed(300.),
            GravityScale(0.),
            RigidBody::Dynamic,
            Collider::rectangle(size_x / 2., size_y / 2.),
            CollisionLayers::new(
                [ColLayer::Shooter],
                [ColLayer::Gate, ColLayer::Walker, ColLayer::Wall],
            ),
        ))
        .with_children(|parent| {
            parent.spawn((
                SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(0., 0., 1.))
                        .with_scale(Vec3::new(1., 1., 1.)),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(100., 100.)),
                        ..default()
                    },
                    texture: assets.bubble.clone(),
                    ..default()
                },
                EndlessShooterHealth,
            ));
        });

    commands.spawn((Score::new(), EndlessShooter));
}

struct WeightedRandomness<T> {
    values: Vec<(f32, T)>,
}

impl<T> WeightedRandomness<T> {
    fn new(values: Vec<(f32, T)>) -> Self {
        if values.len() > 1 {
            for i in 0..(values.len() - 1) {
                if values[i].0 > values[i + 1].0 {
                    panic!("Values must be in ascending order");
                }
            }
        }

        Self { values }
    }

    fn pick<J: DelegatedRng>(&self, rng: &mut J) -> &T {
        let selected_num = rng.f32();
        let value = self
            .values
            .iter()
            .find(|(chance, _)| selected_num <= *chance)
            .unwrap_or(self.values.last().unwrap());
        &value.1
    }
}

struct WalkerTemplate {
    health: f32,
    size: f32,
    speed: f32,
}

const BASIC_WALKER: WalkerTemplate = WalkerTemplate {
    health: 10.,
    size: 0.35,
    speed: 40.,
};

const LARGE_WALKER: WalkerTemplate = WalkerTemplate {
    health: 25.,
    size: 0.4,
    speed: 50.,
};

const HUGE_WALKER: WalkerTemplate = WalkerTemplate {
    health: 50.,
    size: 0.9,
    speed: 30.,
};

const FAST_WALKER: WalkerTemplate = WalkerTemplate {
    health: 10.,
    size: 0.25,
    speed: 100.,
};

#[derive(Component)]
struct WalkerSpawner {
    cooldown: VaryingTimer,
    active_templates: WeightedRandomness<WalkerTemplate>,
}

enum GateTemplate {
    CooldownRange((Range<i32>, i32)),
    Gun((Range<i32>, Gun)),
}

impl GateTemplate {
    fn resolve<T: DelegatedRng>(&self, shooter: &Shooter, rng: &mut T) -> GateKind {
        match self {
            GateTemplate::CooldownRange(range) => GateKind::Cooldown(CooldownGate {
                starting: rng.i32(range.0.clone()),
                gun_index: rng.usize(0..shooter.guns.len()),
                max: range.1,
            }),
            GateTemplate::Gun(gun_gate) => {
                let lock_threshold = rng.i32(gun_gate.0.clone());
                GateKind::Gun(GunGate {
                    lock_threshold,
                    gun: gun_gate.1.clone(),
                })
            }
        }
    }
}

#[derive(Component)]
struct GateSpawner {
    bullet_gate_cooldown: VaryingTimer,
    templates: WeightedRandomness<GateTemplate>,
}

fn setup_spawners(mut commands: Commands, rng: ResMut<GlobalRng>) {
    let rng = rng.into_inner();

    commands.spawn((
        WalkerSpawner {
            cooldown: VaryingTimer::new(
                Duration::from_millis(400)..Duration::from_millis(500),
                rng,
            )
            .with_modifier(2.),
            active_templates: WeightedRandomness::new(vec![
                (0.1, HUGE_WALKER),
                (0.2, FAST_WALKER),
                (0.3, LARGE_WALKER),
                (1., BASIC_WALKER),
            ]),
        },
        RngComponent::from(&mut *rng),
        EndlessShooter,
    ));

    commands.spawn((
        GateSpawner {
            bullet_gate_cooldown: VaryingTimer::new(
                Duration::from_secs(5)..Duration::from_secs(10),
                rng,
            ),
            templates: WeightedRandomness::new(vec![
                (0.05, GateTemplate::Gun((-20..-10, MINIGUN))),
                (0.1, GateTemplate::Gun((-20..-10, ROCKET))),
                (0.15, GateTemplate::Gun((-20..-10, CLUSTER_GUN))),
                (0.2, GateTemplate::Gun((-20..-10, WAVE_GUN))),
                (0.25, GateTemplate::Gun((-20..-10, TURNCOAT_GUN))),
                (1., GateTemplate::CooldownRange((-50..20, 30))),
            ]),
        },
        RngComponent::from(&mut *rng),
        EndlessShooter,
    ));
}

#[derive(Component)]
struct Wall;

fn setup_walls(mut commands: Commands) {
    commands.spawn((
        Transform::from_translation(Vec3::new(-200., 0., ZLayer::Background.to_f32())),
        RigidBody::Static,
        Collider::rectangle(100., GAME_HEIGHT as f32 * 100.),
        CollisionLayers::new([ColLayer::Wall], [ColLayer::Shooter, ColLayer::Walker]),
        Wall,
        EndlessShooter,
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(200., 0., ZLayer::Background.to_f32())),
        RigidBody::Static,
        Collider::rectangle(100., GAME_HEIGHT as f32 * 100.),
        CollisionLayers::new([ColLayer::Wall], [ColLayer::Shooter, ColLayer::Walker]),
        Wall,
        EndlessShooter,
    ));
}

fn teardown(mut commands: Commands, to_delete: Query<Entity, With<EndlessShooter>>) {
    info!("Tearing down endless shooter");
    for entity in &to_delete {
        commands.entity(entity).despawn_recursive();
    }
}

enum ExtraBulletSpawnInfo {
    None,
    Cluster,
    TurnCoat,
}

#[derive(Event)]
struct BulletSpawnEvent {
    direction: Vec2,
    location: Vec2,
    gates_excluded: HashSet<Entity>,
    damage: f32,
    size: f32,
    extra: ExtraBulletSpawnInfo,
    mass: f32,
}

fn queue_spawn_bullets(
    time: Res<Time>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
    mut pet: Query<
        (&mut Shooter, &mut RngComponent, &GlobalTransform, &Sprite),
        With<EndlessShooterPet>,
    >,
) {
    for (mut pet_shooter, mut pet_rng, pet_trans, sprite) in &mut pet {
        let y_speed_bonus = pet_shooter.y_speed_bonus;
        let shooter_multiplier = pet_shooter.multiplier;
        let pet_damage = pet_shooter.damage;
        for owned_gun in &mut pet_shooter.guns {
            let to_spawn = owned_gun
                .last_shot
                .tick(time.delta(), &mut *pet_rng)
                .times_finished_this_tick() as i32
                * shooter_multiplier;

            for _ in 0..to_spawn {
                let x_direction =
                    pet_rng.i32(-owned_gun.gun.x_arc..owned_gun.gun.x_arc) as f32 + pet_rng.f32();
                let y_direction = pet_rng.i32(owned_gun.gun.y_speed.clone()) as f32
                    + pet_rng.f32()
                    + y_speed_bonus;

                let extra = match owned_gun.gun {
                    CLUSTER_GUN => ExtraBulletSpawnInfo::Cluster,
                    TURNCOAT_GUN => ExtraBulletSpawnInfo::TurnCoat,
                    _ => ExtraBulletSpawnInfo::None,
                };

                let mass = match owned_gun.gun {
                    TURNCOAT_GUN => 0.,
                    _ => owned_gun.gun.size + 3.,
                };

                bullet_spawn_events.send(BulletSpawnEvent {
                    direction: Vec2::new(x_direction, y_direction),
                    location: Vec2::new(
                        pet_trans.translation().x,
                        pet_trans.translation().y + sprite.custom_size.unwrap().y / 2. + 3.,
                    ),
                    gates_excluded: HashSet::default(),
                    damage: pet_rng.i32(owned_gun.gun.damage.clone()) as f32 + pet_damage,
                    size: owned_gun.gun.size,
                    extra,
                    mass,
                });
            }
        }
    }
}

fn lock_shooter_y(mut shooter: Query<&mut Transform, With<EndlessShooterPet>>) {
    for mut shooter in &mut shooter {
        shooter.translation.y = SHOOTER_Y;
    }
}

fn spawn_bullets(
    mut commands: Commands,
    assets: Res<EndlessShooterAssets>,
    mut bullet_spawn_events: EventReader<BulletSpawnEvent>,
    mut gates: Query<&mut GateBarrier, With<EndlessShooter>>,
) {
    for event in bullet_spawn_events.read() {
        let new_bullet = commands
            .spawn((
                SpriteBundle {
                    transform: Transform::from_translation(Vec3::new(
                        event.location.x,
                        event.location.y,
                        ZLayer::Bullet.to_f32(),
                    ))
                    .with_scale(Vec3::new(event.size, event.size, 1.)),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(60., 60.)),
                        ..default()
                    },
                    texture: assets.discs.clone(),
                    ..default()
                },
                TextureAtlas {
                    layout: assets.layout.clone(),
                    index: 1,
                },
                RigidBody::Kinematic,
                Collider::circle(30.0),
                ColliderDensity(0.0),
                Mass(event.mass),
                CollisionLayers::new([ColLayer::Bullet], [ColLayer::Walker, ColLayer::Gate]),
                LinearVelocity(event.direction),
                Bullet {
                    damage: event.damage,
                },
                EndlessShooter,
            ))
            .id();

        match event.extra {
            ExtraBulletSpawnInfo::Cluster => {
                commands
                    .entity(new_bullet)
                    .insert(ClusterBullet { shells: 10 });
            }
            ExtraBulletSpawnInfo::TurnCoat => {
                commands.entity(new_bullet).insert(TurnCoatBullet);
            }
            _ => {}
        }

        for gate in event.gates_excluded.iter() {
            if let Ok(mut gate) = gates.get_mut(*gate) {
                gate.pre_collided_bullets.insert(new_bullet);
            }
        }
    }
}

fn tick_bullets(
    mut commands: Commands,
    bullets: Query<(Entity, &GlobalTransform), (With<Bullet>, With<EndlessShooter>)>,
) {
    for (entity, bullet) in bullets.iter() {
        if bullet.translation().y > GAME_TOP_Y as f32
            || bullet.translation().y < GAME_BOTTOM_Y as f32
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}

struct SpawnWalkerIncrease {
    increase_timer: Timer,
}

impl Default for SpawnWalkerIncrease {
    fn default() -> Self {
        Self {
            increase_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }
}

fn spawn_walkers(
    mut local: Local<SpawnWalkerIncrease>,
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<EndlessShooterAssets>,
    mut spawner: Query<(&mut WalkerSpawner, &mut RngComponent), With<EndlessShooter>>,
) {
    let (mut spawner, rng) = spawner.single_mut();

    let rng = rng.into_inner();

    let to_spawn = spawner
        .cooldown
        .tick(time.delta(), rng)
        .times_finished_this_tick();

    if local.increase_timer.tick(time.delta()).just_finished() {
        spawner.cooldown.modifier += 0.1;
    }

    const SPAWN_ZONE_X: i32 = (GAME_WIDTH) / 2;

    for _ in 0..to_spawn {
        let x_spawn = rng.i32(-SPAWN_ZONE_X..SPAWN_ZONE_X) as f32;
        let y_spawn = rng.i32((GAME_TOP_Y + 30)..(GAME_TOP_Y + 50)) as f32;

        let template = spawner.active_templates.pick(rng);

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    x_spawn,
                    y_spawn,
                    ZLayer::Walker.to_f32(),
                ))
                .with_scale(Vec3::new(template.size, template.size, 1.)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(60., 60.)),
                    ..default()
                },
                texture: assets.discs.clone(),
                ..default()
            },
            TextureAtlas {
                layout: assets.layout.clone(),
                index: 2,
            },
            Walker {
                health: template.health,
            },
            Speed(template.speed),
            RigidBody::Dynamic,
            Collider::circle(30.),
            GravityScale(0.),
            ColliderDensity(0.0),
            Mass(template.size),
            CollisionLayers::new(
                [ColLayer::Walker],
                [
                    ColLayer::Bullet,
                    ColLayer::Shooter,
                    ColLayer::Walker,
                    ColLayer::Wall,
                ],
            ),
            LinearVelocity(Vec2::new(0., 0.)),
            EndlessShooter,
        ));
    }
}

#[derive(Default)]
struct UpdateWalkerTracker {
    divisor: u32,
}

fn update_walker_direction(
    rng: ResMut<GlobalRng>,
    mut update_tracker: Local<UpdateWalkerTracker>,
    player: Query<&GlobalTransform, With<EndlessShooterPet>>,
    walker_transforms: Query<&GlobalTransform, With<Walker>>,
    mut walkers: Query<(Entity, &Speed, &mut LinearVelocity), With<Walker>>,
    non_turncoats: Query<Entity, (Without<TurnCoat>, With<Walker>)>,
    turncoats: Query<Entity, (With<TurnCoat>, With<Walker>)>,
) {
    const DIVISOR_SIZE: u32 = 30;

    let rng = rng.into_inner();
    let player = player.single();

    let player_location = player.translation().xy();

    let non_turncoats: Vec<Entity> = non_turncoats.iter().collect::<Vec<_>>();

    for (entity, speed, mut velocity) in walkers.iter_mut() {
        let trans = walker_transforms.get(entity).unwrap();
        if entity.index() % DIVISOR_SIZE != update_tracker.divisor {
            continue;
        }

        let target = if turncoats.get(entity).is_ok() {
            if non_turncoats.is_empty() {
                continue;
            }
            walker_transforms
                .get(non_turncoats[rng.usize(0..non_turncoats.len())])
                .unwrap()
                .translation()
                .xy()
        } else {
            player_location
        };

        let direction = Vec2::normalize(target - trans.translation().xy());
        *velocity = LinearVelocity(direction * speed.0);
    }

    update_tracker.divisor += 1;
    if update_tracker.divisor > DIVISOR_SIZE {
        update_tracker.divisor = 0;
    }
}

fn bullet_hit(
    mut commands: Commands,
    rng: ResMut<GlobalRng>,
    mut spawn_bullets: EventWriter<BulletSpawnEvent>,
    mut bullets: Query<(Entity, &mut Bullet, &GlobalTransform, &CollidingEntities)>,
    cluster_bullets: Query<&ClusterBullet>,
    turncoat_bullets: Query<&TurnCoatBullet>,
    mut walkers: Query<&mut Walker>,
    mut gates: Query<&mut GateBarrier>,
    mut score: Query<&mut Score>,
) {
    let mut score = score.single_mut();
    let rng = rng.into_inner();

    for (bullet_ent, mut bullet, trans, colliding) in &mut bullets.iter_mut() {
        for col_ent in &colliding.0 {
            let col_ent = *col_ent;
            if let Ok(mut walker) = walkers.get_mut(col_ent) {
                let damage = bullet.damage;
                bullet.damage -= walker.health;
                // This should be done somewhere else
                walker.health -= damage;
                if walker.health > 0. {
                    if let Some(mut entity) = commands.get_entity(col_ent) {
                        entity.try_insert(DamageFade::default());
                    }
                } else {
                    score.kills += 1;
                }
                if bullet.damage <= 0. {
                    if let Ok(cluster) = cluster_bullets.get(bullet_ent) {
                        for _ in 0..cluster.shells {
                            let angle = rng.i32(0..360) as f32;
                            let dir = Vec2::new(angle.cos(), angle.sin()) * 500.;
                            spawn_bullets.send(BulletSpawnEvent {
                                direction: dir,
                                location: trans.translation().xy(),
                                gates_excluded: HashSet::from([col_ent]),
                                damage: rng.i32(0..30) as f32,
                                size: 0.1,
                                extra: ExtraBulletSpawnInfo::None,
                                mass: 0.3,
                            });
                        }
                    } else if turncoat_bullets.get(bullet_ent).is_ok() {
                        if let Some(mut entity) = commands.get_entity(col_ent) {
                            entity.insert(TurnCoat);
                        }
                    }
                    commands.entity(bullet_ent).despawn_recursive();
                }
            } else if let Ok(mut gate) = gates.get_mut(col_ent) {
                if gate.pre_collided_bullets.contains(&bullet_ent) {
                    continue;
                }
                gate.pre_collided_bullets.insert(bullet_ent);
                gate.damage_taken += bullet.damage;

                // let x_mod = (vel.x * 0.2).ceil().max(1.) as i32;
                // let x_vel = vel.x + rng.i32(-x_mod..x_mod) as f32;
                // let y_vel = {
                //     if rng.bool() {
                //         rng.f32()
                //     } else {
                //         -rng.f32()
                //     }
                // } + vel.y;

                // spawn_bullets.send(BulletSpawnEvent {
                //     direction: Vec2::new(x_vel, y_vel),
                //     location: Vec2::new(trans.translation().x, trans.translation().y),
                //     gates_excluded: HashSet::from([col_ent]),
                // });
            }
        }
    }
}

fn shooter_hit(
    mut commands: Commands,
    mut spawn_bullets: EventWriter<BulletSpawnEvent>,
    mut shooter: Query<
        (
            Entity,
            &CollidingEntities,
            &mut Shooter,
            &mut RngComponent,
            &GlobalTransform,
        ),
        With<EndlessShooterPet>,
    >,
    invincible_shooters: Query<Entity, (With<Invincibility>, With<EndlessShooterPet>)>,
    gates: Query<(&GateKind, &GateBarrier, &GatePair)>,
    mut walkers: Query<&mut Walker>,
) {
    for (entity, colliding, mut shooter, rng, trans) in &mut shooter {
        let rng = rng.into_inner();
        let mut already_hit = false;

        for col_ent in &colliding.0 {
            if let Ok((gate, barrier, pair)) = gates.get(*col_ent) {
                match gate {
                    GateKind::Cooldown(value) => {
                        let change = (value.starting + barrier.score()).min(value.max);
                        for owned_gun in &mut shooter.guns {
                            owned_gun.last_shot.modifier += change as f64 / 1000.;
                            if owned_gun.last_shot.modifier < 1. {
                                owned_gun.last_shot.modifier = 1.;
                            }
                        }
                        let new_mood = if change > 0 {
                            MoodCategory::Happy
                        } else {
                            MoodCategory::Sad
                        };
                        commands
                            .entity(entity)
                            .insert(TemporaryMoodChange::new(new_mood, Duration::from_secs(2)));

                        for gate in &pair.connected {
                            if let Some(entity) = commands.get_entity(*gate) {
                                entity.despawn_recursive();
                            }
                        }
                    }
                    GateKind::Gun(gun_gate) => {
                        let number = (gun_gate.lock_threshold + barrier.score()).min(0);

                        let mood = if number >= 0 {
                            if let Some(index) =
                                shooter.guns.iter().position(|gun| gun.gun == gun_gate.gun)
                            {
                                shooter.guns[index].last_shot.modifier += 1.;
                            } else {
                                shooter.guns.push(OwnedGun::new(gun_gate.gun.clone(), rng));
                            }
                            MoodCategory::Ecstatic
                        } else {
                            MoodCategory::Sad
                        };

                        commands
                            .entity(entity)
                            .insert(TemporaryMoodChange::new(mood, Duration::from_secs(5)));

                        for gate in &pair.connected {
                            if let Some(entity) = commands.get_entity(*gate) {
                                entity.despawn_recursive();
                            }
                        }
                    }
                }
            } else if let Ok(mut walker) = walkers.get_mut(*col_ent) {
                walker.health = 0.;
                if invincible_shooters.get(entity).is_ok() || already_hit {
                    continue;
                }
                already_hit = true;
                shooter.health -= 1;
                for _ in 0..1000 {
                    let angle = rng.i32(0..360) as f32;
                    let dir = Vec2::new(angle.cos(), angle.sin()) * 100.;
                    spawn_bullets.send(BulletSpawnEvent {
                        direction: dir,
                        location: trans.translation().xy(),
                        gates_excluded: HashSet::default(),
                        damage: rng.i32(0..30) as f32,
                        size: 0.05,
                        extra: ExtraBulletSpawnInfo::None,
                        mass: 0.3,
                    });
                }
                commands.entity(entity).insert((
                    TemporaryMoodChange::new(MoodCategory::Sad, Duration::from_secs(2)),
                    Invincibility::new(Duration::from_secs(5)),
                ));
            }
        }
    }
}

fn walker_hit(
    mut commands: Commands,
    mut walker_health: Query<&mut Walker>,
    walkers: Query<(Entity, &CollidingEntities), With<Walker>>,
    turncoat: Query<Entity, (With<TurnCoat>, With<Walker>)>,
) {
    for (entity, colliding) in &mut walkers.iter() {
        let is_turncoat = turncoat.get(entity).is_ok();

        for col_ent in &colliding.0 {
            if walkers.get(*col_ent).is_ok() && is_turncoat && turncoat.get(*col_ent).is_err() {
                let damage = walker_health.get(entity).unwrap().health;
                {
                    let mut health = walker_health.get_mut(*col_ent).unwrap();
                    health.health -= damage;
                    if health.health > 0. {
                        if let Some(mut entity) = commands.get_entity(*col_ent) {
                            entity.try_insert(DamageFade::default());
                        }
                    }
                }
                walker_health.get_mut(entity).unwrap().health = 0.;
            }
        }
    }
}

#[derive(Component)]
struct GateText;

fn spawn_gate(
    mut commands: Commands,
    time: Res<Time>,
    font_assets: Res<FontAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    rng: ResMut<GlobalRng>,
    shooter: Query<&Shooter, With<EndlessShooterPet>>,
    mut gate_spawner: Query<&mut GateSpawner>,
) {
    let mut gate_spawner = gate_spawner.single_mut();

    const GATE_WIDTH: f32 = (GAME_WIDTH / 2) as f32;
    const GATE_HEIGHT: f32 = 20.0;
    const X_LOCATION_TABLE: [f32; 2] = [-GATE_WIDTH / 2., GATE_WIDTH / 2.];
    const GATE_COLORS: [Color; 2] = [
        Color::linear_rgba(1.0, 1.0, 0., 0.4),
        Color::linear_rgba(1.0, 0.0, 0., 0.4),
    ];

    let rng = rng.into_inner();

    let shooter = shooter.single();

    if gate_spawner
        .bullet_gate_cooldown
        .tick(time.delta(), rng)
        .just_finished()
    {
        let kinds = (0..2)
            .map(|_| gate_spawner.templates.pick(rng).resolve(shooter, rng))
            .collect::<Vec<_>>();

        let mut spawned = Vec::new();

        for (i, kind) in kinds.into_iter().enumerate() {
            let text_ent = commands
                .spawn((
                    Text2dBundle {
                        transform: Transform::from_xyz(0., 0., ZLayer::GateText.to_f32()),
                        text: Text::from_sections([
                            TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.main_font.clone(),
                                    font_size: 15.,
                                    color: Color::WHITE,
                                },
                            ),
                            TextSection::new(
                                "\n",
                                TextStyle {
                                    font: font_assets.main_font.clone(),
                                    font_size: 1.,
                                    color: Color::WHITE,
                                },
                            ),
                            TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.main_font.clone(),
                                    font_size: 17.,
                                    color: Color::WHITE,
                                },
                            ),
                        ]),
                        ..default()
                    },
                    KeyText {
                        keys: hashmap! { 0 => KeyString::direct(kind.text_key(shooter)) },
                    },
                    GateText,
                    EndlessShooter,
                ))
                .id();

            let gate = commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(
                            meshes.add(Rectangle::new(GATE_WIDTH, GATE_HEIGHT + GATE_HEIGHT * 0.5)),
                        ),
                        material: materials.add(GATE_COLORS[i]),
                        transform: Transform::from_xyz(
                            X_LOCATION_TABLE[i],
                            GAME_HEIGHT as f32,
                            ZLayer::Gate.to_f32(),
                        ),
                        ..default()
                    },
                    kind,
                    GateBarrier::default(),
                    RigidBody::Kinematic,
                    Collider::rectangle(GATE_WIDTH - 10., GATE_HEIGHT),
                    CollisionLayers::new([ColLayer::Gate], [ColLayer::Bullet, ColLayer::Shooter]),
                    LinearVelocity(Vec2::new(0., -50.)),
                    EndlessShooter,
                ))
                .add_child(text_ent)
                .id();

            spawned.push(gate);
        }

        for gate in spawned.clone() {
            commands.entity(gate).insert(GatePair {
                connected: spawned.clone(),
            });
        }
    }
}

fn update_gate_color(
    gates: Query<
        (&GateBarrier, &GateKind, &Handle<ColorMaterial>),
        Or<(Added<GateBarrier>, Changed<GateBarrier>)>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for (barrier, kind, material) in gates.iter() {
        match kind {
            GateKind::Cooldown(value) => {
                let number: i32 = (value.starting + barrier.score()).min(value.max);
                if number > 0 {
                    materials.get_mut(material.id()).unwrap().color =
                        Color::linear_rgba(0., 0., 1., 0.4);
                } else {
                    materials.get_mut(material.id()).unwrap().color =
                        Color::linear_rgba(1., 0., 0., 0.4);
                }
            }
            GateKind::Gun(gun_gate) => {
                let number: i32 = (gun_gate.lock_threshold + barrier.score()).min(0);
                if number >= 0 {
                    materials.get_mut(material.id()).unwrap().color =
                        Color::linear_rgba(0., 1., 0., 0.4);
                } else {
                    materials.get_mut(material.id()).unwrap().color =
                        Color::linear_rgba(1., 0., 0., 0.4);
                }
            }
        }
    }
}

fn update_gate_text(
    gates: Query<(&Children, &GateBarrier, &GateKind), Changed<GateBarrier>>,
    mut gate_text: Query<&mut Text, With<GateText>>,
) {
    for (children, barrier, bullet_gate) in gates.iter() {
        for child in children.iter() {
            if let Ok(mut text) = gate_text.get_mut(*child) {
                let mut updated_text = match bullet_gate {
                    GateKind::Cooldown(value) => {
                        let number = (value.starting + barrier.score()).min(value.max);
                        let mut updated_text = String::new();
                        match number {
                            n if n > 0 => updated_text.push('+'),
                            n if n < 0 => updated_text.push('-'),
                            _ => {}
                        }
                        updated_text.push_str(&number.abs().to_string());
                        updated_text
                    }
                    GateKind::Gun(gun_gate) => {
                        let number = (gun_gate.lock_threshold + barrier.score()).min(0);
                        let mut updated_text = String::new();
                        if number < 0 {
                            updated_text.push('-');
                        }
                        updated_text.push_str(&number.abs().to_string());
                        updated_text
                    }
                };
                let width = text.sections[0].value.len();
                // prefix the text with spaces
                if updated_text.len() < width {
                    let delta = (width - updated_text.len()) / 2;
                    let spaces = " ".repeat(delta);
                    updated_text = spaces + &updated_text;
                }

                text.sections[2].value = updated_text.to_string();
            }
        }
    }
}

#[derive(Component)]
struct TemporaryMoodChange {
    old_mood: Option<MoodCategory>,
    new_mood: MoodCategory,
    time: Timer,
}

impl TemporaryMoodChange {
    fn new(new_mood: MoodCategory, time: Duration) -> Self {
        Self {
            old_mood: None,
            new_mood,
            time: Timer::new(time, TimerMode::Once),
        }
    }
}

fn temp_mood_added(
    mut pet: Query<
        (&mut MoodCategory, &mut TemporaryMoodChange),
        (Added<TemporaryMoodChange>, With<EndlessShooter>),
    >,
) {
    for (mut mood, mut temp_mood) in pet.iter_mut() {
        temp_mood.old_mood = Some(*mood);
        *mood = temp_mood.new_mood;
        info!("Setting mood to {:?}", temp_mood.new_mood);
    }
}

fn update_mood(
    mut commands: Commands,
    time: Res<Time>,
    mut pet: Query<(Entity, &mut MoodCategory, &mut TemporaryMoodChange), With<EndlessShooter>>,
) {
    for (entity, mut mood, mut temp_mood) in pet.iter_mut() {
        if temp_mood.time.tick(time.delta()).finished() && temp_mood.old_mood.is_some() {
            info!("Setting mood to {:?}", temp_mood.old_mood);
            *mood = temp_mood.old_mood.unwrap();
            commands.entity(entity).remove::<TemporaryMoodChange>();
        }
    }
}

#[derive(Component)]
struct DamageFade {
    time: Timer,
}

impl Default for DamageFade {
    fn default() -> Self {
        Self {
            time: Timer::new(Duration::from_millis(100), TimerMode::Once),
        }
    }
}

fn update_damage_fade(
    mut commands: Commands,
    time: Res<Time>,
    mut damage_fade: Query<(Entity, &mut DamageFade, &mut Sprite), Without<TurnCoat>>,
) {
    for (entity, mut fade, mut sprite) in damage_fade.iter_mut() {
        if fade.time.tick(time.delta()).finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<DamageFade>();
        } else {
            let percent = fade.time.elapsed().as_secs_f32() / fade.time.duration().as_secs_f32();
            sprite.color = Color::linear_rgb(1. - percent, 0., 0.);
        }
    }
}

fn update_tint_turncoat(mut turncoats: Query<&mut Sprite, Added<TurnCoat>>) {
    for mut sprite in &mut turncoats {
        sprite.color = Color::linear_rgb(0., 1., 0.);
    }
}

fn kill_walkers(
    mut commands: Commands,
    assets: Res<EndlessShooterAssets>,
    walkers: Query<
        (
            Entity,
            &Walker,
            &GlobalTransform,
            &Transform,
            &Sprite,
            &TextureAtlas,
        ),
        Changed<Walker>,
    >,
) {
    for (entity, walker, global_trans, trans, sprite, atlas) in &walkers {
        if walker.health <= 0. || global_trans.translation().y < GAME_BOTTOM_Y as f32 + 15. {
            let mut sprite = sprite.clone();
            sprite.color = Color::srgb_u8(255, 165, 0);
            commands.spawn((
                SpriteBundle {
                    sprite,
                    transform: Transform::from_translation(Vec3::new(
                        trans.translation.x,
                        trans.translation.y,
                        ZLayer::DeadWalker.to_f32(),
                    ))
                    .with_scale(trans.scale),
                    texture: assets.discs.clone(),
                    ..default()
                },
                atlas.clone(),
                Shrinking::new(trans.scale.xy(), Duration::from_secs(5)),
                EndlessShooter,
            ));
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn toggle_walker_bodies(mut walkers: Query<(&mut RigidBody, &GlobalTransform), With<Walker>>) {
    for (mut body, trans) in &mut walkers {
        if trans.translation().y > GAME_TOP_Y as f32 + 20. {
            *body = RigidBody::Kinematic;
        } else {
            *body = RigidBody::Dynamic;
        }
    }
}

#[derive(Component)]
struct EndlessShooterHealth;

fn update_shooter_shield(
    shooter: Query<&Shooter, (With<EndlessShooterPet>, Changed<Shooter>)>,
    mut health: Query<(&mut Sprite, &Parent), With<EndlessShooterHealth>>,
) {
    for (mut sprite, parent) in &mut health {
        if let Ok(shooter) = shooter.get(parent.get()) {
            match shooter.health {
                0 => {}
                1 => sprite.color = Color::linear_rgba(0., 0., 0., 0.),
                2 => sprite.color = Color::linear_rgba(1., 1., 1., 0.3),
                _ => sprite.color = Color::linear_rgba(1., 1., 1., 0.7),
            }
        }
    }
}

fn check_game_over(
    mut state: ResMut<NextState<EndlessShooterState>>,
    shooter: Query<&Shooter, With<EndlessShooterPet>>,
) {
    let shooter = shooter.single();

    if shooter.health <= 0 {
        state.set(EndlessShooterState::GameOver);
    }
}

fn send_complete(mut event_writer: EventWriter<MiniGameCompleted>, score: Query<&Score>) {
    let score = score.single().calc();

    event_writer.send(MiniGameCompleted {
        game_type: MiniGameType::EndlessShooter,
        result: if score > 10000. {
            MiniGameResult::Lose
        } else if score > 5000. {
            MiniGameResult::Draw
        } else {
            MiniGameResult::Win
        },
    });
}

#[derive(Component)]
struct Score {
    kills: i32,
    duration_survived: Duration,
}

impl Score {
    fn new() -> Self {
        Self {
            kills: 0,
            duration_survived: Duration::ZERO,
        }
    }

    fn calc(&self) -> f32 {
        self.kills as f32 + self.duration_survived.as_secs_f32() / 100.
    }
}

#[derive(Component)]
struct ScoreText;

fn tick_score(
    time: Res<Time>,
    mut score: Query<&mut Score, With<Score>>,
    mut text: Query<&mut Text, With<ScoreText>>,
) {
    let mut score = score.single_mut();
    let mut text = text.single_mut();

    score.duration_survived += time.delta();

    text.sections[2].value = format!("{:.0}", score.calc());
}

#[derive(Component)]
struct Invincibility {
    time: Timer,
    flash_timer: Timer,
}

impl Invincibility {
    fn new(time: Duration) -> Self {
        Self {
            time: Timer::new(time, TimerMode::Once),
            flash_timer: Timer::new(Duration::from_millis(250), TimerMode::Repeating),
        }
    }
}

fn tick_invincibility(
    mut commands: Commands,
    time: Res<Time>,
    mut sprites: Query<&mut Sprite, With<EndlessShooter>>,
    mut invincibility: Query<(Entity, &mut Invincibility, &Children), With<EndlessShooter>>,
) {
    for (entity, mut invincibility, children) in &mut invincibility {
        if invincibility.time.tick(time.delta()).just_finished() {
            for child in children.iter().chain(std::iter::once(&entity)) {
                if let Ok(mut sprite) = sprites.get_mut(*child) {
                    sprite.color = Color::WHITE;
                }
            }

            commands.entity(entity).remove::<Invincibility>();
        } else if invincibility.flash_timer.tick(time.delta()).just_finished() {
            for child in children.iter().chain(std::iter::once(&entity)) {
                if let Ok(mut sprite) = sprites.get_mut(*child) {
                    sprite.color = if sprite.color == Color::WHITE {
                        Color::linear_rgba(1., 1., 1., 0.)
                    } else {
                        Color::WHITE
                    };
                }
            }
        }
    }
}

fn setup_game_over(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    score: Query<&Score>,
    to_remove: Query<
        Entity,
        Or<(
            With<ScoreText>,
            With<GateBarrier>,
            With<Shooter>,
            With<Bullet>,
            With<Shrinking>,
        )>,
    >,
) {
    pub const BUTTON_SET: ButtonColorSet =
        ButtonColorSet::new(PALE_PINK, VERY_LIGHT_PINK_RED, Color::WHITE);
    pub const BUTTON_BORDER_SET: ButtonColorSet = ButtonColorSet::new(
        LIGHT_PINK,
        Color::WHITE,
        Color::Srgba(bevy::color::palettes::css::LIMEGREEN),
    );

    for entity in &to_remove {
        commands.entity(entity).despawn_recursive();
    }

    let score = score.single().calc();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },

                ..default()
            },
            EndlessShooter,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    text: Text::from_sections(vec![
                        TextSection::new(
                            "",
                            TextStyle {
                                font: font_assets.main_font.clone(),
                                font_size: 50.,
                                color: Color::BLACK,
                            },
                        ),
                        TextSection::new(
                            "\n",
                            TextStyle {
                                font: font_assets.main_font.clone(),
                                font_size: 50.,
                                color: Color::BLACK,
                            },
                        ),
                        TextSection::new(
                            format!("{:.0}", score),
                            TextStyle {
                                font: font_assets.main_font.clone(),
                                font_size: 50.,
                                color: Color::BLACK,
                            },
                        ),
                    ]),
                    ..default()
                },
                KeyText::new().with(0, MINIGAME_ENDLESS_SHOOTER_SCORE),
                ScoreText,
            ));
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Percent(50.),
                            height: Val::Percent(10.),
                            ..default()
                        },
                        ..default()
                    },
                    ButtonHover::default()
                        .with_background(BUTTON_SET)
                        .with_border(BUTTON_BORDER_SET),
                    QuitButton,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            text: Text::from_section(
                                "",
                                TextStyle {
                                    font: font_assets.main_font.clone(),
                                    font_size: 50.,
                                    color: Color::BLACK,
                                },
                            ),
                            ..default()
                        },
                        KeyText::new().with(0, MINIGAME_ENDLESS_SHOOTER_QUIT),
                    ));
                });
        });
}

#[derive(Component)]
struct QuitButton;

const GAME_OVER_Y_CUTOFF: f32 = GAME_BOTTOM_Y as f32 - 100.;

fn update_walker_direction_game_over(
    mut walkers: Query<(Entity, &Speed, &mut LinearVelocity, &GlobalTransform), With<Walker>>,
) {
    const TARGETS: [Vec2; 7] = [
        Vec2::new(-150., GAME_OVER_Y_CUTOFF),
        Vec2::new(-100., GAME_OVER_Y_CUTOFF),
        Vec2::new(-50., GAME_OVER_Y_CUTOFF),
        Vec2::new(0., GAME_OVER_Y_CUTOFF),
        Vec2::new(50., GAME_OVER_Y_CUTOFF),
        Vec2::new(100., GAME_OVER_Y_CUTOFF),
        Vec2::new(150., GAME_OVER_Y_CUTOFF),
    ];

    for (entity, speed, mut velocity, trans) in &mut walkers {
        let target = TARGETS[entity.index() as usize % TARGETS.len()];
        let direction = Vec2::normalize(target - trans.translation().xy());
        *velocity = LinearVelocity(direction * speed.0);
    }
}

fn despawn_walkers_game_over(
    mut commands: Commands,
    walkers: Query<(Entity, &GlobalTransform), With<Walker>>,
) {
    for (entity, trans) in &walkers {
        if trans.translation().y < GAME_OVER_Y_CUTOFF {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn quit_button_pressed(
    mut state: ResMut<NextState<EndlessShooterState>>,
    button: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
) {
    for interaction in &mut button.iter() {
        if let Interaction::Pressed = interaction {
            state.set(EndlessShooterState::Exit);
        }
    }
}
