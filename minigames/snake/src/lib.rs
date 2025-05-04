#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
};
use sardips_core::{
    assets::{FontAssets, SnakeGameAssets},
    despawn_all,
    food_core::FoodTemplateDatabase,
    minigames_core::{
        MiniGameBackButton, MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType,
    },
    sprite_utils::get_adjusted_size,
    text_translation::{KeyString, KeyText},
};
use shared_deps::bevy_turborand::{DelegatedRng, GlobalRng};
use snake_texture_indexes::HORIZONTAL_BODY;
use text_keys::MINIGAME_SNAKE_SCORE;

pub struct SnakePlugin;

impl Plugin for SnakePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(SnakeState::default())
            .add_event::<SpawnFood>()
            .add_systems(OnEnter(MiniGameState::PlayingSnake), on_start_playing)
            .add_systems(OnExit(MiniGameState::PlayingSnake), despawn_all::<Snake>)
            .add_systems(OnEnter(SnakeState::Loading), setup_loading)
            .add_systems(Update, check_loading.run_if(in_state(SnakeState::Loading)))
            .add_systems(OnExit(SnakeState::Loading), despawn_all::<SnakeLoading>)
            .add_systems(
                OnEnter(SnakeState::Playing),
                (setup_camera_and_ui, spawn_world),
            )
            .add_systems(OnExit(SnakeState::Playing), despawn_all::<SnakePlaying>)
            .add_systems(
                Update,
                (
                    spawn_pending_food,
                    fill_out_snake_section,
                    change_head_direction,
                    food_respawn,
                    debug_input,
                    update_snake_sprites_facing,
                    increase_speed,
                    handle_collisions,
                    update_score_text,
                    check_out_of_bounds,
                )
                    .run_if(in_state(SnakeState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (move_snake_sections, check_collisions).run_if(in_state(SnakeState::Playing)),
            )
            .add_systems(OnEnter(SnakeState::Score), setup_score_screen)
            .add_systems(OnExit(SnakeState::Score), despawn_all::<SnakeScore>)
            .add_systems(
                Update,
                quit_button_pressed.run_if(in_state(SnakeState::Score)),
            )
            .add_systems(OnEnter(SnakeState::Exit), (despawn_all::<Snake>, on_exit));
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum SnakeState {
    #[default]
    None,
    Loading,
    Playing,
    Score,
    Exit,
}

#[derive(Component)]
struct Snake;

fn on_start_playing(mut state: ResMut<NextState<SnakeState>>) {
    state.set(SnakeState::Loading);
}

#[derive(Component)]
struct SnakeLoading;

fn setup_loading() {}

fn check_loading(mut state: ResMut<NextState<SnakeState>>) {
    state.set(SnakeState::Playing);
}

#[derive(Component)]
struct SnakePlaying;

#[derive(Component)]
struct SnakeBackgroundCamera;

#[derive(Component)]
struct SnakeUiCamera;

#[derive(Component)]
struct SnakeSpriteCamera;

#[derive(Component)]
struct SnakeScoreText;

#[derive(Copy, Clone)]
enum ZOrdering {
    #[allow(dead_code)]
    Background = 1,
    MovementGridBackground,
    MovementGrid,
    Snake,
    Food,
}

impl ZOrdering {
    fn to_f32(self) -> f32 {
        self as u8 as f32
    }
}

fn setup_camera_and_ui(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<SnakeGameAssets>,
    font_assets: Res<FontAssets>,
) {
    let background = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 0,
                    clear_color: ClearColorConfig::Custom(Color::srgb_u8(147, 250, 165)),
                    ..default()
                },
                ..default()
            },
            SnakeBackgroundCamera,
            Snake,
        ))
        .id();

    commands
        .spawn((
            TargetCamera(background),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            Snake,
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    height: Val::Percent(100.),
                    ..default()
                },
                image: UiImage::new(assets.background.clone()),
                ..default()
            });
        });

    let grid_background = materials.add(Color::srgba_u8(65, 152, 10, 20));
    let grid_border = materials.add(Color::srgba_u8(0, 0, 0, 50));

    for i in -5..=5 {
        for j in -5..=5 {
            commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Rectangle::new(CELL_SIZE, CELL_SIZE))),
                        material: grid_background.clone(),
                        transform: Transform::from_translation(Vec3::new(
                            i as f32 * CELL_SIZE,
                            j as f32 * CELL_SIZE,
                            ZOrdering::MovementGridBackground.to_f32(),
                        )),
                        ..default()
                    },
                    Snake,
                ))
                .with_children(|parent| {
                    const GIRTH: f32 = CELL_SIZE * 0.1;

                    const SIDES: [(f32, f32, f32, f32); 4] = [
                        // LEFT WALL
                        (GIRTH, CELL_SIZE, -(CELL_SIZE / 2.), 0.),
                        // RIGHT WALL
                        (GIRTH, CELL_SIZE, CELL_SIZE / 2., 0.),
                        // TOP WALL
                        (CELL_SIZE, GIRTH, 0., CELL_SIZE / 2.),
                        // BOTTOM WALL
                        (CELL_SIZE, GIRTH, 0., -(CELL_SIZE / 2.)),
                    ];

                    for (width, height, x, y) in SIDES.iter() {
                        parent.spawn((
                            MaterialMesh2dBundle {
                                mesh: Mesh2dHandle(meshes.add(Rectangle::new(*width, *height))),
                                material: grid_border.clone(),
                                transform: Transform::from_translation(Vec3::new(
                                    *x,
                                    *y,
                                    ZOrdering::MovementGrid.to_f32(),
                                )),
                                ..default()
                            },
                            Snake,
                        ));
                    }
                });
        }
    }

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        SnakeSpriteCamera,
        Snake,
    ));

    let ui_camera = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 2,
                    ..default()
                },
                ..default()
            },
            SnakeUiCamera,
            Snake,
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
            Snake,
            SnakePlaying,
        ))
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(15.),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba_u8(255, 0, 0, 128)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            text: Text::from_sections(vec![TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.main_font.clone(),
                                    font_size: 50.,
                                    color: Color::BLACK,
                                },
                            )]),
                            ..default()
                        },
                        KeyText::new().with_value(0, MINIGAME_SNAKE_SCORE, &["0"]),
                        SnakeScoreText,
                    ));
                });
        });
}

#[derive(Component)]
struct Player;

#[derive(Debug, Component, Copy, Clone, PartialEq, Eq)]
enum SectionDirection {
    Up,
    Down,
    Left,
    Right,
}

impl From<Vec2> for SectionDirection {
    fn from(vec: Vec2) -> Self {
        match vec {
            Vec2 { x, y: _ } if x > 0. => SectionDirection::Right,
            Vec2 { x, y: _ } if x < 0. => SectionDirection::Left,
            Vec2 { x: _, y } if y > 0. => SectionDirection::Up,
            Vec2 { x: _, y } if y < 0. => SectionDirection::Down,
            _ => SectionDirection::Right,
        }
    }
}

#[derive(Component)]
struct PlayerSection;

#[derive(Component)]
struct WorldBounds(Rect);

const CELL_SIZE: f32 = 38.;

const WORLD_WIDTH: f32 = 10.;
const WORLD_HEIGHT: f32 = 10.;

fn spawn_world(mut commands: Commands, mut spawn_food_events: EventWriter<SpawnFood>) {
    let head = commands
        .spawn((
            Player,
            Snake,
            HeadSection::new(CELL_SIZE * 1.5),
            PlayerSection,
            EatenHistory::default(),
            SectionDirection::Left,
        ))
        .id();
    commands.entity(head).insert(SnakeSection { head });

    for _ in 0..1 {
        // commands.spawn((SnakeSection { head }, Snake));
    }

    commands.spawn((
        WorldBounds(Rect::new(
            -(WORLD_WIDTH / 2.),
            -(WORLD_HEIGHT / 2.),
            WORLD_WIDTH / 2.,
            WORLD_HEIGHT / 2.,
        )),
        Snake,
    ));

    spawn_food_events.send(SpawnFood {});
}

#[derive(Component)]
struct SnakeScore;

fn setup_score_screen(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    food_templates: Res<FoodTemplateDatabase>,
    player: Query<&EatenHistory, With<Player>>,
    ui_camera: Query<Entity, With<SnakeUiCamera>>,
) {
    let ui_camera = ui_camera.single();
    let eaten_history = player.single();

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
            Snake,
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
                    text: Text::from_sections(vec![TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.main_font.clone(),
                            font_size: 50.,
                            color: Color::WHITE,
                        },
                    )]),
                    ..default()
                },
                KeyText::new().with_value(
                    0,
                    MINIGAME_SNAKE_SCORE,
                    &[eaten_history.score().to_string().as_str()],
                ),
                SnakeScoreText,
            ));

            let tally = eaten_history.eaten_tally();

            for (food, _) in tally.iter().take(5) {
                let template = food_templates.get(food).unwrap();
                parent.spawn((
                    TextBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        text: Text::from_sections(vec![TextSection::new(
                            "",
                            TextStyle {
                                font: font_assets.main_font.clone(),
                                font_size: 30.,
                                color: Color::WHITE,
                            },
                        )]),
                        ..default()
                    },
                    KeyText::new().with(0, format!("food.{}", template.name.to_lowercase())),
                ));
            }

            parent.spawn(MiniGameBackButton);
        });
}

#[derive(Component)]
struct QuitButton;

fn quit_button_pressed(
    mut state: ResMut<NextState<SnakeState>>,
    quit_buttons: Query<&Interaction, (With<QuitButton>, Changed<Interaction>)>,
) {
    for interaction in &quit_buttons {
        if interaction == &Interaction::Pressed {
            state.set(SnakeState::Exit);
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

#[derive(Event)]
struct SpawnFood {}

#[derive(Component)]
struct Food(String);

const COL_SIZE_MODIFIER: f32 = 0.6;

fn spawn_pending_food(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut spawn_food_events: EventReader<SpawnFood>,
    global_rng: ResMut<GlobalRng>,
    food_templates: Res<FoodTemplateDatabase>,
    world_bounds: Query<&WorldBounds>,
) {
    let rng = global_rng.into_inner();
    let world_bounds = world_bounds.single();

    for _ in spawn_food_events.read() {
        let template = food_templates.random(rng);

        let size = template.sprite_size.vec2(template.texture_size);

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(
                    rng.i32((world_bounds.0.min.x as i32)..(world_bounds.0.max.x as i32)) as f32
                        * CELL_SIZE,
                    rng.i32((world_bounds.0.min.y as i32)..(world_bounds.0.max.y as i32)) as f32
                        * CELL_SIZE,
                    ZOrdering::Food.to_f32(),
                )),
                sprite: Sprite {
                    custom_size: Some(get_adjusted_size(CELL_SIZE, size)),
                    ..default()
                },
                texture: asset_server.load(&template.texture),
                ..default()
            },
            Food(template.name.clone()),
            CollisionBox(Rect::new(
                -CELL_SIZE / 2. * COL_SIZE_MODIFIER,
                -CELL_SIZE / 2. * COL_SIZE_MODIFIER,
                CELL_SIZE * COL_SIZE_MODIFIER,
                CELL_SIZE * COL_SIZE_MODIFIER,
            )),
            Snake,
        ));
    }
}

#[derive(Component)]
struct HeadSection {
    sections: Vec<Entity>,
    speed: f32,
    history: Vec<Vec2>,
    next_direction: SectionDirection,
}

impl HeadSection {
    pub fn new(speed: f32) -> Self {
        let mut history = vec![];
        for i in 0..50 {
            history.push(Vec2::new(-(i as f32 * CELL_SIZE), 0.));
        }

        Self {
            sections: Vec::new(),
            speed,
            history,
            next_direction: SectionDirection::Right,
        }
    }
}

#[derive(Component)]
struct SnakeSection {
    head: Entity,
}

#[derive(Component)]
struct SectionBuilt;

mod snake_texture_indexes {
    pub const HORIZONTAL_BODY: usize = 0;
    pub const VERTICAL_BODY: usize = 1;
    pub const HORIZONTAL_HEAD: usize = 2;
    pub const VERTICAL_HEAD: usize = 3;
}

fn fill_out_snake_section(
    mut commands: Commands,
    assets: Res<SnakeGameAssets>,
    mut head_section: Query<&mut HeadSection>,
    new_sections: Query<(Entity, &SnakeSection), Without<SectionBuilt>>,
) {
    for (entity, section) in &new_sections {
        let head_entity = section.head;
        let mut head = head_section.get_mut(head_entity).unwrap();

        let (spawn_position, parent_pos) = (Vec2::new(0., 0.), Vec2::new(CELL_SIZE, 0.));

        commands.entity(entity).insert((
            SpriteBundle {
                transform: Transform::from_translation(
                    spawn_position.extend(ZOrdering::Snake.to_f32()),
                ),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(CELL_SIZE, CELL_SIZE)),
                    ..default()
                },
                texture: assets.snake.clone(),
                ..default()
            },
            TextureAtlas {
                layout: assets.snake_layout.clone(),
                index: HORIZONTAL_BODY,
            },
            Collides::default(),
            CollisionBox(Rect::new(
                (-CELL_SIZE / 2.) * COL_SIZE_MODIFIER,
                (-CELL_SIZE / 2.) * COL_SIZE_MODIFIER,
                (CELL_SIZE) * COL_SIZE_MODIFIER,
                (CELL_SIZE) * COL_SIZE_MODIFIER,
            )),
            // This doesn't matter for non head
            SectionDirection::Right,
            Snake,
            SectionBuilt,
            LastPosition(parent_pos),
        ));

        // Head should include a reference to the sections
        head.sections.push(entity);
    }
}

#[derive(Component)]
struct LastPosition(Vec2);

// I might be a fucking idiot like the biggest fucking idiot in the world spent like 4 hours trying to make some stupid fucking move each part on it's own work always looked like shit
// when I should just be setting each position relative to the head each frame FUCK ME im dumb as a sack a shit.
fn move_snake_sections(
    time: Res<Time>,
    mut heads: Query<(&mut HeadSection, &mut SectionDirection)>,
    mut sections: Query<(&mut Transform, &mut LastPosition), With<SnakeSection>>,
) {
    for (mut head, mut direction) in &mut heads {
        let mut percent_to_next_cell = 0.;
        for i in 0..head.sections.len() {
            if let Ok((mut trans, mut last_position)) = sections.get_mut(head.sections[i]) {
                if i == 0 {
                    let step_size = head.speed * time.delta_seconds();
                    let mut pos = trans.translation.truncate();
                    match *direction {
                        SectionDirection::Up => pos.y -= step_size,
                        SectionDirection::Down => pos.y += step_size,
                        SectionDirection::Left => pos.x -= step_size,
                        SectionDirection::Right => pos.x += step_size,
                    }

                    let last_pos = head.history[0];
                    let distance = last_pos.distance(pos);
                    percent_to_next_cell = distance / CELL_SIZE;
                    if distance >= CELL_SIZE {
                        if head.next_direction != *direction {
                            *direction = head.next_direction;
                        }

                        last_position.0 = pos;
                        // Update history
                        head.history.insert(0, pos);
                        if head.history.len() > (head.sections.len() + 1).max(50) {
                            head.history.pop();
                        }
                    }
                    trans.translation = pos.extend(trans.translation.z);
                } else {
                    let next = *head.history.get(i - 1).unwrap_or(&Vec2::ZERO);
                    let current = *head.history.get(i).unwrap_or(&Vec2::ZERO);
                    // Find the middle point based on the distance to the next cell
                    let middle = current + (next - current) * percent_to_next_cell;
                    trans.translation = middle.extend(trans.translation.z);
                }
            }
        }
    }
}

fn change_head_direction(
    mut heads: Query<&mut HeadSection, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mut head in &mut heads {
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            head.next_direction = SectionDirection::Down;
        } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            head.next_direction = SectionDirection::Up;
        } else if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            head.next_direction = SectionDirection::Left;
        } else if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            head.next_direction = SectionDirection::Right;
        }
    }
}

fn debug_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    heads: Query<Entity, With<HeadSection>>,
) {
    for head in &heads {
        if keyboard_input.just_pressed(KeyCode::Space) {
            commands.spawn((SnakeSection { head }, Snake));
        }
    }
}

#[derive(Component)]
struct CollisionBox(Rect);

#[derive(Component, Default)]
struct Collides {
    colliding_with: Vec<Entity>,
}

fn check_collisions(
    has_collisions: Query<(Entity, &Transform, &CollisionBox)>,
    mut to_check: Query<(Entity, &mut Collides)>,
) {
    for (entity, mut collides) in &mut to_check {
        let (_, transform, collision_box) = has_collisions.get(entity).unwrap();

        let rect_a_left = transform.translation.x - collision_box.0.width() / 2.;
        let rect_a_right = transform.translation.x + collision_box.0.width() / 2.;
        let rect_a_top = transform.translation.y + collision_box.0.height() / 2.;
        let rect_a_bottom = transform.translation.y - collision_box.0.height() / 2.;

        for (other_entity, other_transform, other_collision_box) in &has_collisions {
            if entity == other_entity {
                continue;
            }

            // check for overlap
            if rect_a_left < other_transform.translation.x + other_collision_box.0.width() / 2.
                && rect_a_right > other_transform.translation.x - other_collision_box.0.width() / 2.
                && rect_a_top > other_transform.translation.y - other_collision_box.0.height() / 2.
                && rect_a_bottom
                    < other_transform.translation.y + other_collision_box.0.height() / 2.
            {
                collides.colliding_with.push(other_entity);
            }
        }
    }
}

fn handle_collisions(
    mut commands: Commands,
    mut state: ResMut<NextState<SnakeState>>,
    mut player: Query<&mut EatenHistory, With<Player>>,
    heads: Query<&HeadSection, With<HeadSection>>,
    sections: Query<Entity, With<SnakeSection>>,
    foods: Query<&Food>,
    mut section_collides: Query<(Entity, &mut Collides), (Changed<Collides>, With<SnakeSection>)>,
) {
    for (entity, mut collides) in &mut section_collides {
        let head = heads.get(entity);
        for other_entity in &collides.colliding_with {
            if let Ok(head) = head {
                if let Ok(food) = foods.get(*other_entity) {
                    commands.entity(*other_entity).despawn_recursive();
                    commands.spawn((SnakeSection { head: entity }, Snake));
                    if let Ok(mut player) = player.get_mut(entity) {
                        player.foods.push(food.0.clone());
                    }
                } else if sections.get(*other_entity).is_ok()
                    && head.sections.contains(other_entity)
                    && *other_entity != head.sections[1]
                {
                    state.set(SnakeState::Score);
                }
            }
        }

        collides.colliding_with.clear();
    }
}

fn food_respawn(
    mut spawn_food_events: EventWriter<SpawnFood>,
    mut removed_food: RemovedComponents<Food>,
) {
    for _ in removed_food.read() {
        spawn_food_events.send(SpawnFood {});
    }
}

fn update_snake_sprites_facing(
    heads: Query<&HeadSection, With<SectionBuilt>>,
    mut sprites: Query<(&mut Sprite, &mut TextureAtlas, &SectionDirection), With<SnakeSection>>,
) {
    for head in &heads {
        for (i, section) in head.sections.iter().enumerate() {
            if let Ok((mut sprite, mut atlas, direction)) = sprites.get_mut(*section) {
                let (vertical_index, horizontal_index) = if i == 0 {
                    (
                        snake_texture_indexes::VERTICAL_HEAD,
                        snake_texture_indexes::HORIZONTAL_HEAD,
                    )
                } else {
                    (
                        snake_texture_indexes::VERTICAL_BODY,
                        snake_texture_indexes::HORIZONTAL_BODY,
                    )
                };

                let (index, flip_y, flip_x) = match *direction {
                    SectionDirection::Up => (vertical_index, true, false),
                    SectionDirection::Down => (vertical_index, false, false),
                    SectionDirection::Left => (horizontal_index, false, false),
                    SectionDirection::Right => (horizontal_index, false, true),
                };

                if atlas.index != index {
                    atlas.index = index;
                }

                if sprite.flip_x != flip_x {
                    sprite.flip_x = flip_x;
                }

                if sprite.flip_y != flip_y {
                    sprite.flip_y = flip_y;
                }
            }
        }
    }
}

fn increase_speed(time: Res<Time>, mut heads: Query<&mut HeadSection>) {
    for mut head in &mut heads {
        head.speed += 1. * time.delta_seconds();
    }
}

#[derive(Component, Default)]
struct EatenHistory {
    foods: Vec<String>,
}

impl EatenHistory {
    pub fn score(&self) -> usize {
        self.foods.len()
    }

    pub fn eaten_tally(&self) -> Vec<(&str, usize)> {
        let mut tally = HashMap::new();
        for food in &self.foods {
            *tally.entry(food.as_str()).or_insert(0) += 1;
        }

        let mut tally: Vec<_> = tally.into_iter().collect();
        tally.sort_by(|a, b| b.1.cmp(&a.1));
        tally
    }
}

fn update_score_text(
    mut score_texts: Query<&mut KeyText, With<SnakeScoreText>>,
    player: Query<&EatenHistory, (With<Player>, Changed<EatenHistory>)>,
) {
    for mut text in &mut score_texts {
        for player in &player {
            text.set_section(
                0,
                KeyString::value(MINIGAME_SNAKE_SCORE, &[&player.score().to_string()]),
            );
        }
    }
}

fn check_out_of_bounds(
    mut state: ResMut<NextState<SnakeState>>,
    heads: Query<&Transform, With<HeadSection>>,
    world_bounds: Query<&WorldBounds>,
) {
    let world_bounds = world_bounds.single();
    let bounds = Rect::new(
        world_bounds.0.min.x * CELL_SIZE + (CELL_SIZE * 0.5),
        world_bounds.0.min.y * CELL_SIZE - (CELL_SIZE * 0.5),
        world_bounds.0.max.x * CELL_SIZE + (CELL_SIZE * 0.5),
        world_bounds.0.max.y * CELL_SIZE - (CELL_SIZE * 0.5),
    );
    for head in &heads {
        if !bounds.contains(head.translation.truncate()) {
            state.set(SnakeState::Score);
        }
    }
}
