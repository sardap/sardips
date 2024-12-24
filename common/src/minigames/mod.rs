pub mod rhythm_template;
mod sprint;
pub mod translate_wordbank;
// Checkers
// Snake
// Bug catch
// Battleship
// Candy Crush clone

use bevy::prelude::*;
use rhythm_template::RhythmTemplatePlugin;
use translate_wordbank::TranslateWordBankPlugin;

use crate::{
    assets::FontAssets,
    button_hover::ButtonHover,
    money::{Money, Wallet},
    palettes,
    pet::{
        fun::{Fun, MinigamePreference, MinigamePreferences},
        mood::MoodImageIndexes,
        view::PetView,
        Pet,
    },
    player::Player,
    text_translation::KeyText,
    view::HasView,
    GameState,
};
use text_keys;

use self::sprint::SprintPlugin;

pub struct MinigamePlugin;

impl Plugin for MinigamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state::<MiniGameState>(MiniGameState::default())
            .add_event::<MiniGameCompleted>()
            .add_plugins((SprintPlugin, RhythmTemplatePlugin, TranslateWordBankPlugin))
            .add_systems(OnEnter(MiniGameState::None), remove_playing)
            .add_systems(OnEnter(MiniGameState::Selecting), set_minigame_pet)
            .add_systems(Update, (mini_game_completed, handle_back));
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MiniGameResult {
    Incomplete,
    Win,
    Lose,
    Draw,
}

#[derive(Event)]
pub struct MiniGameCompleted {
    pub game_type: MiniGameType,
    pub result: MiniGameResult,
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum MiniGameState {
    #[default]
    None,
    Selecting,
    PlayingTicTakToe,
    PlayingSprint,
    PlayingHigherLower,
    PlayingFourInRow,
    PlayingEndlessShooter,
    PlayingRhythm,
    PlayingTranslate,
}

struct MiniGamePrize {
    pub money: Money,
    pub fun: f32,
}

impl MiniGamePrize {
    pub fn new(money: Money, fun: f32) -> Self {
        Self { money, fun }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MiniGameType {
    TicTacToe,
    Sprint,
    HigherLower,
    FourInRow,
    EndlessShooter,
    Rhythm,
    Translate,
}

impl MiniGameType {
    fn prize(&self, result: MiniGameResult) -> Option<MiniGamePrize> {
        if result == MiniGameResult::Incomplete {
            return None;
        }

        Some(match self {
            MiniGameType::TicTacToe => match result {
                MiniGameResult::Win => MiniGamePrize::new(600, 5.),
                MiniGameResult::Draw => MiniGamePrize::new(200, 1.),
                MiniGameResult::Lose => MiniGamePrize::new(100, 1.),
                _ => return None,
            },
            MiniGameType::Sprint => todo!(),
            MiniGameType::HigherLower => match result {
                MiniGameResult::Win => MiniGamePrize::new(1000, 10.),
                MiniGameResult::Draw => MiniGamePrize::new(50, 1.),
                MiniGameResult::Lose => MiniGamePrize::new(50, 1.),
                _ => return None,
            },
            MiniGameType::FourInRow => match result {
                MiniGameResult::Win => MiniGamePrize::new(800, 20.),
                MiniGameResult::Draw => MiniGamePrize::new(20000, 5.),
                MiniGameResult::Lose => MiniGamePrize::new(500, 1.),
                _ => return None,
            },
            MiniGameType::EndlessShooter => match result {
                MiniGameResult::Win => MiniGamePrize::new(1000, 10.),
                MiniGameResult::Draw => MiniGamePrize::new(50, 1.),
                MiniGameResult::Lose => MiniGamePrize::new(50, 1.),
                _ => return None,
            },
            MiniGameType::Rhythm => match result {
                MiniGameResult::Win => MiniGamePrize::new(1000, 10.),
                MiniGameResult::Draw => MiniGamePrize::new(50, 1.),
                MiniGameResult::Lose => MiniGamePrize::new(50, 1.),
                _ => return None,
            },
            MiniGameType::Translate => match result {
                MiniGameResult::Win => MiniGamePrize::new(1000, 10.),
                MiniGameResult::Draw => MiniGamePrize::new(50, 1.),
                MiniGameResult::Lose => MiniGamePrize::new(50, 1.),
                _ => return None,
            },
        })
    }
}

fn mini_game_completed(
    mut mini_game_state: ResMut<NextState<MiniGameState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut game_completed: EventReader<MiniGameCompleted>,
    mut playing: Query<(&mut Fun, &MinigamePreferences), With<Playing>>,
    mut player_wallet: Query<&mut Wallet, With<Player>>,
) {
    for event in game_completed.read() {
        let prize = event.game_type.prize(event.result).unwrap();

        if let Ok((mut fun, preferences)) = playing.get_single_mut() {
            let preference: &MinigamePreference = preferences
                .0
                .get(&event.game_type)
                .unwrap_or(&MinigamePreference::Neutral);

            let fun_score = prize.fun * preference.fun_modifier();

            fun.add(fun_score);
        }

        if let Ok(mut wallet) = player_wallet.get_single_mut() {
            wallet.balance += prize.money;
        }

        mini_game_state.set(MiniGameState::None);
        game_state.set(GameState::ViewScreen);
    }
}

#[derive(Component)]
struct SelectedPet;

#[derive(Component)]
pub struct Playing;

fn set_minigame_pet(
    mut commands: Commands,
    pet: Query<(Entity, &HasView), With<Pet>>,
    pet_view: Query<(&Handle<Image>, &TextureAtlas, &MoodImageIndexes, &Sprite), With<PetView>>,
    currently_playing: Query<Entity, With<Playing>>,
) {
    for entity in currently_playing.iter() {
        commands.entity(entity).remove::<Playing>();
    }

    if let Some((entity, has_view)) = pet.iter().next() {
        commands.entity(entity).insert(SelectedPet);
        if let Ok((image, atlas, mood, sprite)) = pet_view.get(has_view.view_entity) {
            commands.spawn((Playing, image.clone(), atlas.clone(), *mood, sprite.clone()));
        }
    }
}

fn remove_playing(mut commands: Commands, pet: Query<Entity, With<Playing>>) {
    for entity in pet.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct BackButton;

#[derive(Bundle)]
pub struct MiniGameBackExitButton {
    pub button: ButtonBundle,
    pub back_button: BackButton,
    pub button_hover: ButtonHover,
}

impl MiniGameBackExitButton {
    pub fn spawn(parent: &mut ChildBuilder, fonts: &FontAssets) {
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        margin: UiRect::top(Val::Px(30.)),
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        border: UiRect::all(Val::Px(5.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                },
                BackButton,
                ButtonHover::default()
                    .with_background(palettes::minigame_select::BUTTON_SET)
                    .with_border(palettes::minigame_select::BUTTON_BORDER_SET),
            ))
            .with_children(|parent| {
                parent.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font: fonts.main_font.clone(),
                            font_size: 40.,
                            color: Color::BLACK,
                        },
                    ),
                    KeyText::new().with(0, text_keys::BACK),
                ));
            });
    }
}

fn handle_back(
    mut mini_game_state: ResMut<NextState<MiniGameState>>,
    button: Query<&Interaction, (Changed<Interaction>, With<BackButton>)>,
) {
    let interaction = match button.get_single() {
        Ok(interaction) => interaction,
        Err(_) => return,
    };

    if *interaction != Interaction::Pressed {
        return;
    }

    mini_game_state.set(MiniGameState::Selecting);
}
