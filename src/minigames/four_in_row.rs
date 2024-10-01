use core::fmt;

use bevy::prelude::*;
use bevy_turborand::{DelegatedRng, GlobalRng, RngComponent};

use super::{
    MiniGameBackExitButton, MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType, Playing,
};
use crate::{
    assets::{FontAssets, FourInRowAssets},
    interaction::{AttachToCursor, Clickable, Hovering, MouseCamera},
    pet::{
        mood::{AutoSetMoodImage, MoodCategory, MoodImages},
        move_towards::{MoveTowardsOnSpawn, MovingTowards},
    },
    sounds::{PlaySoundEffect, SoundEffect},
    velocity::{MovementDirection, Speed},
};
use bevy_prototype_lyon::prelude::*;

pub struct FourInRowPlugin;

impl Plugin for FourInRowPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(FourInRowState::default())
            .add_event::<Move>()
            .add_systems(
                OnEnter(MiniGameState::PlayingFourInRow),
                (setup_background, setup_game),
            )
            .add_systems(
                OnExit(MiniGameState::PlayingFourInRow),
                (send_complete, teardown).chain(),
            )
            .add_systems(OnExit(FourInRowState::Playing), teardown_playing)
            .add_systems(OnEnter(FourInRowState::GameOver), setup_game_over)
            .add_systems(
                Update,
                (update_mood_image).run_if(in_state(MiniGameState::PlayingFourInRow)),
            )
            .add_systems(
                Update,
                (
                    handle_input,
                    process_move,
                    run_computer_turn,
                    update_turn_display,
                    convert_falling_discs,
                )
                    .run_if(in_state(FourInRowState::Playing)),
            )
            .add_systems(
                Update,
                (convert_falling_discs, check_game_over)
                    .run_if(in_state(FourInRowState::WaitingForFalling)),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum FourInRowState {
    #[default]
    None,
    Playing,
    WaitingForFalling,
    GameOver,
}

#[derive(Component)]
struct FourInRow;

#[derive(Component)]
struct FourInRowPet;

#[derive(Component)]
struct FourInRowCamera;

fn setup_background(mut commands: Commands, assets: Res<FourInRowAssets>) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        MouseCamera,
        FourInRow,
        FourInRowCamera,
    ));

    let ui_camera = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 0,
                    ..default()
                },
                ..default()
            },
            FourInRow,
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
            FourInRow,
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
}

const DISC_LAYER: f32 = 3.;
const BOARD_LAYER: f32 = 5.;
const LINE_LAYER: f32 = 6.;

fn setup_game(
    mut commands: Commands,
    mut state: ResMut<NextState<FourInRowState>>,
    assets: Res<FourInRowAssets>,
    fonts: Res<FontAssets>,
    mut rng: ResMut<GlobalRng>,
    pet_sheet: Query<(&Handle<Image>, &TextureAtlas, &Sprite, &MoodImages), With<Playing>>,
) {
    state.set(FourInRowState::Playing);

    let mut board = commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(30., 50., 0.))
                .with_scale(Vec3::new(0.75, 0.75, 1.)),
            ..default()
        },
        GameBoard(Board::blank()),
        FourInRow,
    ));

    for row in 0..ROWS {
        for column in 0..COLUMNS {
            // Make the first column clickable
            // Should this be UI probably?
            board.with_children(|parent| {
                let mut builder = parent.spawn((
                    SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(
                            (column as f32 - COLUMNS as f32 / 2.) * 80.,
                            (row as f32 - ROWS as f32 / 2.) * 80.,
                            BOARD_LAYER,
                        )),
                        sprite: Sprite {
                            custom_size: Some(Vec2::new(80., 80.)),
                            ..default()
                        },
                        texture: assets.board_tile.clone(),
                        ..default()
                    },
                    Square(square_to_index(row, column)),
                    FourInRow,
                ));

                if row == ROWS - 1 {
                    builder.insert(Clickable::new(Vec2::new(-40., 40.), Vec2::new(-40., 40.)));
                }
            });
        }
    }

    let players = [
        commands.spawn((Player(Side::Red), FourInRow)).id(),
        commands.spawn((Player(Side::Yellow), FourInRow)).id(),
    ];

    let player_color = match rng.i32(0..2) {
        0 => Side::Red,
        1 => Side::Yellow,
        _ => unreachable!(),
    };

    let (player_index, computer_index) = match player_color {
        Side::Red => (0, 1),
        Side::Yellow => (1, 0),
    };

    commands.entity(players[player_index]).insert(HumanInput);
    commands
        .entity(players[computer_index])
        .insert((Computer::default(), RngComponent::from(&mut rng)));

    // Spawn pet image
    let (image, atlas, sprite, mood_images) = pet_sheet.single();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0., -255., DISC_LAYER)),
            sprite: Sprite {
                custom_size: sprite.custom_size,
                ..default()
            },
            texture: image.clone(),
            ..default()
        },
        atlas.clone(),
        mood_images.clone(),
        MoodCategory::Neutral,
        AutoSetMoodImage,
        FourInRowPet,
        FourInRow,
    ));

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., DISC_LAYER))
                .with_scale(Vec3::new(0.75, 0.75, 1.)),
            sprite: Sprite {
                custom_size: Some(Vec2::new(80., 80.)),
                ..default()
            },
            texture: assets.discs.clone(),
            ..default()
        },
        TextureAtlas {
            layout: assets.layout.clone(),
            index: player_color.to_sprite_index(),
            ..default()
        },
        AttachToCursor,
        InputDisc,
        FourInRow,
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    margin: UiRect::top(Val::Px(50.)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            FourInRow,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    transform: Transform::from_translation(Vec3::new(0., 0., 0.))
                        .with_scale(Vec3::new(0.5, 0.5, 1.)),
                    image: UiImage::new(assets.discs.clone()),
                    ..default()
                },
                TextureAtlas {
                    layout: assets.layout.clone(),
                    ..default()
                },
                TurnDiscDisplay,
            ));

            parent.spawn((
                TextBundle {
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
                TurnTextDisplay,
            ));
        });
}

fn teardown(mut commands: Commands, to_delete: Query<Entity, With<FourInRow>>) {
    for entity in to_delete.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn teardown_playing(
    mut commands: Commands,
    to_despawn: Query<Entity, Or<(With<TurnTextDisplay>, With<InputDisc>)>>,
) {
    for entity in to_despawn.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_game_over(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    board: Query<&GameBoard>,
    discs: Query<(&Transform, &Disc)>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    margin: UiRect::top(Val::Px(20.)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            FourInRow,
        ))
        .with_children(|parent| {
            MiniGameBackExitButton::spawn(parent, &fonts);
        });

    let board = board.single();

    match board.0.status() {
        GameStatus::InProgress => panic!("Game not over"),
        GameStatus::Draw => {}
        GameStatus::Win(info) => {
            let mut points = vec![];

            for (transform, disc) in discs.iter() {
                if info.line & (1 << disc.0) != 0 {
                    points.push(transform.translation);
                }
            }

            // Find two furthest points
            let mut max_distance = 0.;
            let mut max_points = (Vec3::ZERO, Vec3::ZERO);
            for point1 in points.iter() {
                for point2 in points.iter() {
                    let distance = point1.distance(*point2);
                    if distance > max_distance {
                        max_distance = distance;
                        max_points = (*point1, *point2);
                    }
                }
            }

            // spawn line between points
            let mut path_builder = PathBuilder::new();
            path_builder.move_to(Vec2::new(max_points.0.x, max_points.0.y));
            path_builder.line_to(Vec2::new(max_points.1.x, max_points.1.y));
            path_builder.close();
            let path = path_builder.build();

            commands.spawn((
                ShapeBundle {
                    path,
                    spatial: SpatialBundle {
                        transform: Transform::from_xyz(0., 0., LINE_LAYER),
                        ..default()
                    },
                    ..default()
                },
                Stroke::new(Color::Srgba(bevy::color::palettes::css::LIMEGREEN), 3.0),
                Fill::color(Color::Srgba(bevy::color::palettes::css::LIMEGREEN)),
                FourInRow,
            ));
        }
    }
}

fn send_complete(
    mut event_writer: EventWriter<MiniGameCompleted>,
    board: Query<&GameBoard>,
    player: Query<&Player, With<HumanInput>>,
) {
    let board = board.single();
    let player = player.single().0;

    event_writer.send(MiniGameCompleted {
        game_type: MiniGameType::FourInRow,
        result: match board.0.status() {
            GameStatus::InProgress => MiniGameResult::Incomplete,
            GameStatus::Draw => MiniGameResult::Draw,
            GameStatus::Win(side) => {
                if side.player == player {
                    MiniGameResult::Win
                } else {
                    MiniGameResult::Lose
                }
            }
        },
    });
}

#[derive(Event)]
struct Move {
    col: usize,
}

impl Move {
    fn new(index: usize) -> Self {
        Self { col: index }
    }
}

#[derive(Component)]
struct Disc(usize);

#[derive(Component)]
struct FallingDisc;

fn process_move(
    mut commands: Commands,
    mut moves: EventReader<Move>,
    mut state: ResMut<NextState<FourInRowState>>,
    assets: Res<FourInRowAssets>,
    mut board: Query<(&mut GameBoard, &Transform)>,
    squares: Query<(&Square, &GlobalTransform)>,
) {
    let (mut board, board_trans) = board.single_mut();

    for move_event in moves.read() {
        if board
            .0
            .possible_moves()
            .iter()
            .find(|&&col| col == move_event.col)
            .is_none()
        {
            error!("Column full");
            continue;
        }

        let current_player = board.0.side_to_move;

        board.0 = board.0.make_move_new(move_event.col);

        let (row, column) = board.0.last_move.unwrap();

        let target = squares
            .iter()
            .find(|(square, _)| square.0 == square_to_index(row, column))
            .unwrap()
            .1
            .translation();

        let source = {
            // Get position of the input column
            let trans = squares
                .iter()
                .find(|(square, _)| square.0 == square_to_index(ROWS - 1, move_event.col))
                .unwrap()
                .1
                .translation();

            trans.xy()
        };

        commands.spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(source.x, source.y, DISC_LAYER))
                    .with_scale(board_trans.scale),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(80., 80.)),
                    ..default()
                },
                texture: assets.discs.clone(),
                ..default()
            },
            TextureAtlas {
                layout: assets.layout.clone(),
                index: current_player.to_sprite_index(),
                ..default()
            },
            Speed(250.),
            MovementDirection {
                direction: Vec2::ZERO,
            },
            Disc(square_to_index(row, column)),
            MoveTowardsOnSpawn {
                target: target.xy(),
            },
            FallingDisc,
            FourInRow,
        ));

        // Check game over
        match board.0.status() {
            GameStatus::InProgress => {}
            GameStatus::Draw | GameStatus::Win(_) => {
                state.set(FourInRowState::WaitingForFalling);
            }
        }
    }
}

fn convert_falling_discs(
    mut commands: Commands,
    mut sounds: EventWriter<PlaySoundEffect>,
    falling_discs: Query<Entity, With<FallingDisc>>,
    fallen_discs: Query<
        Entity,
        (
            With<Disc>,
            Without<MoveTowardsOnSpawn>,
            Without<MovingTowards>,
        ),
    >,
) {
    for entity in falling_discs.iter() {
        if fallen_discs.get(entity).is_ok() {
            commands.entity(entity).remove::<FallingDisc>();
            sounds.send(PlaySoundEffect::new(SoundEffect::PlasticDrop));
        }
    }
}

fn check_game_over(
    mut state: ResMut<NextState<FourInRowState>>,
    falling_discs: Query<Entity, With<FallingDisc>>,
) {
    if falling_discs.iter().count() == 0 {
        info!("All discs have fallen Game over");
        state.set(FourInRowState::GameOver);
    }
}

#[derive(Component)]
struct Computer {
    move_timer: Timer,
}

impl Default for Computer {
    fn default() -> Self {
        Self {
            move_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        }
    }
}

#[derive(Component)]
struct Player(Side);

#[derive(Component)]
struct HumanInput;

fn run_computer_turn(
    time: Res<Time>,
    mut moves: EventWriter<Move>,
    board: Query<&GameBoard>,
    mut computer: Query<(&Player, &mut Computer, &mut RngComponent)>,
) {
    let (player, mut computer, mut rng) = match computer.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    let board = board.single();

    if board.0.side_to_move != player.0 {
        return;
    }

    if computer.move_timer.tick(time.delta()).just_finished() {
        let possible_moves = if rng.i32(0..7) != 0 {
            best_moves(&board.0)
        } else {
            board.0.possible_moves()
        };
        let selected_move = possible_moves[rng.usize(0..possible_moves.len() as usize)];
        moves.send(Move::new(selected_move));
    }
}

#[derive(Component)]
struct InputDisc;

fn handle_input(
    mut moves: EventWriter<Move>,
    buttons: Res<ButtonInput<MouseButton>>,
    board: Query<&GameBoard>,
    player: Query<&Player, With<HumanInput>>,
    mut player_disc_visibility: Query<&mut Visibility, With<InputDisc>>,
    squares: Query<&Square, With<Hovering>>,
) {
    let player = match player.get_single() {
        Ok(player) => player,
        Err(_) => return,
    };

    let mut visibility = player_disc_visibility.single_mut();

    let board = board.single();
    if board.0.side_to_move != player.0 {
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Visible;

    let square = match squares.get_single() {
        Ok(disc) => disc,
        Err(_) => return,
    };

    if buttons.just_pressed(MouseButton::Left) {
        moves.send(Move::new(square.0 % COLUMNS));
    }
}

#[derive(Component)]
struct TurnTextDisplay;

#[derive(Component)]
struct TurnDiscDisplay;

fn update_turn_display(
    mut turn_display: Query<&mut Text, With<TurnTextDisplay>>,
    mut disc_display: Query<&mut TextureAtlas, With<TurnDiscDisplay>>,
    board: Query<&GameBoard, Changed<GameBoard>>,
    player: Query<&Player, With<HumanInput>>,
) {
    let board = match board.get_single() {
        Ok(board) => board,
        Err(_) => return,
    };

    let player = player.single();

    let mut turn_display_text = turn_display.single_mut();

    turn_display_text.sections[0].value = if board.0.side_to_move == player.0 {
        "Your Turn".to_string()
    } else {
        "Computer's Turn".to_string()
    };

    let mut disc_display = disc_display.single_mut();

    disc_display.index = board.0.side_to_move.to_sprite_index();
}

fn update_mood_image(
    mut mood_images: Query<&mut MoodCategory, With<FourInRowPet>>,
    player: Query<&Player, With<HumanInput>>,
    board: Query<&GameBoard, Changed<GameBoard>>,
) {
    let board = match board.get_single() {
        Ok(board) => board,
        Err(_) => return,
    };

    let mut mood = mood_images.single_mut();

    let player = player.single();

    *mood = match board.0.status() {
        GameStatus::InProgress => {
            // Players turn
            let win = any_move_results_in_win(&board.0);

            match (board.0.side_to_move == player.0, win.is_some()) {
                (true, true) => MoodCategory::Happy,
                (false, true) => MoodCategory::Sad,
                _ => MoodCategory::Neutral,
            }
        }
        GameStatus::Draw => MoodCategory::Sad,
        GameStatus::Win(info) => {
            if player.0 == info.player {
                MoodCategory::Ecstatic
            } else {
                MoodCategory::Despairing
            }
        }
    };
}

#[derive(Component)]
struct GameBoard(Board);

#[derive(Component)]
struct Square(usize);

const ROWS: usize = 6;
const COLUMNS: usize = 7;

const BOARD_SIZE: usize = ROWS * COLUMNS;

// Should probably pre calculated these at compile time
lazy_static! {
    static ref WINNING_COMBINATIONS: Vec<BitBoard> = generate_n_long_lines(4);
}

fn generate_n_long_lines(n: usize) -> Vec<BitBoard> {
    let mut lines = Vec::new();

    // Horizontal
    for row in 0..ROWS {
        for column in 0..COLUMNS - n + 1 {
            let mut line = 0;
            for i in 0..n {
                line |= 1 << (row * COLUMNS + column + i);
            }
            lines.push(line);
        }
    }

    // Vertical
    for row in 0..ROWS - n + 1 {
        for column in 0..COLUMNS {
            let mut line = 0;
            for i in 0..n {
                line |= 1 << ((row + i) * COLUMNS + column);
            }
            lines.push(line);
        }
    }

    // Diagonal
    for row in 0..ROWS - n + 1 {
        for column in 0..COLUMNS - n + 1 {
            let mut line = 0;
            for i in 0..n {
                line |= 1 << ((row + i) * COLUMNS + column + i);
            }
            lines.push(line);
        }
    }

    // Anti-diagonal
    for row in 0..ROWS - n + 1 {
        for column in n - 1..COLUMNS {
            let mut line = 0;
            for i in 0..n {
                line |= 1 << ((row + i) * COLUMNS + column - i);
            }
            lines.push(line);
        }
    }

    lines
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Side {
    Red,
    Yellow,
}

impl Side {
    fn to_index(&self) -> usize {
        match self {
            Side::Red => 0,
            Side::Yellow => 1,
        }
    }

    fn to_sprite_index(&self) -> usize {
        match self {
            Side::Red => 1,
            Side::Yellow => 2,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Red => write!(f, "Red"),
            Side::Yellow => write!(f, "Yellow"),
        }
    }
}

const ALL_PLAYERS: [Side; 2] = [Side::Red, Side::Yellow];

type BitBoard = u64;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct WinInfo {
    player: Side,
    line: BitBoard,
}

impl WinInfo {
    fn new(player: Side, line: BitBoard) -> Self {
        Self { player, line }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum GameStatus {
    InProgress,
    Draw,
    Win(WinInfo),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Board {
    bb: [BitBoard; 2],
    side_to_move: Side,
    last_move: Option<(usize, usize)>,
}

impl Board {
    fn blank() -> Self {
        Self {
            bb: [0, 0],
            side_to_move: Side::Red,
            last_move: None,
        }
    }

    fn get(&self, square: usize) -> Option<Side> {
        for player in ALL_PLAYERS.iter() {
            if self.bb[player.to_index()] & (1 << square) != 0 {
                return Some(*player);
            }
        }

        None
    }

    fn make_move_new(&self, column: usize) -> Self {
        let mut result = *self;

        let mut row = 0;
        while row < ROWS && result.get(row * COLUMNS + column).is_some() {
            row += 1;
        }
        if row < ROWS {
            result.bb[result.side_to_move.to_index()] |= 1 << (row * COLUMNS + column);
        }

        result.last_move = Some((row, column));

        result.side_to_move = match result.side_to_move {
            Side::Red => Side::Yellow,
            Side::Yellow => Side::Red,
        };

        result
    }

    fn complete_board(&self) -> BitBoard {
        let mut result = 0;

        for player in ALL_PLAYERS.iter() {
            result |= self.bb[player.to_index()];
        }

        result
    }

    fn possible_moves(&self) -> Vec<usize> {
        let mut moves = Vec::new();

        let complete_board = self.complete_board();

        for column in 0..COLUMNS {
            let top_cell_mask = 1 << ((ROWS - 1) * COLUMNS + column);
            if complete_board & top_cell_mask == 0 {
                moves.push(column);
            }
        }

        moves
    }

    fn status(&self) -> GameStatus {
        for combination in WINNING_COMBINATIONS.iter() {
            for player in ALL_PLAYERS.iter() {
                if self.bb[player.to_index()] & combination == *combination {
                    return GameStatus::Win(WinInfo::new(*player, *combination));
                }
            }
        }

        if self.complete_board().count_ones() == BOARD_SIZE as u32 {
            return GameStatus::Draw;
        }

        GameStatus::InProgress
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in (0..ROWS).rev() {
            write!(f, "{} ", row)?; // Write the row number at the start of each row
            for column in 0..COLUMNS {
                let square = row * COLUMNS + column; // Corrected here
                let player = self.get(square);
                match player {
                    Some(Side::Red) => write!(f, "R ")?,
                    Some(Side::Yellow) => write!(f, "Y ")?,
                    None => write!(f, ". ")?,
                }
            }
            writeln!(f)?;
        }

        write!(f, "  ")?; // Add some space for alignment
        for column in 0..COLUMNS {
            write!(f, "{} ", column)?; // Write the column numbers at the bottom
        }
        writeln!(f)?;

        Ok(())
    }
}

const WIN_EVAL: f32 = f32::MAX;
const LOSE_EVAL: f32 = f32::MIN;
const UNKNOWN_EVAL: f32 = 0.;

fn evaluate(board: &Board) -> f32 {
    match board.status() {
        GameStatus::InProgress | GameStatus::Draw => UNKNOWN_EVAL,
        GameStatus::Win(info) => {
            if info.player == board.side_to_move {
                WIN_EVAL
            } else {
                LOSE_EVAL
            }
        }
    }
}

fn nega_max(board: &Board, depth: i32) -> f32 {
    let cols = board.possible_moves();
    if depth == 0 || cols.is_empty() {
        return evaluate(board);
    }

    let mut max = f32::NEG_INFINITY;

    for col in cols {
        let new_board = board.make_move_new(col);
        let score = -nega_max(&new_board, depth - 1);
        if score > max {
            max = score;
        }
    }

    return max;
}

fn any_move_results_in_win(board: &Board) -> Option<Player> {
    let board = *board;

    let cols = board.possible_moves();
    for col in cols {
        let new_board = board.make_move_new(col);
        if matches!(new_board.status(), GameStatus::Win(_)) {
            return Some(Player(board.side_to_move));
        }
    }

    None
}

fn best_moves(board: &Board) -> Vec<usize> {
    // if board is empty go middle
    if board.bb.iter().all(|&bb| bb == 0) {
        return vec![COLUMNS / 2];
    }

    let mut best_moves = Vec::new();
    let mut max = f32::MIN;
    let cols = board.possible_moves();
    for col in &cols {
        let new_board = board.make_move_new(*col);
        let score = -nega_max(&new_board, 5);
        if score > max {
            max = score;
            best_moves.clear();
            best_moves.push(*col);
        } else if score == max {
            best_moves.push(*col);
        }
    }

    if best_moves.is_empty() {
        cols
    } else {
        best_moves
    }
}

fn square_to_index(row: usize, column: usize) -> usize {
    row * COLUMNS + column
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_move() {
        let mut board = Board::blank();

        board = board.make_move_new(0);

        assert_eq!(board.side_to_move, Side::Yellow);
        assert_eq!(board.get(square_to_index(0, 0)), Some(Side::Red));

        board = board.make_move_new(0);

        assert_eq!(board.side_to_move, Side::Red);
        assert_eq!(board.get(square_to_index(1, 0)), Some(Side::Yellow));
    }

    #[test]
    fn test_valid_moves() {
        let mut board = Board::blank();

        assert_eq!(board.possible_moves(), vec![0, 1, 2, 3, 4, 5, 6]);

        for _ in 0..ROWS {
            board = board.make_move_new(0);
        }

        assert_eq!(board.possible_moves(), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_best_moves() {
        let mut board = Board::blank();

        let moves = best_moves(&board);

        assert_eq!(moves, vec![3]);

        // Red
        board = board.make_move_new(3);
        // Yellow
        board = board.make_move_new(0);
        // Red
        board = board.make_move_new(3);
        // Yellow
        board = board.make_move_new(1);
        // Red
        board = board.make_move_new(3);
        // Yellow

        let moves = best_moves(&board);

        assert_eq!(moves, vec![3]);
    }

    #[test]
    fn test_draws() {
        let mut board = Board::blank();

        while board.status() == GameStatus::InProgress {
            let moves = best_moves(&board);
            let move_index = moves[0];
            board = board.make_move_new(move_index);
        }

        assert_eq!(board.status(), GameStatus::Draw);
    }
}
