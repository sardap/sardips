#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
use bevy::prelude::*;
use sardips_core::{
    assets::{self, FontAssets, TicTacToeAssets},
    autoscroll::AutoScroll,
    button_hover::{ButtonColorSet, ButtonHover},
    minigames_core::{
        MiniGameBackButton, MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType, Playing,
    },
    mood_core::{AutoSetMoodImage, MoodCategory, MoodImageIndexes},
    sounds::{PlaySoundEffect, SoundEffect},
    text_translation::KeyText,
};
use shared_deps::{
    bevy_parallax::{
        CreateParallaxEvent, LayerComponent, LayerData, LayerSpeed, ParallaxCameraComponent,
    },
    bevy_turborand::{DelegatedRng, GlobalRng},
};

pub struct TicTacToePlugin;

impl Plugin for TicTacToePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(TicTacToeState::default())
            .add_event::<MakeMove>()
            .add_systems(
                OnEnter(MiniGameState::PlayingTicTakToe),
                (setup_camera, setup_game),
            )
            .add_systems(
                OnExit(MiniGameState::PlayingTicTakToe),
                (send_complete, teardown).chain(),
            )
            .add_systems(OnEnter(TicTacToeState::GameOver), setup_game_over)
            .add_systems(
                Update,
                (square_button_pressed, handle_move).run_if(in_state(TicTacToeState::Playing)),
            )
            .add_systems(
                Update,
                (update_board, update_pet_mood).run_if(in_state(MiniGameState::PlayingTicTakToe)),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum TicTacToeState {
    #[default]
    None,
    Playing,
    GameOver,
}

#[derive(Component)]
struct TicTacToe;

#[derive(Component)]
struct TicTacToeCamera;

fn setup_camera(mut commands: Commands, mut create_parallax: EventWriter<CreateParallaxEvent>) {
    let camera: Entity = commands
        .spawn((
            Camera2dBundle::default(),
            ParallaxCameraComponent::default(),
            AutoScroll::new(Vec2::new(0.0, 70.)),
            TicTacToe,
            TicTacToeCamera,
        ))
        .id();

    create_parallax.send(CreateParallaxEvent {
        camera,
        layers_data: vec![LayerData {
            speed: LayerSpeed::Vertical(0.5),
            path: assets::TicTacToeAssets::BACKGROUND.to_string(),
            tile_size: UVec2::new(30, 30),
            cols: 1,
            rows: 1,
            scale: Vec2::splat(1.0),
            z: 0.0,
            ..Default::default()
        }],
    });
}

fn setup_game(
    mut commands: Commands,
    mut tic_state: ResMut<NextState<TicTacToeState>>,
    assets: Res<TicTacToeAssets>,
    pet_sheet: Query<(&Handle<Image>, &TextureAtlas, &MoodImageIndexes), With<Playing>>,
) {
    commands.spawn((Game::new(), TicTacToe));
    tic_state.set(TicTacToeState::Playing);

    let (image, atlas, mood_images) = pet_sheet.single();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            TicTacToe,
            TicTacToeUiRoot,
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
                            height: Val::Px(300.),
                            width: Val::Px(300.),
                            // top: Val::Px(50.),
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::flex(3, 1.),
                            grid_template_rows: RepeatedGridTrack::flex(3, 1.),
                            row_gap: Val::Px(5.),
                            column_gap: Val::Px(5.),
                            border: UiRect::all(Val::Px(10.)),
                            ..default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        background_color: BackgroundColor(Color::BLACK),
                        ..default()
                    },
                    BoardBackground,
                ))
                .with_children(|parent| {
                    for square in ALL_SQUARES {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        display: Display::Grid,
                                        padding: UiRect::all(Val::Px(3.0)),
                                        ..default()
                                    },
                                    ..default()
                                },
                                ButtonHover::default().with_background(ButtonColorSet::new(
                                    Color::Srgba(bevy::color::palettes::css::BEIGE),
                                    Color::WHITE,
                                    Color::Srgba(bevy::color::palettes::css::BEIGE),
                                )),
                                SquareButton(square),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    ImageBundle {
                                        image: UiImage::new(assets.sprites.clone()),
                                        style: Style {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                    TextureAtlas {
                                        layout: assets.layout.clone(),
                                        index: 0,
                                    },
                                    SquareImage,
                                ));
                            });
                    }
                });
        });
}

fn send_complete(mut event_writer: EventWriter<MiniGameCompleted>, game: Query<&Game>) {
    let game = game.single();

    let result = match game.status {
        BoardStatus::Draw => MiniGameResult::Draw,
        BoardStatus::Win(info) => match info.side {
            Side::X => MiniGameResult::Win,
            Side::O => MiniGameResult::Lose,
        },
        _ => MiniGameResult::Incomplete,
    };

    event_writer.send(MiniGameCompleted {
        game_type: MiniGameType::TicTacToe,
        result,
    });
}

fn teardown(
    mut commands: Commands,
    mut tic_state: ResMut<NextState<TicTacToeState>>,
    query: Query<Entity, Or<(With<TicTacToe>, With<LayerComponent>)>>,
) {
    tic_state.set(TicTacToeState::None);

    for entity in &mut query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_game_over(
    mut commands: Commands,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
    mut sounds: EventWriter<PlaySoundEffect>,
    fonts: Res<FontAssets>,
    game: Query<&Game>,
    buttons: Query<Entity, (With<SquareButton>, With<Interaction>)>,
    camera: Query<Entity, With<TicTacToeCamera>>,
    layers: Query<Entity, With<LayerComponent>>,
    ui_root: Query<Entity, With<TicTacToeUiRoot>>,
) {
    // Disable existing buttons
    for entity in &buttons {
        commands.entity(entity).remove::<Interaction>();
        commands.entity(entity).remove::<ButtonHover>();
    }

    // Delete layers
    for entity in &layers {
        commands.entity(entity).despawn_recursive();
    }

    let game = game.single();

    let camera = camera.single();

    sounds.send(PlaySoundEffect::new(match game.status {
        BoardStatus::InProgress => todo!(),
        BoardStatus::Draw => SoundEffect::Draw,
        BoardStatus::Win(info) => match info.side {
            Side::X => SoundEffect::Victory,
            Side::O => SoundEffect::Defeat,
        },
    }));

    create_parallax.send(CreateParallaxEvent {
        camera,
        layers_data: vec![LayerData {
            speed: LayerSpeed::Vertical(0.5),
            path: match game.status {
                BoardStatus::InProgress => panic!("Invalid game status"),
                BoardStatus::Draw => assets::TicTacToeAssets::DRAW_BACKGROUND.to_string(),
                BoardStatus::Win(info) => match info.side {
                    Side::X => assets::TicTacToeAssets::WIN_BACKGROUND.to_string(),
                    Side::O => assets::TicTacToeAssets::LOSE_BACKGROUND.to_string(),
                },
            },
            tile_size: UVec2::new(30, 30),
            cols: 1,
            rows: 1,
            scale: Vec2::splat(1.0),
            z: 0.0,
            ..Default::default()
        }],
    });

    let ui_root = ui_root.single();

    commands.entity(ui_root).with_children(|parent| {
        parent
            .spawn((NodeBundle {
                style: Style {
                    margin: UiRect::top(Val::Px(30.)),
                    width: Val::Px(150.),
                    height: Val::Px(65.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                background_color: BackgroundColor(Color::Srgba(
                    bevy::color::palettes::css::ANTIQUE_WHITE,
                )),
                border_color: BorderColor(Color::BLACK),
                ..default()
            },))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_sections(vec![TextSection::new(
                        "",
                        TextStyle {
                            font: fonts.main_font.clone(),
                            font_size: 60.,
                            color: Color::BLACK,
                        },
                    )]),
                    KeyText::new().with(
                        0,
                        match game.status {
                            BoardStatus::Draw => text_keys::DRAW,
                            BoardStatus::Win(info) => match info.side {
                                Side::X => text_keys::VICTORY,
                                Side::O => text_keys::DEFEAT,
                            },
                            _ => panic!("Invalid game status"),
                        },
                    ),
                ));
            });

        parent.spawn(MiniGameBackButton);
    });
}

#[derive(Component)]
struct TicTacToeUiRoot;

#[derive(Component)]
struct UiPetImage;

#[derive(Component)]
struct SquareImage;

#[derive(Component)]
struct BoardBackground;

fn update_board(
    game: Query<&Game>,
    mut square_buttons: Query<(&SquareButton, &mut BackgroundColor, &Children)>,
    mut square_images: Query<&mut TextureAtlas, With<SquareImage>>,
    mut board_border: Query<&mut BorderColor, With<BoardBackground>>,
) {
    let mut board_border = board_border.single_mut();

    let game = game.single();

    let win_board = match game.status {
        BoardStatus::Win(info) => info.win_board,
        _ => EMPTY,
    };

    *board_border = BorderColor(match game.status {
        BoardStatus::InProgress => Color::BLACK,
        BoardStatus::Draw => Color::Srgba(bevy::color::palettes::css::ORANGE),
        BoardStatus::Win(info) => match info.side {
            Side::X => Color::Srgba(bevy::color::palettes::css::DARK_GREEN),
            Side::O => Color::Srgba(bevy::color::palettes::css::TOMATO),
        },
    });

    for (button, mut background_color, children) in square_buttons.iter_mut() {
        let mut atlas = square_images.get_mut(children[0]).unwrap();

        if let Some(side) = game.board.get_square(button.0) {
            atlas.index = match side {
                Side::X => 2,
                Side::O => 1,
            };

            if *win_board & *BitBoard::from_square(button.0) != 0 {
                *background_color = BackgroundColor(match game.status {
                    BoardStatus::Win(info) => match info.side {
                        Side::X => Color::Srgba(bevy::color::palettes::css::LIMEGREEN),
                        Side::O => Color::Srgba(bevy::color::palettes::css::RED),
                    },
                    _ => Color::BLACK,
                });
            }
        } else {
            atlas.index = 0;
        }
    }
}

#[derive(Component, Deref)]
struct SquareButton(Square);

fn square_button_pressed(
    mut game_move: EventWriter<MakeMove>,
    buttons: Query<(&SquareButton, &Interaction), Changed<Interaction>>,
) {
    for (button, interaction) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        game_move.send(MakeMove(button.0));
    }
}

#[derive(Event)]
struct MakeMove(Square);

fn handle_move(
    mut commands: Commands,
    mut rng: ResMut<GlobalRng>,
    mut moves: EventReader<MakeMove>,
    mut game: Query<&mut Game>,
    mut tic_state: ResMut<NextState<TicTacToeState>>,
    mut sounds: EventWriter<PlaySoundEffect>,
    buttons: Query<(Entity, &SquareButton)>,
) {
    let mut game = game.single_mut();

    for game_move in moves.read() {
        game.make_move(game_move.0);

        sounds.send(PlaySoundEffect::new(SoundEffect::Place));

        match game.status {
            BoardStatus::InProgress => {}
            BoardStatus::Draw | BoardStatus::Win(_) => {
                tic_state.set(TicTacToeState::GameOver);
                return;
            }
        }

        // Make computer move
        let moves = if rng.i32(0..4) == 0 {
            game.board.possible_moves()
        } else {
            best_moves(&game.board)
        };

        game.make_move(moves[rng.usize(0..moves.len())]);

        match game.status {
            BoardStatus::InProgress => {}
            BoardStatus::Draw | BoardStatus::Win(_) => {
                tic_state.set(TicTacToeState::GameOver);
                return;
            }
        }
    }

    // Disable buttons which have been used
    for (entity, button) in &buttons {
        if game.board.get_square(button.0).is_some() {
            commands.entity(entity).remove::<Interaction>();
            commands.entity(entity).remove::<ButtonHover>();
        }
    }
}

fn update_pet_mood(
    mut mood: Query<&mut MoodCategory, With<UiPetImage>>,
    game: Query<&Game, Changed<Game>>,
) {
    let game: &Game = match game.get_single() {
        Ok(game) => game,
        Err(_) => return,
    };

    let worst = game
        .board
        .possible_moves()
        .iter()
        .map(|sq| -nega_max(&game.board.make_move_new(*sq), 1))
        .min()
        .unwrap_or(match game.status {
            BoardStatus::Draw => 0,
            BoardStatus::Win(info) => match info.side {
                Side::X => 2,
                Side::O => -2,
            },
            _ => 0,
        });

    let mut mood = mood.single_mut();

    *mood = match worst {
        2 => MoodCategory::Ecstatic,
        1 => MoodCategory::Happy,
        0 => MoodCategory::Neutral,
        -1 => MoodCategory::Sad,
        -2 => MoodCategory::Despairing,
        _ => panic!("Invalid worst"),
    };
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash, Deref, DerefMut)]
struct BitBoard(u16);

impl BitBoard {
    fn from_square(square: Square) -> Self {
        BitBoard(1 << *square)
    }
}

const EMPTY: BitBoard = BitBoard(0);
const WINS: [BitBoard; 8] = [
    BitBoard(0b111000000),
    BitBoard(0b000111000),
    BitBoard(0b000000111),
    BitBoard(0b100100100),
    BitBoard(0b010010010),
    BitBoard(0b001001001),
    BitBoard(0b100010001),
    BitBoard(0b001010100),
];

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
enum Side {
    X,
    O,
}

impl Side {
    fn to_index(self) -> usize {
        match self {
            Side::X => 0,
            Side::O => 1,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => Side::X,
            1 => Side::O,
            _ => panic!("Invalid index"),
        }
    }

    fn other(&self) -> Self {
        match self {
            Side::X => Side::O,
            Side::O => Side::X,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash, Deref)]
struct Square(u8);

impl Square {
    const A1: Square = Square(0);
    const B1: Square = Square(1);
    const C1: Square = Square(2);
    const A2: Square = Square(3);
    const B2: Square = Square(4);
    const C2: Square = Square(5);
    const A3: Square = Square(6);
    const B3: Square = Square(7);
    const C3: Square = Square(8);
}

const ALL_SQUARES: [Square; 9] = [
    Square::A1,
    Square::B1,
    Square::C1,
    Square::A2,
    Square::B2,
    Square::C2,
    Square::A3,
    Square::B3,
    Square::C3,
];

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
struct WinInfo {
    side: Side,
    win_board: BitBoard,
}

impl WinInfo {
    fn new(side: Side, win_board: BitBoard) -> Self {
        Self { side, win_board }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
enum BoardStatus {
    InProgress,
    Draw,
    Win(WinInfo),
}

#[derive(Copy, Clone)]
struct Board {
    size: u8,
    pieces: [BitBoard; 2],
    side_to_move: Side,
}

impl Board {
    fn new() -> Self {
        Self {
            pieces: [EMPTY, EMPTY],
            side_to_move: Side::X,
            size: 9,
        }
    }

    fn make_move_new(&self, game_move: Square) -> Self {
        let mut result = *self;

        let bb = result.pieces[result.side_to_move.to_index()];

        result.pieces[result.side_to_move.to_index()] = BitBoard(*bb | 1 << *game_move);

        result.side_to_move = result.side_to_move.other();

        result
    }

    fn possible_moves(&self) -> Vec<Square> {
        if self.win_position() {
            return Vec::new();
        }

        let mut moves = Vec::new();

        let mut filled = 0;
        for bb in &self.pieces {
            filled |= **bb;
        }

        for i in 0..self.size {
            if (filled >> i) & 1 == 0 {
                moves.push(Square(i));
            }
        }

        moves
    }

    fn win_position(&self) -> bool {
        for bb in self.pieces.iter() {
            for win in WINS {
                if **bb & *win == *win {
                    return true;
                }
            }
        }

        false
    }

    fn status(&self) -> BoardStatus {
        for (i, bb) in self.pieces.iter().enumerate() {
            for win in WINS {
                if **bb & *win == *win {
                    return BoardStatus::Win(WinInfo::new(Side::from_index(i), win));
                }
            }
        }

        if self.possible_moves().is_empty() {
            return BoardStatus::Draw;
        }

        BoardStatus::InProgress
    }

    fn get_square(&self, square: Square) -> Option<Side> {
        let sq_bb = BitBoard::from_square(square);

        for (i, bb) in self.pieces.iter().enumerate() {
            if **bb & *sq_bb == *sq_bb {
                return Some(Side::from_index(i));
            }
        }

        None
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();

        for i in 0..self.size {
            let side = match self.get_square(Square(i)) {
                Some(Side::X) => "X",
                Some(Side::O) => "O",
                None => " ",
            };

            result.push_str(side);

            if i % 3 != 2 {
                result.push_str(" | ");
            } else if i != self.size - 1 {
                result.push_str("\n---------\n");
            }
        }

        write!(f, "{}", result)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component)]
struct Game {
    board: Board,
    status: BoardStatus,
}

impl Game {
    fn new() -> Self {
        Self {
            board: Board::new(),
            status: BoardStatus::InProgress,
        }
    }

    fn make_move(&mut self, game_move: Square) {
        self.board = self.board.make_move_new(game_move);
        self.status = self.board.status();
    }
}

fn best_moves(board: &Board) -> Vec<Square> {
    let moves = board.possible_moves();

    let mut best_moves = Vec::new();
    let mut best_rating = i32::MIN;
    for square in moves.iter() {
        let new_board = board.make_move_new(*square);
        let score = -nega_max(&new_board, 9);
        match score.cmp(&best_rating) {
            std::cmp::Ordering::Greater => {
                best_moves.clear();
                best_rating = score;
                best_moves.push(*square);
            }
            std::cmp::Ordering::Equal => {
                best_moves.push(*square);
            }
            _ => {}
        }
    }

    best_moves
}

fn nega_max(board: &Board, depth: i32) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let mut max = i32::MIN;
    let squares = board.possible_moves();
    if squares.is_empty() {
        return evaluate(board);
    }
    for square in squares {
        let new_board = board.make_move_new(square);
        let score = -nega_max(&new_board, depth - 1);
        if score > max {
            max = score;
        }
    }

    max
}

fn evaluate(board: &Board) -> i32 {
    match board.status() {
        BoardStatus::InProgress | BoardStatus::Draw => 0,
        BoardStatus::Win(info) => {
            if info.side == board.side_to_move {
                1
            } else {
                -1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_possible_moves_blank_board() {
        let board: Board = Board::default();

        let moves = board.possible_moves();

        assert_eq!(moves.len(), 9);
    }

    #[test]
    fn test_possible_moves_xox_x_xo_o() {
        let mut board: Board = Board::default();

        board.pieces[Side::X.to_index()] = BitBoard(0b101010000);
        board.pieces[Side::O.to_index()] = BitBoard(0b010000101);

        let moves = board.possible_moves();

        assert_eq!(moves.len(), 3);
    }

    #[test]
    fn test_win_x() {
        let mut board: Board = Board::default();

        board.pieces[Side::X.to_index()] = BitBoard(0b111000000);
        board.pieces[Side::O.to_index()] = BitBoard(0b000101010);

        assert_eq!(
            board.status(),
            BoardStatus::Win(WinInfo::new(Side::X, BitBoard(0b111000000)))
        );
    }

    #[test]
    fn test_win_o() {
        let mut board: Board = Board::default();

        board.pieces[Side::X.to_index()] = BitBoard(0b000101010);
        board.pieces[Side::O.to_index()] = BitBoard(0b111000000);

        assert_eq!(
            board.status(),
            BoardStatus::Win(WinInfo::new(Side::O, BitBoard(0b111000000)))
        );
    }

    #[test]
    fn test_make_move() {
        let board: Board = Board::default();

        let updated_board = board.make_move_new(Square::A1);

        assert_eq!(
            updated_board.pieces[Side::X.to_index()],
            BitBoard(0b000000001)
        );
    }

    #[test]
    fn test_game_stalemate() {
        let mut game: Game = Game::new();

        const MOVES: [Square; 9] = [
            Square::A1,
            Square::B2,
            Square::C1,
            Square::B1,
            Square::B3,
            Square::A2,
            Square::C2,
            Square::C3,
            Square::A3,
        ];

        for (i, m) in MOVES.iter().enumerate() {
            game.make_move(*m);

            if i < 8 {
                assert_eq!(game.board.status(), BoardStatus::InProgress);
            }
        }

        assert_eq!(game.board.status(), BoardStatus::Draw);
    }

    #[test]
    fn test_get_square() {
        let mut board: Board = Board::default();

        board.pieces[Side::X.to_index()] = BitBoard(0b000000001);
        board.pieces[Side::O.to_index()] = BitBoard(0b000000010);

        assert_eq!(board.get_square(Square::A1), Some(Side::X));
        assert_eq!(board.get_square(Square::B1), Some(Side::O));
    }

    #[test]
    fn test_best_moves_is_draw() {
        let mut game: Game = Game::new();

        while game.status == BoardStatus::InProgress {
            let moves = best_moves(&game.board);
            let best_move = moves[0];
            game.make_move(best_move);
        }

        assert_eq!(game.status, BoardStatus::Draw);
    }
}
