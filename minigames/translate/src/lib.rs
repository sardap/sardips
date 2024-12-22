#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
use std::{collections::HashSet, time::Duration};

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::hashbrown::HashMap,
};
use sardips::{
    age::UpdateAge,
    assets::{FontAssets, TranslateAssets},
    despawn_all,
    interaction::{MouseCamera, WorldMouse},
    minigames::{
        translate_wordbank::{Word, WordBank, WordSet},
        MiniGameState,
    },
    random_choose,
    shrink::Shrinking,
    text_database::{get_writing_system, Language, WritingSystems},
    text_translation::KeyText,
};
use shared_deps::{
    avian2d::prelude::{
        Collider, CollisionLayers, GravityScale, LinearVelocity, Mass, PhysicsLayer, RigidBody,
        SpatialQuery, SpatialQueryFilter,
    },
    bevy_turborand::{DelegatedRng, GlobalRng},
};
use text_keys::MINIGAME_TRANSLATE_SCORE;

pub struct TranslateGamePlugin;

impl Plugin for TranslateGamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(TranslateState::default())
            .add_event::<WordToKill>()
            .add_systems(
                OnEnter(MiniGameState::PlayingTranslate),
                (setup_game, setup_camera_and_ui),
            )
            .add_systems(
                OnExit(MiniGameState::PlayingTranslate),
                despawn_all::<Translate>,
            )
            .add_systems(OnEnter(TranslateState::Loading), setup_loading)
            .add_systems(
                Update,
                check_loading.run_if(in_state(TranslateState::Loading)),
            )
            .add_systems(
                OnEnter(TranslateState::Playing),
                (setup_spawners, select_words, setup_walls, setup_history),
            )
            .add_systems(
                OnExit(TranslateState::Playing),
                despawn_all::<TranslatePlaying>,
            )
            .add_systems(
                Update,
                (
                    spawn_words,
                    mark_fallen_words_to_kill,
                    kill_words,
                    handle_click,
                    add_select_border,
                    remove_select_border,
                    unstick_stuck,
                    update_score_text,
                )
                    .run_if(in_state(TranslateState::Playing)),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum TranslateState {
    #[default]
    None,
    Loading,
    Playing,
    #[allow(dead_code)]
    Score,
    #[allow(dead_code)]
    Exit,
}

#[derive(Component)]
struct Translate;

#[derive(Component)]
struct SelectedLanguage {
    native_language: Language,
    selected_language: Language,
}

#[derive(Component)]
struct ActiveWordBank {
    sets: Vec<WordSet>,
}

#[derive(Component)]
struct GameContext;

fn setup_game(mut commands: Commands, mut state: ResMut<NextState<TranslateState>>) {
    state.set(TranslateState::Loading);

    let native = Language::English;
    let selected = Language::Korean;

    commands.spawn((
        SelectedLanguage {
            native_language: native,
            selected_language: selected,
        },
        GameContext,
        Translate,
    ));
}

fn setup_loading(asset_server: Res<AssetServer>, mut bank: ResMut<WordBank>) {
    bank.start_loading(&asset_server);
}

fn check_loading(
    mut state: ResMut<NextState<TranslateState>>,
    bank: Res<WordBank>,
    asset_server: Res<AssetServer>,
) {
    if bank.loading_complete(&asset_server) {
        state.set(TranslateState::Playing);
    }
}

#[derive(Component)]
struct TranslateUiCamera;

#[derive(Component)]
struct TranslateSpriteCamera;

fn setup_camera_and_ui(
    mut commands: Commands,
    assets: Res<TranslateAssets>,
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
            TranslateUiCamera,
            Translate,
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
            Translate,
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    // width: Val::Percent(100.),
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
                    height: Val::Percent(100.),
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                },

                ..default()
            },
            Translate,
        ))
        .with_children(|parent| {
            parent.spawn((NodeBundle {
                style: Style {
                    width: Val::Px(5.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.65, 0., 0., 0.5)),
                ..default()
            },));
        });

    commands
        .spawn((
            TargetCamera(ui_camera),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                },

                ..default()
            },
            Translate,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    style: Style {
                        top: Val::Px(10.),
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
                KeyText::new().with(0, MINIGAME_TRANSLATE_SCORE),
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
        TranslateSpriteCamera,
        Translate,
    ));
}

#[derive(Component)]
struct TranslatePlaying;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct WordSpawner {
    timer: Timer,
}

fn setup_spawners(mut commands: Commands) {
    commands.spawn((
        WordSpawner {
            timer: Timer::new(Duration::from_secs_f32(5.), TimerMode::Repeating),
        },
        TranslatePlaying,
        Translate,
    ));
}

const WALL_FILTERS: [ColLayer; 2] = [ColLayer::Wall, ColLayer::Word];
const WALL_HEIGHT: f32 = 70000.;

fn setup_walls(mut commands: Commands) {
    // Middle
    commands.spawn((
        TransformBundle {
            local: Transform::from_translation(Vec3::new(0., 0., ZLayer::Background.to_f32())),
            ..default()
        },
        Collider::rectangle(40., WALL_HEIGHT),
        CollisionLayers::new([ColLayer::Wall], WALL_FILTERS),
        RigidBody::Static,
        TranslatePlaying,
        Translate,
    ));

    const WALL_WIDTH: f32 = 300.;

    // Left
    commands.spawn((
        TransformBundle {
            local: Transform::from_translation(Vec3::new(
                BOUNDS.min.x - WALL_WIDTH / 2.,
                0.,
                ZLayer::Background.to_f32(),
            )),
            ..default()
        },
        Collider::rectangle(WALL_WIDTH, WALL_HEIGHT),
        CollisionLayers::new([ColLayer::Wall], WALL_FILTERS),
        RigidBody::Static,
        TranslatePlaying,
        Translate,
    ));

    // Right
    commands.spawn((
        TransformBundle {
            local: Transform::from_translation(Vec3::new(
                BOUNDS.max.x + WALL_WIDTH / 2.,
                0.,
                ZLayer::Background.to_f32(),
            )),
            ..default()
        },
        Collider::rectangle(WALL_WIDTH, WALL_HEIGHT),
        CollisionLayers::new([ColLayer::Wall], WALL_FILTERS),
        RigidBody::Static,
        TranslatePlaying,
        Translate,
    ));
}

fn select_words(
    mut commands: Commands,
    word_bank: Res<WordBank>,
    language: Query<&SelectedLanguage>,
) {
    let language = language.single();

    let sets = word_bank.set_for_languages(&[language.native_language, language.selected_language]);
    assert!(!sets.is_empty());
    commands.spawn((ActiveWordBank { sets }, Translate));
}

fn setup_history(mut commands: Commands, existing: Query<Entity, With<History>>) {
    if existing.iter().count() > 0 {
        return;
    }

    commands.spawn((History::default(), Translate));
}

/*

        LEFT (FADE IN)      |   RIGHT (INTO FIRE)
    ------------------------------------------------
    |                       |                       |
    |                       |                       |
    |       ICE             |                       |
    |                       |                       |
    |                       |                       |
    |                       |                       |
    |                       |    얼음                |
    |                       |                       |
    |                       |                       |
    |                       |                       |
    |                       |                       |
    |                       |                       |
    |                       |                       |
    |                       |                       |
    |                       |🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥🔥|
    ------------------------------------------------

*/

#[derive(Copy, Clone)]
enum ZLayer {
    Background = 1,
    Word,
}

impl ZLayer {
    fn to_f32(self) -> f32 {
        self as u8 as f32
    }
}

#[derive(PhysicsLayer, Default)]
enum ColLayer {
    #[default]
    Default,
    Word,
    Wall,
}

const BOUNDS: Rect = Rect {
    min: Vec2::new(-250., -GAME_HEIGHT / 2.),
    max: Vec2::new(250., GAME_HEIGHT / 2.),
};

const WORD_SIZE: Vec2 = Vec2::new(80., 80.);

const GAME_HEIGHT: f32 = 700.;

const LEFT_BOUNDS: Rect = Rect {
    min: Vec2::new(-200., -GAME_HEIGHT / 2.),
    max: Vec2::new(-50., GAME_HEIGHT / 2.),
};

const RIGHT_BOUNDS: Rect = Rect {
    min: Vec2::new(50., -GAME_HEIGHT / 2.),
    max: Vec2::new(200., GAME_HEIGHT / 2.),
};

fn random_point_in_bounds<T: DelegatedRng>(rng: &mut T, bounds: &Rect) -> Vec2 {
    Vec2::new(
        rng.i32(bounds.min.x as i32..bounds.max.x as i32) as f32,
        rng.i32(bounds.min.y as i32..bounds.max.y as i32) as f32,
    )
}

#[derive(Component)]
struct VisualWord {
    set_id: String,
    siblings: Vec<Entity>,
}

#[derive(Component)]
struct FallingWord;

fn spawn_words(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    global_rng: ResMut<GlobalRng>,
    time: Res<Time>,
    fonts: Res<FontAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    active_word_bank: Query<&ActiveWordBank>,
    language: Query<&SelectedLanguage>,
    mut spawner: Query<&mut WordSpawner>,
    existing_words: Query<&VisualWord>,
) {
    let language = language.single();
    let word_bank = active_word_bank.single();

    let rng = global_rng.into_inner();

    for mut spawner in &mut spawner {
        if spawner.timer.tick(time.delta()).just_finished() {
            let active_sets = existing_words
                .iter()
                .map(|w| w.set_id.clone())
                .collect::<HashSet<_>>();
            let mut set = random_choose(rng, &word_bank.sets);
            for _ in 0..10 {
                if !active_sets.contains(&set.id) {
                    break;
                }
                set = random_choose(rng, &word_bank.sets);
            }

            let (left_word, right_word) = if rng.bool() {
                (
                    set.words.get(&language.native_language).unwrap(),
                    set.words.get(&language.selected_language).unwrap(),
                )
            } else {
                (
                    set.words.get(&language.selected_language).unwrap(),
                    set.words.get(&language.native_language).unwrap(),
                )
            };

            #[derive(PartialEq, Eq)]
            enum Sides {
                None,
                Left,
                Right,
            }

            let picture_side = {
                let chance = rng.f32();
                if chance < 0.25 {
                    Sides::Left
                } else if chance < 0.5 {
                    Sides::Right
                } else {
                    Sides::None
                }
            };

            let get_font_size = |word: &Word| {
                let font_size = match get_writing_system(&word.word) {
                    WritingSystems::Korean => KOREAN_FONT_SIZE_LOOKUP_TABLE.iter(),
                    WritingSystems::Other => ENGLISH_FONT_SIZE_LOOKUP_TABLE.iter(),
                }
                .find(|threshold| word.word.len() <= threshold.length)
                .unwrap_or(&DEFAULT_FONT_SIZE);

                font_size
            };

            let left_size = if picture_side == Sides::Left {
                get_font_size(left_word)
            } else {
                &DEFAULT_FONT_SIZE
            };

            let left_word_collider = Collider::rectangle(left_size.size.x, left_size.size.y);

            let (left_pos, right_pos) = {
                (
                    {
                        let mut pos: Vec2 = Vec2::ZERO;
                        for _ in 0..100 {
                            pos = random_point_in_bounds(rng, &LEFT_BOUNDS);
                            let query = spatial_query.cast_shape(
                                &left_word_collider,
                                pos,
                                0.,
                                Dir2::X,
                                0.,
                                false,
                                SpatialQueryFilter::from_mask(ColLayer::Word),
                            );
                            if query.is_none() {
                                break;
                            }
                        }
                        pos.extend(ZLayer::Word.to_f32())
                    },
                    Vec3::new(
                        rng.i32(RIGHT_BOUNDS.min.x as i32..RIGHT_BOUNDS.max.x as i32) as f32,
                        rng.i32(400..500) as f32,
                        ZLayer::Word.to_f32(),
                    ),
                )
            };

            let mut spawn_picture = |commands: &mut Commands, pos: Vec3| {
                commands
                    .spawn((
                        SpriteBundle {
                            transform: Transform::from_translation(pos),
                            sprite: Sprite {
                                custom_size: Some(DEFAULT_FONT_SIZE.size),
                                ..default()
                            },
                            texture: random_choose(rng, &set.images).clone(),
                            ..default()
                        },
                        Collider::rectangle(DEFAULT_FONT_SIZE.size.x, DEFAULT_FONT_SIZE.size.y),
                    ))
                    .id()
            };

            struct FontSizeThreshold {
                length: usize,
                font_size: f32,
                size: Vec2,
            }
            const DEFAULT_FONT_SIZE: FontSizeThreshold = FontSizeThreshold {
                length: 0,
                font_size: 15.,
                size: Vec2::new(80., 80.),
            };
            const ENGLISH_FONT_SIZE_LOOKUP_TABLE: [FontSizeThreshold; 5] = [
                FontSizeThreshold {
                    length: 2,
                    font_size: 50.,
                    size: Vec2::new(80., 80.),
                },
                FontSizeThreshold {
                    length: 3,
                    font_size: 40.,
                    size: Vec2::new(80., 80.),
                },
                FontSizeThreshold {
                    length: 6,
                    font_size: 30.,
                    size: Vec2::new(80., 60.),
                },
                FontSizeThreshold {
                    length: 8,
                    font_size: 20.,
                    size: Vec2::new(80., 50.),
                },
                FontSizeThreshold {
                    length: 10,
                    font_size: 20.,
                    size: Vec2::new(100., 50.),
                },
            ];
            const KOREAN_FONT_SIZE_LOOKUP_TABLE: [FontSizeThreshold; 2] = [
                FontSizeThreshold {
                    length: 2,
                    font_size: 50.,
                    size: Vec2::new(80., 80.),
                },
                FontSizeThreshold {
                    length: 10,
                    font_size: 31.,
                    size: Vec2::new(80., 80.),
                },
            ];
            let mut spawn_word = |commands: &mut Commands, word: &Word, pos: Vec3| {
                let font_size = get_font_size(word);

                commands
                    .spawn((
                        MaterialMesh2dBundle {
                            mesh: Mesh2dHandle(
                                meshes.add(Rectangle::new(font_size.size.x, font_size.size.y)),
                            ),
                            material: materials.add(Color::WHITE),
                            transform: Transform::from_translation(pos),
                            ..default()
                        },
                        Collider::rectangle(font_size.size.x, font_size.size.y),
                    ))
                    .with_children(|parent| {
                        parent.spawn(Text2dBundle {
                            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                            text: Text::from_section(
                                word.word.clone(),
                                TextStyle {
                                    font: fonts.main_font.clone(),
                                    font_size: font_size.font_size,
                                    color: Color::BLACK,
                                },
                            ),
                            ..default()
                        });
                    })
                    .id()
            };

            let (left_entity, right_entity) = match picture_side {
                Sides::Left => (
                    spawn_picture(&mut commands, left_pos),
                    spawn_word(&mut commands, left_word, right_pos),
                ),
                Sides::Right => (
                    spawn_word(&mut commands, right_word, left_pos),
                    spawn_picture(&mut commands, right_pos),
                ),
                Sides::None => (
                    spawn_word(&mut commands, left_word, left_pos),
                    spawn_word(&mut commands, right_word, right_pos),
                ),
            };

            let elements = vec![left_entity, right_entity];

            commands.entity(left_entity).insert(RigidBody::Static);

            commands.entity(right_entity).insert((
                RigidBody::Dynamic,
                GravityScale((rng.i32(80..300) / 100) as f32 + rng.f32()),
                Mass(5.0),
                FallingWord,
                ShouldUnstick::default(),
                LinearVelocity(Vec2::new(0., 0.)),
            ));

            for entity in &[left_entity, right_entity] {
                commands.entity(*entity).insert((
                    TranslatePlaying,
                    Translate,
                    UpdateAge::default(),
                    CollisionLayers::new([ColLayer::Word], [ColLayer::Word, ColLayer::Wall]),
                    VisualWord {
                        siblings: elements.clone(),
                        set_id: set.id.clone(),
                    },
                ));
            }
        }
    }
}

fn mark_fallen_words_to_kill(
    mut kill_words: EventWriter<WordToKill>,
    query: Query<
        (Entity, &Transform, &VisualWord, &UpdateAge),
        (With<FallingWord>, Without<Shrinking>),
    >,
    mut history: Query<&mut History>,
) {
    for (entity, transform, word, age) in &query {
        if transform.translation.y < -GAME_HEIGHT / 2. - 50. {
            let mut history = history.single_mut();

            kill_words.send(WordToKill {
                entity,
                correct: false,
            });
            history.history.push(HistoryEntry {
                word_set_id: word.set_id.clone(),
                correct: WordHistoryResult::Missed,
                time_alive: age.0,
            });
        }
    }
}

#[derive(Event)]
struct WordToKill {
    entity: Entity,
    correct: bool,
}

fn kill_words(
    mut commands: Commands,
    mut kill_words: EventReader<WordToKill>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<&VisualWord>,
    colliders: Query<&Collider, With<VisualWord>>,
    dying: Query<Entity, (With<Translate>, With<Shrinking>)>,
) {
    let mut complete_to_kill = HashMap::new();

    for to_kill in kill_words.read() {
        if let Ok(visual_word) = query.get(to_kill.entity) {
            for sibling in &visual_word.siblings {
                if dying.get(*sibling).is_err() {
                    complete_to_kill.insert(*sibling, to_kill.correct);
                }
            }
        }
    }

    for (to_kill, correct) in complete_to_kill {
        commands.entity(to_kill).remove::<Collider>();
        commands.entity(to_kill).remove::<Selected>();
        commands.entity(to_kill).insert((
            Shrinking::new(Vec2::new(1., 1.), Duration::from_secs_f32(1.)),
            RigidBody::Static,
        ));

        let size = colliders
            .get(to_kill)
            .unwrap()
            .shape()
            .as_cuboid()
            .unwrap()
            .half_extents
            * 2.;
        commands.entity(to_kill).with_children(|parent| {
            parent.spawn(MaterialMesh2dBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., 5.)),
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(size.x, size.y))),
                material: materials.add(if correct {
                    Color::srgba(0., 1., 0., 0.3)
                } else {
                    Color::srgba(1., 0., 0., 0.3)
                }),
                ..default()
            });
        });
    }
}

#[derive(Component)]
struct Selected;

fn handle_click(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    mut kill_words: EventWriter<WordToKill>,
    buttons: Res<ButtonInput<MouseButton>>,
    world_mouse: Query<&WorldMouse>,
    words: Query<(&Transform, &VisualWord, &UpdateAge)>,
    selected: Query<Entity, With<Selected>>,
    mut history: Query<&mut History>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let world_mouse = world_mouse.single();
        let mut history = history.single_mut();

        if let Some(projection) = spatial_query.project_point(
            world_mouse.last_position,
            true,
            SpatialQueryFilter::from_mask(ColLayer::Word),
        ) {
            if selected.get(projection.entity).is_ok() {
                commands.entity(projection.entity).remove::<Selected>();
            } else if let Ok((trans, word, age)) = words.get(projection.entity) {
                let dist = trans.translation.xy() - world_mouse.last_position;
                if dist.length() > WORD_SIZE.x + 10. {
                    return;
                }

                if selected.iter().count() < 1 {
                    commands.entity(projection.entity).insert(Selected);
                } else {
                    let other_siblings: Vec<_> = word
                        .siblings
                        .iter()
                        .filter(|e| **e != projection.entity)
                        .collect();

                    // selected pair of words despawn
                    let correct = if other_siblings.iter().all(|e| selected.get(**e).is_ok()) {
                        for sibling in other_siblings {
                            kill_words.send(WordToKill {
                                entity: *sibling,
                                correct: true,
                            });
                        }
                        WordHistoryResult::Correct
                    } else {
                        // deselect existing
                        for entity in selected.iter() {
                            commands.entity(entity).remove::<Selected>();
                        }
                        WordHistoryResult::Incorrect
                    };

                    history.history.push(HistoryEntry {
                        word_set_id: word.set_id.clone(),
                        correct,
                        time_alive: age.0,
                    });
                }
            }
        }
    }
}

#[derive(Component)]
struct SelectedBox;

fn add_select_border(
    mut commands: Commands,
    query: Query<(Entity, &Collider), (Added<Selected>, With<VisualWord>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, col) in &mut query.iter() {
        let size = col.shape().as_cuboid().unwrap().half_extents * 2.;

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(size.x + 10., size.y + 10.))),
                    material: materials.add(Color::srgb(1., 0., 0.)),
                    transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
                    ..default()
                },
                SelectedBox,
            ));
        });
    }
}

fn remove_select_border(
    mut commands: Commands,
    mut removed_select: RemovedComponents<Selected>,
    select_boxes: Query<(Entity, &Parent), With<SelectedBox>>,
) {
    for removed in removed_select.read() {
        for (entity, parent) in &select_boxes {
            if parent.get() == removed {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[derive(Component)]
struct ShouldUnstick {
    pub last_y: Option<Vec2>,
    pub check_timer: Timer,
}

impl Default for ShouldUnstick {
    fn default() -> Self {
        Self {
            last_y: None,
            check_timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
        }
    }
}

fn unstick_stuck(
    time: Res<Time>,
    mut query: Query<
        (
            &mut ShouldUnstick,
            &Transform,
            &mut RigidBody,
            &mut LinearVelocity,
        ),
        (With<FallingWord>, Without<Shrinking>),
    >,
) {
    for (mut unstick, transform, mut body, mut velocity) in &mut query {
        if unstick.check_timer.tick(time.delta()).just_finished() {
            if let Some(last) = unstick.last_y {
                if (transform.translation.y - last.y).abs() < 0.1 {
                    *body = RigidBody::Kinematic;
                    *velocity = LinearVelocity(Vec2::new(0., -50.));
                } else {
                    *body = RigidBody::Dynamic;
                }
            }
            unstick.last_y = Some(transform.translation.xy());
        }

        unstick.last_y = Some(transform.translation.xy());
    }
}

enum WordHistoryResult {
    Correct,
    Incorrect,
    Missed,
}

struct HistoryEntry {
    #[allow(dead_code)]
    pub word_set_id: String,
    pub correct: WordHistoryResult,
    pub time_alive: Duration,
}

#[derive(Component, Default)]
struct History {
    pub history: Vec<HistoryEntry>,
}

impl History {
    pub fn score(&self) -> f32 {
        let mut score = 0.;

        for entry in &self.history {
            match entry.correct {
                WordHistoryResult::Correct => {
                    score += 100. / entry.time_alive.as_secs_f32();
                }
                WordHistoryResult::Incorrect => {
                    score -= 30.;
                }
                WordHistoryResult::Missed => {
                    score -= 100.;
                }
            }
        }

        score
    }
}

fn update_score_text(
    history: Query<&History, Changed<History>>,
    mut score_text: Query<&mut Text, With<ScoreText>>,
) {
    if let Ok(history) = history.get_single() {
        let mut score_text = score_text.single_mut();
        score_text.sections[2].value = format!("{:.2}", history.score());
    }
}
