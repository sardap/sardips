use bevy::{prelude::*, render::view::RenderLayers};
use bevy_parallax::{CreateParallaxEvent, LayerData, LayerSpeed, ParallaxCameraComponent};

use crate::{
    assets,
    autoscroll::AutoScroll,
    pet::mood::{AutoSetMoodImage, MoodCategory, MoodImages},
};

use super::{MiniGameState, Playing};

pub struct SprintPlugin;

impl Plugin for SprintPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(SprintState::default())
            .add_systems(OnEnter(MiniGameState::PlayingSprint), setup)
            .add_systems(OnExit(MiniGameState::PlayingSprint), teardown)
            // .add_systems(
            //     FixedUpdate,
            //     (apply_velocity).run_if(in_state(SprintState::Playing)),
            // )
            .add_systems(
                Update,
                (handle_input, despawn_obstacles, apply_velocity)
                    .run_if(in_state(SprintState::Playing)),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum SprintState {
    #[default]
    None,
    Playing,
}

#[derive(Component)]
struct Sprint;

#[derive(Component)]
struct SprintBackgroundCamera;

#[derive(Component)]
struct SprintGameCamera;

#[derive(Component)]
struct SprintPlayer;

#[derive(Component)]
struct Obstacle;

const BACKGROUND_LAYER: usize = 1;
const GAME_LAYER: usize = 2;

const HORIZONTAL_SPEED: f32 = 30.;
const GROUND_Y: f32 = -133.0;

const GRAVITY: f32 = 9.8;

fn setup(
    mut commands: Commands,
    mut state: ResMut<NextState<SprintState>>,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
    pet_sheet: Query<(&Handle<Image>, &TextureAtlas, &MoodImages), With<Playing>>,
) {
    state.set(SprintState::Playing);

    let background_camera: Entity = commands
        .spawn((
            Camera2dBundle::default(),
            RenderLayers::from_layers(&[BACKGROUND_LAYER]),
            ParallaxCameraComponent::new(BACKGROUND_LAYER as u8),
            AutoScroll::new(Vec2::new(HORIZONTAL_SPEED, 0.)),
            Sprint,
            SprintBackgroundCamera,
        ))
        .id();

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 2,
                ..default()
            },
            ..default()
        },
        RenderLayers::from_layers(&[GAME_LAYER]),
        Sprint,
        SprintGameCamera,
    ));

    create_parallax.send(CreateParallaxEvent {
        camera: background_camera,
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Horizontal(0.0),
                path: assets::SprintAssets::SKY.to_string(),
                tile_size: UVec2::new(143, 700),
                scale: Vec2::splat(1.0),
                z: 0.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.25),
                path: assets::SprintAssets::HILLS.to_string(),
                tile_size: UVec2::new(2000, 700),
                scale: Vec2::splat(1.0),
                z: 1.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.5),
                path: assets::SprintAssets::GROUND.to_string(),
                tile_size: UVec2::new(127, 700),
                scale: Vec2::splat(1.0),
                z: 2.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Horizontal(0.5),
                path: assets::SprintAssets::GRASS_BLADES.to_string(),
                tile_size: UVec2::new(218, 700),
                scale: Vec2::splat(1.0),
                z: 4.0,
                ..default()
            },
        ],
    });

    commands.spawn((SpriteBundle { ..default() }, Sprint));

    let (image, texture_atlas, mood_images) = pet_sheet.single();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-200., GROUND_Y, 10.0)),
            sprite: Sprite {
                custom_size: Some(Vec2::new(64.0, 64.0)),
                ..default()
            },
            texture: image.clone(),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas.layout.clone(),
            ..default()
        },
        RenderLayers::from_layers(&[GAME_LAYER]),
        *mood_images,
        MoodCategory::Neutral,
        AutoSetMoodImage,
        BoundingRect {
            min: Vec2::new(-32., -32.),
            max: Vec2::new(32., 32.),
        },
        Velocity::default(),
        Sprint,
        SprintPlayer,
    ));

    // Spawn osculates
    for i in 0..100 {
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(i as f32 * 100., GROUND_Y, 10.0)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(32.0, 32.0)),
                    ..default()
                },
                texture: image.clone(),
                ..default()
            },
            RenderLayers::from_layers(&[GAME_LAYER]),
            *mood_images,
            MoodCategory::Neutral,
            AutoSetMoodImage,
            Velocity(Vec2::new(-HORIZONTAL_SPEED, 0.)),
            Sprint,
            Obstacle,
        ));
    }
}

fn teardown(mut commands: Commands, to_delete: Query<Entity, With<Sprint>>) {
    for entity in to_delete.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[allow(dead_code)]
#[derive(Component)]
struct BoundingRect {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<BoundingRect>>,
) {
    for (mut transform, mut velocity) in query.iter_mut() {
        transform.translation.y += velocity.y * time.delta_seconds();
        transform.translation.x += velocity.x * time.delta_seconds();

        if transform.translation.y < GROUND_Y {
            transform.translation.y = GROUND_Y;
            velocity.y = 0.;
        } else {
            velocity.y -= GRAVITY * time.delta_seconds();
        }
    }
}

fn handle_input(
    mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut Velocity, With<SprintPlayer>>,
) {
    let mut vel = query.single_mut();

    if mouse.pressed(MouseButton::Left) && vel.y == 0. {
        vel.y = 20.;
    }
}

fn despawn_obstacles(mut commands: Commands, query: Query<(Entity, &Transform), With<Obstacle>>) {
    for (entity, transform) in query.iter() {
        if transform.translation.x < -500. {
            commands.entity(entity).despawn_recursive();
        }
    }
}
