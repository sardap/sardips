use std::time::Duration;

use bevy::prelude::*;
use sardips_core::{
    despawn_all,
    interaction::MouseCamera,
    minigames_core::{MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType},
    velocity::Speed,
};
use shared_deps::{
    avian3d::prelude::*,
    bevy_rts_camera::{Ground, RtsCamera, RtsCameraControls},
    bevy_turborand::{DelegatedRng, GlobalRng, RngComponent},
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use templates::{BaseBattlerTemplate, LINE_MAN_TEMPLATE, TemplatePlugin};

mod templates;

pub struct RectClashPlugin;

impl Plugin for RectClashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(RectClashState::default())
            .add_event::<SpawnBattler>()
            .add_plugins(TemplatePlugin)
            .add_systems(
                OnEnter(MiniGameState::PlayingRectClash),
                on_start_playing_rect_clash,
            )
            .add_systems(
                OnExit(MiniGameState::PlayingRectClash),
                despawn_all::<RectClash>,
            )
            .add_systems(OnEnter(RectClashState::Loading), setup_loading)
            .add_systems(
                Update,
                check_loading.run_if(in_state(RectClashState::Loading)),
            )
            .add_systems(
                OnExit(RectClashState::Loading),
                despawn_all::<RectClashLoading>,
            )
            .add_systems(
                OnEnter(RectClashState::Playing),
                (setup_camera_and_ui, setup_world, spawn_starting_battlers),
            )
            .add_systems(
                OnExit(RectClashState::Playing),
                despawn_all::<RectClashPlaying>,
            )
            .add_systems(
                Update,
                (
                    spawn_battlers,
                    debug_spawn_battlers,
                    despawn_out_of_bounds,
                    setup_independent_brain,
                    find_closest_enemy,
                    move_towards_target,
                    battlers_collision,
                )
                    .run_if(in_state(RectClashState::Playing)),
            )
            .add_systems(OnEnter(RectClashState::Score), setup_score_screen)
            .add_systems(OnExit(RectClashState::Score), despawn_all::<RectClashScore>)
            .add_systems(
                Update,
                quit_button_pressed.run_if(in_state(RectClashState::Score)),
            )
            .add_systems(
                OnEnter(RectClashState::Exit),
                (despawn_all::<RectClash>, on_exit),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum RectClashState {
    #[default]
    None,
    Loading,
    Playing,
    Score,
    Exit,
}

#[derive(Component)]
struct RectClash;

fn on_start_playing_rect_clash(mut state: ResMut<NextState<RectClashState>>) {
    state.set(RectClashState::Loading);
}

#[derive(Component)]
struct RectClashLoading;

fn setup_loading() {}

fn check_loading(mut state: ResMut<NextState<RectClashState>>) {
    state.set(RectClashState::Playing);
}

#[derive(Component)]
struct RectClashPlaying;

#[derive(Component)]
struct BackgroundUiCamera;

#[derive(Component)]
struct UiCamera;

#[derive(Component)]
struct GameCamera;

fn setup_camera_and_ui(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        BackgroundUiCamera,
        RectClash,
    ));

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 3,
                ..default()
            },
            ..default()
        },
        RectClash,
        UiCamera,
    ));

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        RtsCamera::default(),
        RtsCameraControls::default(),
        MouseCamera,
        GameCamera,
        RectClash,
    ));
}

const GROUND_LEVEL: f32 = 0.;

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 1000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_euler(
                EulerRot::YXZ,
                150.0f32.to_radians(),
                -40.0f32.to_radians(),
                0.0,
            )),
            ..default()
        },
        RectClash,
        RectClashPlaying,
    ));

    let terrain_material = materials.add(Color::srgb_u8(0, 255, 0));

    const GROUND_SIZE: Vec3 = Vec3::new(20., 0.5, 20.);

    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(Cuboid::new(GROUND_SIZE.x, GROUND_SIZE.y, GROUND_SIZE.z)),
            material: terrain_material.clone(),
            transform: Transform::from_translation(Vec3::new(0., GROUND_LEVEL, 0.)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(GROUND_SIZE.x, GROUND_SIZE.y, GROUND_SIZE.z),
        Ground,
        RectClash,
    ));
}

fn spawn_starting_battlers(mut spawn: EventWriter<SpawnBattler>) {
    spawn.send(SpawnBattler {
        team: Team::Red,
        location: Vec2::new(0., -5.),
        squad_size: 1,
        template: &LINE_MAN_TEMPLATE,
    });

    spawn.send(SpawnBattler {
        team: Team::Blue,
        location: Vec2::new(0., 5.),
        squad_size: 1,
        template: &LINE_MAN_TEMPLATE,
    });
}

const DEFAULT_LAYER: u32 = 1 << 0;
const GROUND_LAYER: u32 = 1 << 1;
// Reserve last four layers for battler teams
const BATTLER_LAYER: u32 = 1 << 2;

const fn battler_team_layer(team: Team) -> u32 {
    match team {
        Team::Red => 1 << 31,
        Team::Blue => 1 << 30,
    }
}

fn all_other_team_layers(target_team: Team) -> u32 {
    let mut layers = 0;
    for team in Team::iter() {
        if team != target_team {
            layers |= battler_team_layer(team);
        }
    }

    layers
}

#[derive(Component)]
struct Battler;

#[derive(Component)]
struct BrainIndependent;

#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, EnumIter)]
enum Team {
    Red,
    Blue,
}

#[derive(Component)]
struct BattleBubble {
    active_query: Option<Entity>,
    query_timer: Timer,
}

impl BattleBubble {
    fn new<T: DelegatedRng>(rng: &mut T) -> Self {
        let mut timer = Timer::from_seconds(0.1, TimerMode::Repeating);
        timer.tick(Duration::from_nanos(
            rng.u64(0..timer.duration().as_nanos() as u64),
        ));
        Self {
            active_query: None,
            query_timer: timer,
        }
    }
}

#[derive(Event)]
struct SpawnBattler {
    team: Team,
    location: Vec2,
    template: &'static BaseBattlerTemplate,
    squad_size: usize,
}

fn spawn_battlers(
    mut commands: Commands,
    mut spawn_battler: EventReader<SpawnBattler>,
    global_rng: ResMut<GlobalRng>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let global_rng = global_rng.into_inner();

    for event in spawn_battler.read() {
        // Treat as center spawn in snake formation
        let cols = 10.;
        let rows = (event.squad_size as f32 / cols).ceil();

        // Move location to the center of the cols
        // Im fairly certain this is wrong
        let location = Vec2::new(
            event.location.x - (cols / 2. * 0.9),
            event.location.y - (rows / 2. * 0.9),
        );

        let locations = (0..event.squad_size)
            .map(|i| {
                let x = (location.x + (i as f32 % cols) * 0.9)
                    + (global_rng.i32(-1..1) as f32 * event.template.col_discipline);
                let y = (location.y + (i as f32 / cols).floor() * 0.9)
                    + (global_rng.i32(-1..1) as f32 * event.template.col_discipline);
                Vec2::new(x, y)
            })
            .collect::<Vec<_>>();

        for location in locations {
            let color = match event.team {
                Team::Red => Color::srgb_u8(255, 0, 0),
                Team::Blue => Color::srgb_u8(0, 0, 255),
            };

            // Add slight vartiation to size
            let size = event.template.size
                + Vec3::new(
                    global_rng.i32(-25..25) as f32 * 0.001,
                    global_rng.i32(-25..25) as f32 * 0.001,
                    0.,
                );

            commands.spawn((
                MaterialMeshBundle {
                    mesh: meshes.add(Cuboid::new(size.x, size.y, size.z)),
                    material: materials.add(color),
                    transform: Transform::from_translation(Vec3::new(
                        location.x,
                        GROUND_LEVEL + 1.0,
                        location.y,
                    )),
                    ..default()
                },
                event.team,
                Battler,
                Speed(1.0),
                RngComponent::new(),
                BrainIndependent,
                RigidBody::Dynamic,
                LinearVelocity(Vec3::ZERO),
                Collider::cuboid(size.x, size.y, size.z),
                CollisionLayers::from_bits(
                    BATTLER_LAYER,
                    DEFAULT_LAYER | GROUND_LAYER | BATTLER_LAYER,
                ),
                BattleBubble::new(global_rng),
                RectClash,
                RectClashPlaying,
            ));
        }
    }
}

fn setup_independent_brain(mut commands: Commands, query: Query<Entity, Added<BrainIndependent>>) {
    for entity in &query {
        commands.entity(entity).insert(FindClosestEnemy::default());
    }
}

#[derive(Component, Default)]
struct FindClosestEnemy {
    ray: Option<Entity>,
    looking_direction_index: usize,
}

fn find_closest_enemy(
    mut commands: Commands,
    casts: Query<(&RayCaster, &RayHits)>,
    battlers: Query<(&GlobalTransform, &Team), With<Battler>>,
    mut closest_enemies: Query<(Entity, &mut FindClosestEnemy)>,
) {
    const DIRECTIONS: [Dir3; 6] = [
        Dir3::X,
        Dir3::Y,
        Dir3::Z,
        Dir3::NEG_X,
        Dir3::NEG_Y,
        Dir3::NEG_Z,
    ];

    for (entity, mut closest) in &mut closest_enemies {
        let (transform, team) = battlers.get(entity).unwrap();

        if let Some(ray_entity) = closest.ray {
            if let Ok((ray, hits)) = casts.get(ray_entity) {
                if !hits.is_empty() {
                    for hit in hits.iter_sorted() {
                        info!(
                            "{} Hit entity {} at {} with normal {} from direction {:?}",
                            entity,
                            hit.entity,
                            ray.origin + *ray.direction * hit.time_of_impact,
                            hit.normal,
                            ray.direction
                        );
                        commands.entity(entity).remove::<FindClosestEnemy>();
                        commands
                            .entity(entity)
                            .insert(MoveTowardsTarget::new(hit.entity));
                    }
                } else {
                    info!("{} No hits from direction {:?}", entity, ray.direction);
                }
                commands.entity(ray_entity).despawn();
                closest.ray = None;
            }
        } else if let Some(direction) = DIRECTIONS.get(closest.looking_direction_index) {
            let query_filter = SpatialQueryFilter::from_mask(all_other_team_layers(*team));
            closest.ray = Some(
                commands
                    .spawn(
                        RayCaster::new(transform.translation(), *direction)
                            .with_query_filter(query_filter),
                    )
                    .id(),
            );
            closest.looking_direction_index += 1;
        }
    }
}

#[derive(Component)]
struct MoveTowardsTarget {
    entity: Entity,
    direction_update: Timer,
}

impl MoveTowardsTarget {
    fn new(target: Entity) -> Self {
        let mut timer = Timer::from_seconds(0.1, TimerMode::Repeating);
        timer.tick(timer.duration());

        Self {
            entity: target,
            direction_update: timer,
        }
    }
}

fn move_towards_target(
    mut commands: Commands,
    time: Res<Time>,
    targets: Query<&GlobalTransform>,
    mut move_towards_targets: Query<(
        Entity,
        &GlobalTransform,
        &Speed,
        &mut LinearVelocity,
        &mut MoveTowardsTarget,
    )>,
) {
    for (entity, transform, speed, mut vel, mut target) in &mut move_towards_targets {
        if let Ok(target_transform) = targets.get(target.entity) {
            let direction = target_transform.translation() - transform.translation();
            let distance = direction.length();
            if distance < 0.1 {
                info!("{} reached target {}", entity, target.entity);
                commands.entity(entity).remove::<MoveTowardsTarget>();
            } else if target.direction_update.tick(time.delta()).just_finished() {
                let direction = direction / distance;

                let speed = speed.0;

                let movement = direction * speed;

                vel.0 = Vec3::new(movement.x, 0., movement.z);
            }
        }
    }
}

#[derive(Component)]
struct BattleFightingCast;

fn battlers_collision(
    mut commands: Commands,
    time: Res<Time>,
    casts: Query<(&ShapeCaster, &ShapeHits), With<BattleFightingCast>>,
    mut battler: Query<(Entity, &GlobalTransform, &mut BattleBubble, &Team), With<Battler>>,
) {
    for (entity, transform, mut bubble, team) in &mut battler {
        if bubble.query_timer.tick(time.delta()).just_finished() {
            let query = commands
                .spawn((
                    ShapeCaster::new(
                        Collider::sphere(0.5),
                        transform.translation(),
                        Quat::default(),
                        Dir3::X,
                    )
                    .with_query_filter(SpatialQueryFilter::from_mask(all_other_team_layers(*team))),
                    BattleFightingCast,
                    RectClash,
                ))
                .id();
            bubble.active_query = Some(query);
        }

        if let Some(query) = bubble.active_query {
            if let Ok((_, hits)) = casts.get(query) {
                info!("{} Hit {} entities", entity, hits.len());
                commands.entity(query).despawn();
            }
        }
    }
}

fn debug_spawn_battlers(
    mut spawn: EventWriter<SpawnBattler>,
    mut global_rng: ResMut<GlobalRng>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        spawn.send(SpawnBattler {
            team: if global_rng.bool() {
                Team::Red
            } else {
                Team::Blue
            },
            location: Vec2::new(
                global_rng.i32(-5..5) as f32 * 0.7,
                global_rng.i32(-5..5) as f32 * 0.7,
            ),
            squad_size: 80,
            template: &LINE_MAN_TEMPLATE,
        });
    }
}

#[derive(Component)]
struct RectClashScore;

fn setup_score_screen() {}

#[derive(Component)]
struct QuitButton;

fn quit_button_pressed(
    mut state: ResMut<NextState<RectClashState>>,
    quit_buttons: Query<&Interaction, (With<QuitButton>, Changed<Interaction>)>,
) {
    for interaction in &quit_buttons {
        if interaction == &Interaction::Pressed {
            state.set(RectClashState::Exit);
        }
    }
}

fn on_exit(
    mut state: ResMut<NextState<MiniGameState>>,
    mut event_writer: EventWriter<MiniGameCompleted>,
) {
    state.set(MiniGameState::None);

    let score = 0.;

    event_writer.send(MiniGameCompleted {
        game_type: MiniGameType::Translate,
        result: if score > 10000. {
            MiniGameResult::Lose
        } else if score > 5000. {
            MiniGameResult::Draw
        } else {
            MiniGameResult::Win
        },
    });
}

fn despawn_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<RectClashPlaying>>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y < GROUND_LEVEL - 100. {
            commands.entity(entity).despawn();
        }
    }
}
