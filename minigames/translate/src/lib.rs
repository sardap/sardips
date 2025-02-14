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
use sardips_core::{
    assets::{FontAssets, TranslateAssets},
    despawn_all,
    interaction::{MouseCamera, WorldMouse},
    minigames_core::{
        translate_wordbank::{Word, WordBank, WordSet},
        MiniGameBackButton, MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType,
        Playing,
    },
    mood_core::{AutoSetMoodImage, MoodCategory, MoodImageIndexes},
    random_choose,
    shrink::Shrinking,
    text_database::{get_writing_system, Language, WritingSystems},
    text_translation::KeyText,
    VaryingTimer,
};
use shared_deps::{
    avian2d::prelude::{
        Collider, CollisionLayers, GravityScale, LinearVelocity, Mass, PhysicsLayer, RigidBody,
        SpatialQuery, SpatialQueryFilter,
    },
    bevy_turborand::{DelegatedRng, GlobalRng},
};
use text_keys::{MINIGAME_TRANSLATE_SCORE, MINIGAME_TRANSLATE_TIME_LEFT};

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
                (
                    setup_spawners,
                    select_words,
                    setup_walls,
                    setup_history,
                    setup_game_timer,
                ),
            )
            .add_systems(
                OnExit(TranslateState::Playing),
                despawn_all::<TranslatePlaying>,
            )
            .add_systems(
                Update,
                (
                    spawn_words,
                    add_border_to_word,
                    mark_fallen_words_to_kill,
                    kill_words,
                    handle_click,
                    change_select_border_color,
                    remove_select_border,
                    unstick_stuck,
                    update_score_text,
                    tick_game_timer,
                    tick_update_ages,
                    update_pet_mood,
                )
                    .run_if(in_state(TranslateState::Playing)),
            )
            .add_systems(OnEnter(TranslateState::Score), setup_score_screen)
            .add_systems(
                OnExit(TranslateState::Score),
                despawn_all::<TranslateScoreScreen>,
            )
            .add_systems(
                OnEnter(TranslateState::Exit),
                (despawn_all::<Translate>, on_exit),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum TranslateState {
    #[default]
    None,
    Loading,
    Playing,
    Score,
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
struct TranslateBackgroundUiCamera;

#[derive(Component)]
struct TranslateSpriteCamera;

#[derive(Component)]
struct UiPetImage;

fn setup_camera_and_ui(
    mut commands: Commands,
    assets: Res<TranslateAssets>,
    font_assets: Res<FontAssets>,
    pet_sheet: Query<(&Handle<Image>, &TextureAtlas, &MoodImageIndexes), With<Playing>>,
) {
    let ui_background_camera: Entity = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 0,
                    ..default()
                },
                ..default()
            },
            TranslateBackgroundUiCamera,
            Translate,
        ))
        .id();

    let ui_camera: Entity = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 3,
                    ..default()
                },
                ..default()
            },
            Translate,
        ))
        .id();

    commands
        .spawn((
            TargetCamera(ui_background_camera),
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
            TargetCamera(ui_background_camera),
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
                    text: Text::from_sections(vec![TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.main_font.clone(),
                            font_size: 20.,
                            color: Color::BLACK,
                        },
                    )]),
                    ..default()
                },
                KeyText::new().with(0, MINIGAME_TRANSLATE_TIME_LEFT),
                TranslatePlaying,
            ));

            parent.spawn((
                TextBundle {
                    style: Style {
                        top: Val::Px(10.),
                        ..default()
                    },
                    text: Text::from_sections(vec![TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.main_font.clone(),
                            font_size: 20.,
                            color: Color::BLACK,
                        },
                    )]),
                    ..default()
                },
                TimeRemainingText,
                TranslatePlaying,
            ));

            parent.spawn((
                TextBundle {
                    style: Style {
                        margin: UiRect::top(Val::Px(10.)),
                        ..default()
                    },
                    text: Text::from_sections(vec![TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.main_font.clone(),
                            font_size: 20.,
                            color: Color::BLACK,
                        },
                    )]),
                    ..default()
                },
                KeyText::new().with(0, MINIGAME_TRANSLATE_SCORE),
                TranslatePlaying,
            ));

            parent.spawn((
                TextBundle {
                    style: Style { ..default() },
                    text: Text::from_sections(vec![TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.main_font.clone(),
                            font_size: 20.,
                            color: Color::BLACK,
                        },
                    )]),
                    ..default()
                },
                ScoreText,
                TranslatePlaying,
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

    let (image, atlas, mood_images) = pet_sheet.single();
    commands
        .spawn((
            TargetCamera(ui_camera),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            Translate,
            TranslatePlaying,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Px(50.),
                        margin: UiRect::bottom(Val::Px(50.)),
                        ..default()
                    },
                    image: UiImage::new(image.clone()),
                    ..default()
                },
                atlas.clone(),
                UiPetImage,
                *mood_images,
                MoodCategory::Neutral,
                AutoSetMoodImage,
            ));
        });
}

#[derive(Component)]
struct TranslatePlaying;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct TimeRemainingText;

#[derive(Component)]
struct WordSpawner {
    timer: VaryingTimer,
    pending: usize,
}

fn setup_spawners(mut commands: Commands, rng: ResMut<GlobalRng>) {
    let rng = rng.into_inner();

    commands.spawn((
        WordSpawner {
            timer: VaryingTimer::new(
                Duration::from_secs_f32(3.)..Duration::from_secs_f32(5.),
                rng,
            ),
            pending: 1,
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

#[derive(Component)]
struct GameTimer {
    timer: Timer,
}

impl Default for GameTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(60.), TimerMode::Once),
        }
    }
}

fn setup_game_timer(mut commands: Commands) {
    commands.spawn((GameTimer::default(), Translate, TranslatePlaying));
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

const GAME_HEIGHT: f32 = 600.;

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
        if spawner.timer.tick(time.delta(), rng).just_finished() {
            let selected = rng.f32();
            let count = if selected < 0.1 {
                4
            } else if selected < 0.9 {
                2
            } else {
                1
            };

            spawner.pending += count;
        }

        if spawner.pending > 0 {
            spawner.pending -= 1;

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

            let (left_word, right_word) = if picture_side == Sides::None {
                if rng.bool() {
                    (
                        set.words.get(&language.native_language).unwrap(),
                        set.words.get(&language.selected_language).unwrap(),
                    )
                } else {
                    (
                        set.words.get(&language.selected_language).unwrap(),
                        set.words.get(&language.native_language).unwrap(),
                    )
                }
            } else {
                let word = set.words.get(&language.selected_language).unwrap();
                (word, word)
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
                            // pos = Vec2::new(0., 0.);
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
                GravityScale(((rng.i32(80..300) / 100) as f32 + rng.f32()) / 2.),
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

fn add_border_to_word(
    mut commands: Commands,
    query: Query<(Entity, &Collider), (Added<VisualWord>, With<Collider>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, col) in &mut query.iter() {
        let size = col.shape().as_cuboid().unwrap().half_extents * 2.;

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(size.x + 10., size.y + 10.))),
                    material: materials.add(UNSELECTED_BORDER_COLOR),
                    transform: Transform::from_translation(Vec3::new(0., 0., -1.)),
                    ..default()
                },
                SelectedBox,
            ));
        });
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
                result: WordHistoryResult::Missed,
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
                        result: correct,
                        time_alive: age.0,
                    });
                }
            }
        }
    }
}

const UNSELECTED_BORDER_COLOR: Color = Color::srgba(0., 0., 0., 1.);
const SELECTED_BORDER_COLOR: Color = Color::srgba(0.3, 1., 0.3, 1.5);

#[derive(Component)]
struct SelectedBox;

fn change_select_border_color(
    query: Query<&Children, (Added<Selected>, With<VisualWord>)>,
    mut boxes: Query<&mut Handle<ColorMaterial>, With<SelectedBox>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for children in &query {
        for child in children.iter() {
            if let Ok(mut material) = boxes.get_mut(*child) {
                *material = materials.add(SELECTED_BORDER_COLOR);
            }
        }
    }
}

fn remove_select_border(
    mut removed_select: RemovedComponents<Selected>,
    mut select_boxes: Query<(&Parent, &mut Handle<ColorMaterial>), With<SelectedBox>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for removed in removed_select.read() {
        for (parent, mut material) in &mut select_boxes {
            if parent.get() == removed {
                *material = materials.add(UNSELECTED_BORDER_COLOR);
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
    pub result: WordHistoryResult,
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
            match entry.result {
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
        score_text.sections[0].value = format!("{:.2}", history.score());
    }
}

fn tick_game_timer(
    time: Res<Time>,
    mut state: ResMut<NextState<TranslateState>>,
    mut game_timer: Query<&mut GameTimer>,
    mut timer_text: Query<&mut Text, With<TimeRemainingText>>,
) {
    for mut timer in &mut game_timer {
        timer.timer.tick(time.delta());

        let mut timer_text = timer_text.single_mut();
        let remaining = Duration::from_secs_f32(
            timer.timer.duration().as_secs_f32() - timer.timer.elapsed().as_secs_f32(),
        );
        timer_text.sections[0].value =
            format!("{}:{}", remaining.as_secs(), remaining.as_millis() % 1000);

        if timer.timer.finished() {
            state.set(TranslateState::Score);
        }
    }
}

#[derive(Component)]
struct TranslateScoreScreen;

#[derive(Component)]
struct ScoreScreenText;

fn setup_score_screen(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    camera: Query<Entity, With<TranslateSpriteCamera>>,
    history: Query<&History>,
    pet_sheet: Query<(&Handle<Image>, &TextureAtlas, &MoodImageIndexes), With<Playing>>,
) {
    let ui_camera = camera.single();

    let history = history.single();

    const FONT_SIZE: f32 = 30.;

    let (image, atlas, mood_images) = pet_sheet.single();

    commands
        .spawn((
            TargetCamera(ui_camera),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
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
                ImageBundle {
                    style: Style {
                        width: Val::Px(80.),
                        margin: UiRect::bottom(Val::Px(50.)),
                        ..default()
                    },
                    image: UiImage::new(image.clone()),
                    ..default()
                },
                atlas.clone(),
                UiPetImage,
                *mood_images,
                MoodCategory::Neutral,
                AutoSetMoodImage,
            ));

            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(300.),
                            height: Val::Px(50.),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(1., 1., 1., 0.5)),
                        border_color: BorderColor(Color::BLACK),
                        border_radius: BorderRadius::all(Val::Px(10.)),
                        ..default()
                    },
                    Translate,
                    TranslateScoreScreen,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            style: Style { ..default() },
                            text: Text::from_sections(vec![TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.main_font.clone(),
                                    font_size: FONT_SIZE,
                                    color: Color::BLACK,
                                },
                            )]),
                            ..default()
                        },
                        KeyText::new().with(0, MINIGAME_TRANSLATE_SCORE),
                    ));

                    parent.spawn((
                        TextBundle {
                            style: Style { ..default() },
                            text: Text::from_sections(vec![TextSection::new(
                                history.score().to_string(),
                                TextStyle {
                                    font: font_assets.main_font.clone(),
                                    font_size: FONT_SIZE,
                                    color: Color::BLACK,
                                },
                            )]),
                            ..default()
                        },
                        ScoreScreenText,
                    ));
                });

            parent.spawn(MiniGameBackButton);
        });
}

fn on_exit(
    mut state: ResMut<NextState<MiniGameState>>,
    mut event_writer: EventWriter<MiniGameCompleted>,
    history: Query<&History>,
) {
    state.set(MiniGameState::None);

    let score = history.single().score();

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

#[derive(Component, Default)]
pub struct UpdateAge(pub Duration);

fn tick_update_ages(time: Res<Time>, mut query: Query<&mut UpdateAge>) {
    for mut update_age in query.iter_mut() {
        update_age.0 += time.delta();
    }
}

fn update_pet_mood(
    history: Query<&History, Changed<History>>,
    mut pet: Query<&mut MoodCategory, With<UiPetImage>>,
) {
    if let Ok(score) = history.get_single() {
        let recent_value: f32 = score
            .history
            .iter()
            .rev()
            .take(5)
            .map(|e| match e.result {
                WordHistoryResult::Correct => 1.,
                WordHistoryResult::Incorrect => 0.,
                WordHistoryResult::Missed => -1.,
            })
            .sum();

        let updated_mood = if recent_value > 5. {
            MoodCategory::Ecstatic
        } else if recent_value > 2. {
            MoodCategory::Happy
        } else if recent_value > 0. {
            MoodCategory::Neutral
        } else if recent_value > -2. {
            MoodCategory::Sad
        } else {
            MoodCategory::Despairing
        };

        for mut mood in &mut pet {
            *mood = updated_mood
        }
    }
}
