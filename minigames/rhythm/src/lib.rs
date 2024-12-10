mod palette;

use std::time::Duration;

use bevy::{
    prelude::*,
    render::view::visibility,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use palette::{
    ACTIVE_PROGRESS_BAR_COLOR, HIT_INPUT_MARKER, INACTIVE_PROGRESS_BAR_COLOR, PASSED_INPUT_MARKER,
    PENDING_INPUT_MARKER, PROGRESS_MARKER,
};
use sardips::{
    assets::FontAssets,
    despawn_all,
    interaction::MouseCamera,
    minigames::{rhythm_template::ActiveRhythmTemplate, MiniGameState},
    text_translation::KeyText,
};
use shared_deps::bevy_kira_audio::prelude::*;
use text_keys::MINIGAME_RHYTHM_LOADING;

pub struct RhythmPlugin;

impl Plugin for RhythmPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state::<RhythmState>(RhythmState::default())
            .add_systems(Startup, (initialize_materials, initialize_meshes))
            .add_systems(OnEnter(MiniGameState::PlayingRhythm), setup_game)
            .add_systems(OnExit(MiniGameState::PlayingRhythm), teardown_game)
            .add_systems(OnEnter(RhythmState::Exit), exit_game)
            .add_systems(OnEnter(RhythmState::Loading), setup_loading)
            .add_systems(OnExit(RhythmState::Loading), despawn_all::<RhythmLoading>)
            .add_systems(
                Update,
                check_loading_finished.run_if(in_state(RhythmState::Loading)),
            )
            .add_systems(OnEnter(RhythmState::Intro), setup_intro)
            .add_systems(OnExit(RhythmState::Intro), despawn_all::<RhythmIntro>)
            .add_systems(
                Update,
                tick_intro_expire.run_if(in_state(RhythmState::Intro)),
            )
            .add_systems(
                OnEnter(RhythmState::Playing),
                (setup_background, spawn_page_entities),
            )
            .add_systems(OnExit(RhythmState::Playing), despawn_all::<RhythmMain>)
            .add_systems(
                Update,
                (
                    tick_background,
                    populate_page,
                    setup_line,
                    tick_line,
                    tick_input_colors,
                    handle_click,
                )
                    .chain()
                    .run_if(in_state(RhythmState::Playing)),
            );
    }
}

const TEXT_KEY_PREFIX: &str = "minigame.rhythm.";

fn create_text_key(key: &str) -> String {
    format!("{}{}", TEXT_KEY_PREFIX, key)
}

#[derive(Resource)]
struct PreMadeMaterials {
    inactive_progress_bar: Handle<ColorMaterial>,
    active_progress_bar: Handle<ColorMaterial>,
    passed_input_marker: Handle<ColorMaterial>,
    hit_input_marker: Handle<ColorMaterial>,
    pending_input_marker: Handle<ColorMaterial>,
    progress_marker: Handle<ColorMaterial>,
}

fn initialize_materials(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(PreMadeMaterials {
        inactive_progress_bar: materials.add(INACTIVE_PROGRESS_BAR_COLOR),
        active_progress_bar: materials.add(ACTIVE_PROGRESS_BAR_COLOR),
        passed_input_marker: materials.add(PASSED_INPUT_MARKER),
        hit_input_marker: materials.add(HIT_INPUT_MARKER),
        pending_input_marker: materials.add(PENDING_INPUT_MARKER),
        progress_marker: materials.add(PROGRESS_MARKER),
    });
}

#[derive(Resource)]
struct PreMadeMeshes {
    input_marker_mesh: Handle<Mesh>,
    progress_marker_mesh: Handle<Mesh>,
}

fn initialize_meshes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(PreMadeMeshes {
        input_marker_mesh: meshes.add(Circle::new(10.)),
        progress_marker_mesh: meshes.add(Circle::new(10.)),
    });
}

#[derive(States, Default, Debug, Hash, Eq, PartialEq, Clone)]
enum RhythmState {
    #[default]
    None,
    Loading,
    Intro,
    Playing,
    Score,
    Exit,
}

#[derive(Component)]
struct Rhythm;

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut template: ResMut<ActiveRhythmTemplate>,
    mut minigame_state: ResMut<NextState<MiniGameState>>,
    mut rhythm_state: ResMut<NextState<RhythmState>>,
) {
    if template.selected_template.is_none() {
        minigame_state.set(MiniGameState::None);
        return;
    }

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        RhythmUiCamera,
        Rhythm,
    ));

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        MouseCamera,
        RhythmGameCamera,
        Rhythm,
    ));

    // Start loading
    let active_template = template.selected_template.as_mut().unwrap();
    active_template.start_load(&asset_server);

    rhythm_state.set(RhythmState::Loading);
}

fn teardown_game(
    mut commands: Commands,
    mut rhythm_state: ResMut<NextState<RhythmState>>,
    rhythms: Query<Entity, With<Rhythm>>,
) {
    for entity in rhythms.iter() {
        commands.entity(entity).despawn_recursive();
    }

    rhythm_state.set(RhythmState::None);
}

fn exit_game(mut commands: Commands, mut minigame_state: ResMut<NextState<MiniGameState>>) {
    minigame_state.set(MiniGameState::None);
}

#[derive(Component)]
struct RhythmGameCamera;

#[derive(Component)]
struct RhythmUiCamera;

#[derive(Component)]
struct RhythmLoading;

fn setup_loading(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    ui_camera: Query<Entity, With<RhythmUiCamera>>,
) {
    let ui_camera = ui_camera.single();
    commands.spawn((
        TargetCamera(ui_camera),
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
                    font: fonts.main_font.clone(),
                    font_size: 40.,
                    ..default()
                },
            ),
            ..default()
        },
        KeyText::new().with(0, MINIGAME_RHYTHM_LOADING),
        RhythmLoading,
        Rhythm,
    ));
}

fn check_loading_finished(
    asset_server: Res<AssetServer>,
    active_template: Res<ActiveRhythmTemplate>,
    mut state: ResMut<NextState<RhythmState>>,
) {
    if active_template
        .selected_template
        .as_ref()
        .unwrap()
        .loaded(&asset_server)
    {
        state.set(RhythmState::Intro);
    }
}

#[derive(Component)]
struct RhythmIntro;

#[derive(Component)]
struct MusicPlayer {
    handle: Handle<AudioInstance>,
}

fn setup_intro(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    audio: Res<Audio>,
    active_template: Res<ActiveRhythmTemplate>,
    ui_camera: Query<Entity, With<RhythmUiCamera>>,
) {
    let active_template = active_template.selected_template.as_ref().unwrap();

    let mut instance = audio.play(active_template.song_handle.as_ref().unwrap().clone());

    commands.spawn((
        MusicPlayer {
            handle: instance.handle(),
        },
        Rhythm,
    ));

    let image = active_template.intro.image_handle.as_ref().unwrap().clone();

    let ui_camera = ui_camera.single();

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
            RhythmIntro,
            Rhythm,
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                image: UiImage::new(image),
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
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                },

                ..default()
            },
            RhythmIntro,
            Rhythm,
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
                            font: fonts.main_font.clone(),
                            font_size: 40.,
                            ..default()
                        },
                    ),
                    ..default()
                },
                KeyText::new().with(0, create_text_key(&active_template.intro.intro_text_key)),
            ));
        });
}

fn tick_intro_expire(
    audio: Res<Audio>,
    active_template: Res<ActiveRhythmTemplate>,
    mut state: ResMut<NextState<RhythmState>>,
    player: Query<&MusicPlayer>,
) {
    let active_template = active_template.active();

    let player = player.single();

    let playback_state = audio.state(&player.handle);

    match playback_state {
        PlaybackState::Playing { position } => {
            if position >= active_template.intro.end {
                state.set(RhythmState::Playing);
            }
        }
        PlaybackState::Stopped => {
            state.set(RhythmState::Exit);
        }
        _ => {}
    }
}

#[derive(Component)]
struct RhythmMain;

#[derive(Component)]
struct BackgroundHandler {
    current_index: usize,
}

#[derive(Component)]
struct RhythmBackgroundImage;

fn setup_background(
    mut commands: Commands,
    active_template: Res<ActiveRhythmTemplate>,
    ui_camera: Query<Entity, With<RhythmUiCamera>>,
) {
    let active_template = active_template.selected_template.as_ref().unwrap();
    let ui_camera = ui_camera.single();

    commands.spawn((BackgroundHandler { current_index: 0 }, RhythmMain, Rhythm));

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
            RhythmIntro,
            Rhythm,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    image: UiImage::new(
                        active_template.backgrounds[0]
                            .image_handle
                            .as_ref()
                            .unwrap()
                            .clone(),
                    ),
                    ..default()
                },
                RhythmBackgroundImage,
            ));
        });
}

fn tick_background(
    time: Res<Time>,
    active_template: Res<ActiveRhythmTemplate>,
    audio: Res<Audio>,
    mut background_handler: Query<&mut BackgroundHandler>,
    mut background_image: Query<&mut UiImage, With<RhythmBackgroundImage>>,
    music_player: Query<&MusicPlayer>,
) {
    let mut background_handler = background_handler.single_mut();
    let active_template = active_template.active();
    let music_player = music_player.single();
    match audio.state(&music_player.handle) {
        PlaybackState::Playing { position } => {
            let current_index = background_handler.current_index;
            let current_background = &active_template.backgrounds[current_index];

            if position >= current_background.end
                && background_handler.current_index + 1 < active_template.backgrounds.len()
            {
                background_handler.current_index = current_index + 1;
                let next_background =
                    &active_template.backgrounds[background_handler.current_index];

                let mut image = background_image.single_mut();
                image.texture = next_background.image_handle.as_ref().unwrap().clone();
            }
        }
        _ => {}
    }
}

#[derive(Copy, Clone)]
enum ZLayer {
    Background = 1,
    Lyrics,
    ProgressBar,
    InputMarker,
    ProgressMarker,
}

impl ZLayer {
    fn to_f32(self) -> f32 {
        self as u8 as f32
    }
}

#[derive(Component)]
struct RhythmPageHandler;

#[derive(Component)]
struct RhythmPageIndex {
    page: usize,
}

#[derive(Component)]
struct RhythmLineIndex {
    line: usize,
}

#[derive(Bundle)]
struct RhythmPageBundle {
    page_handler: RhythmPageHandler,
    page_index: RhythmPageIndex,
    line_index: RhythmLineIndex,
}

#[derive(Component)]
struct RhythmLine {
    number: usize,
}

struct LineEntities {
    progress: Entity,
    text: Entity,
}

impl LineEntities {
    fn new(progress: Entity, text: Entity) -> Self {
        Self { progress, text }
    }
}

#[derive(Component)]
struct LineMap {
    map: Vec<LineEntities>,
}

#[derive(Component)]
struct ProgressMarker;

fn spawn_page_entities(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    pre_made_meshes: Res<PreMadeMeshes>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<PreMadeMaterials>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(pre_made_meshes.progress_marker_mesh.clone()),
            material: materials.progress_marker.clone(),
            transform: Transform::from_xyz(0., 0., ZLayer::ProgressMarker.to_f32()),
            ..default()
        },
        ProgressMarker,
        RhythmMain,
        Rhythm,
    ));

    let mut lines = vec![];

    // Spawn lines
    for i in 0..4 {
        let y_start = 200. - i as f32 * 130.;
        let y_text = y_start;
        let y_progress = y_start - 50.;

        let progress_bar_id = commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(400., 10.))),
                    material: materials.inactive_progress_bar.clone(),
                    transform: Transform::from_xyz(0., y_progress, ZLayer::ProgressBar.to_f32()),
                    ..default()
                },
                RhythmLine { number: i },
                RhythmMain,
                Rhythm,
            ))
            .id();

        let text_id = commands
            .spawn((
                Text2dBundle {
                    transform: Transform {
                        translation: Vec3::new(0., y_text, ZLayer::Lyrics.to_f32()),
                        ..default()
                    },
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: fonts.main_font.clone(),
                            font_size: 20.,
                            color: palette::INACTIVE_LYRIC_COLOR,
                        },
                    ),
                    ..default()
                },
                KeyText::new(),
                RhythmLine { number: i },
                RhythmMain,
                Rhythm,
            ))
            .id();
        lines.push(LineEntities::new(progress_bar_id, text_id));
    }

    commands.spawn((
        RhythmPageBundle {
            page_handler: RhythmPageHandler,
            page_index: RhythmPageIndex { page: 0 },
            line_index: RhythmLineIndex { line: 0 },
        },
        LineMap { map: lines },
        RhythmMain,
        Rhythm,
    ));
}

#[derive(Component)]
struct InputMarker {
    sound: Handle<AudioSource>,
    hit: bool,
}

impl InputMarker {
    fn new(sound: Handle<AudioSource>) -> Self {
        Self { sound, hit: false }
    }
}

fn populate_page(
    mut commands: Commands,
    active_template: Res<ActiveRhythmTemplate>,
    materials: Res<PreMadeMaterials>,
    meshes: Res<PreMadeMeshes>,
    mut page_handler: Query<
        (&LineMap, &RhythmPageIndex, &mut RhythmLineIndex),
        (With<RhythmPageHandler>, Changed<RhythmPageIndex>),
    >,
    mut text_lines: Query<(&mut KeyText, &mut Text, &RhythmLine)>,
    mut progress_lines: Query<(&mut Visibility, &mut Handle<ColorMaterial>), With<RhythmLine>>,
    progress_lines_trans: Query<&Transform, With<RhythmLine>>,
    existing_input_markers: Query<Entity, With<InputMarker>>,
) {
    if let Ok((line_map, page_index, mut line_index)) = page_handler.get_single_mut() {
        for entity in existing_input_markers.iter() {
            commands.entity(entity).despawn_recursive();
        }

        let active_template = active_template.active();
        if let Some(active_page) = active_template.pages.get(page_index.page) {
            line_index.line = 0;

            for (i, line) in active_page.iter().enumerate() {
                let (mut key_text, mut text, _) = text_lines.get_mut(line_map.map[i].text).unwrap();
                key_text.set(0, create_text_key(&line.text));
                text.sections[0].style.color = palette::INACTIVE_LYRIC_COLOR;

                let (mut visibility, mut mat) =
                    progress_lines.get_mut(line_map.map[i].progress).unwrap();
                *visibility = Visibility::Visible;
                *mat = materials.inactive_progress_bar.clone();
            }

            for i in active_page.len()..4 {
                let (mut key_text, _, _) = text_lines.get_mut(line_map.map[i].text).unwrap();
                key_text.set(0, "".to_string());

                let (mut visibility, _) = progress_lines.get_mut(line_map.map[i].progress).unwrap();
                *visibility = Visibility::Hidden;
            }

            // Spawn input marker here
            for input in &active_template.inputs {
                if input.page() != page_index.page {
                    continue;
                }

                let when = input.start();

                let line_number = input.line();

                let y = progress_lines_trans
                    .get(line_map.map[line_number].progress)
                    .unwrap()
                    .translation
                    .y;

                let line = &active_page[line_number];

                let progress = ((when - line.start) / (line.end - line.start)) as f32;

                let x = progress * 400. - 200.;

                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.input_marker_mesh.clone()),
                        material: materials.pending_input_marker.clone(),
                        transform: Transform::from_xyz(x, y, ZLayer::InputMarker.to_f32()),
                        ..default()
                    },
                    InputMarker::new(active_template.get_input_sound(&input)),
                    RhythmMain,
                    Rhythm,
                ));
            }
        } else {
            // End of game
            line_index.line = 0;
        }
    }
}

fn setup_line(
    active_template: Res<ActiveRhythmTemplate>,
    materials: Res<PreMadeMaterials>,
    page_handler: Query<
        (&LineMap, &RhythmPageIndex, &RhythmLineIndex),
        (With<RhythmPageHandler>, Changed<RhythmLineIndex>),
    >,
    mut lines: Query<&mut Text, With<RhythmLine>>,
    mut progress: Query<&mut Handle<ColorMaterial>, With<RhythmLine>>,
) {
    if let Ok((line_map, page_index, line_index)) = page_handler.get_single() {
        let active_template = active_template.active();
        let active_page = &active_template.pages[page_index.page];

        for i in 0..active_page.len() {
            let mut text = lines.get_mut(line_map.map[i].text).unwrap();
            text.sections[0].style.color = if i == line_index.line {
                palette::ACTIVE_LYRIC_COLOR
            } else {
                palette::INACTIVE_LYRIC_COLOR
            };

            let mut progress = progress.get_mut(line_map.map[i].progress).unwrap();
            *progress = if i == line_index.line {
                materials.active_progress_bar.clone()
            } else {
                materials.inactive_progress_bar.clone()
            };
        }
    }
}

fn tick_line(
    audio: Res<Audio>,
    active_template: Res<ActiveRhythmTemplate>,
    music_player: Query<&MusicPlayer>,
    mut page_handler: Query<
        (&LineMap, &mut RhythmPageIndex, &mut RhythmLineIndex),
        With<RhythmPageHandler>,
    >,
    progress_line: Query<&GlobalTransform, With<RhythmLine>>,
    mut marker: Query<&mut Transform, With<ProgressMarker>>,
) {
    let music_player = music_player.single();
    let (line_map, mut page_index, mut line_index) = page_handler.single_mut();

    let active_template = active_template.active();
    let active_page = &active_template.pages[page_index.page];
    let active_line = &active_page[line_index.line];

    let playback_state = audio.state(&music_player.handle);

    match playback_state {
        PlaybackState::Playing { position } => {
            if position > active_line.end {
                if line_index.line + 1 >= active_page.len() {
                    page_index.page += 1;
                } else {
                    line_index.line += 1;
                }
                return;
            }

            let progress_transform = progress_line
                .get(line_map.map[line_index.line].progress)
                .unwrap();

            let mut marker_transform = marker.single_mut();
            marker_transform.translation.y = progress_transform.translation().y;

            let progress =
                ((position - active_line.start) / (active_line.end - active_line.start)) as f32;

            marker_transform.translation.x = progress * 400. - 200.;
        }
        _ => {}
    }
}

const INPUT_THRESHOLD: f32 = 20.;
const INPUT_VOLUME: f64 = 0.5;

fn tick_input_colors(
    materials: Res<PreMadeMaterials>,
    mut input_markers: Query<(&mut Handle<ColorMaterial>, &GlobalTransform, &InputMarker)>,
    progress_markers: Query<&GlobalTransform, With<ProgressMarker>>,
) {
    let progress_marker = progress_markers.single();
    let x = progress_marker.translation().x;

    for (mut mat, input_transform, marker) in input_markers.iter_mut() {
        let input_x = input_transform.translation().x;
        let input_y = input_transform.translation().y;

        if input_y == progress_marker.translation().y {
            if input_x + INPUT_THRESHOLD < x && !marker.hit {
                *mat = materials.passed_input_marker.clone();
            }
        }
    }
}

fn handle_click(
    buttons: Res<ButtonInput<MouseButton>>,
    materials: Res<PreMadeMaterials>,
    audio: Res<Audio>,
    mut input_markers: Query<(
        &mut Handle<ColorMaterial>,
        &GlobalTransform,
        &mut InputMarker,
    )>,
    progress_markers: Query<&GlobalTransform, With<ProgressMarker>>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let progress_marker = progress_markers.single();
        let x = progress_marker.translation().x;

        for (mut mat, input_transform, mut marker) in input_markers.iter_mut() {
            let input_x = input_transform.translation().x;
            let input_y = input_transform.translation().y;

            if input_y != progress_marker.translation().y {
                continue;
            }

            if x > input_x - INPUT_THRESHOLD && x < input_x + INPUT_THRESHOLD {
                *mat = materials.hit_input_marker.clone();
                marker.hit = true;
                audio.play(marker.sound.clone()).with_volume(INPUT_VOLUME);
            }
        }
    }
}
