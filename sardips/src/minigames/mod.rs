// Checkers
// Bug catch
// Battleship
// Candy Crush clone

use bevy::prelude::*;
use sardips_core::{
    assets::FontAssets,
    button_hover::ButtonHover,
    fun_core::Fun,
    minigames_core::{
        MiniGameBackButton, MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType, Playing,
    },
    money_core::Money,
    mood_core::MoodImageIndexes,
    text_translation::KeyText,
    view::HasView,
    GameState,
};

use crate::{
    money::Wallet,
    palettes,
    pet::{
        fun::{MinigamePreference, MinigamePreferences},
        view::PetView,
        Pet,
    },
    player::Player,
};
use text_keys;

pub struct MinigamePlugin;

impl Plugin for MinigamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MiniGameState::None), remove_playing)
            .add_systems(OnEnter(MiniGameState::Selecting), set_minigame_pet)
            .add_systems(Update, (mini_game_completed, handle_back))
            .add_systems(
                Update,
                spawn_back_button.run_if(resource_exists::<FontAssets>),
            );
    }
}

struct MiniGamePrize {
    pub money: Money,
    pub fun: f32,
}

impl MiniGamePrize {
    pub fn new(money: Money, fun: f32) -> Self {
        Self { money, fun }
    }

    fn from_result(game_type: &MiniGameType, result: MiniGameResult) -> Option<MiniGamePrize> {
        if result == MiniGameResult::Incomplete {
            return None;
        }

        Some(match game_type {
            MiniGameType::TicTacToe => match result {
                MiniGameResult::Win => MiniGamePrize::new(600, 5.),
                MiniGameResult::Draw => MiniGamePrize::new(200, 1.),
                MiniGameResult::Lose => MiniGamePrize::new(100, 1.),
                _ => return None,
            },
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
            MiniGameType::RectClash => match result {
                MiniGameResult::Win => MiniGamePrize::new(1000, 10.),
                MiniGameResult::Draw => MiniGamePrize::new(50, 1.),
                MiniGameResult::Lose => MiniGamePrize::new(50, 1.),
                _ => return None,
            },
            MiniGameType::Snake => match result {
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
        let prize = MiniGamePrize::from_result(&event.game_type, event.result).unwrap();

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

fn spawn_back_button(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    new_buttons: Query<Entity, Added<MiniGameBackButton>>,
) {
    for entity in new_buttons.iter() {
        commands.entity(entity).insert((
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
            MiniGameBackButton,
            ButtonHover::default()
                .with_background(palettes::minigame_select::BUTTON_SET)
                .with_border(palettes::minigame_select::BUTTON_BORDER_SET),
        ));

        commands
            .spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: fonts.main_font.clone(),
                        font_size: 40.,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, text_keys::BACK),
            ))
            .set_parent(entity);
    }
}

fn handle_back(
    mut mini_game_state: ResMut<NextState<MiniGameState>>,
    button: Query<&Interaction, (Changed<Interaction>, With<MiniGameBackButton>)>,
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
