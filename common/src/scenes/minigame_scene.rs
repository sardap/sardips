use bevy::prelude::*;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    assets::FontAssets,
    button_hover::ButtonHover,
    minigames::MiniGameState,
    palettes,
    text_translation::{KeyString, KeyText},
    GameState,
};

use text_keys::{
    MINIGAME_SELECT_ENDLESS_RHYTHM, MINIGAME_SELECT_ENDLESS_SHOOTER, MINIGAME_SELECT_FOUR_IN_ROW,
    MINIGAME_SELECT_HIGHER_LOWER, MINIGAME_SELECT_SPRINT, MINIGAME_SELECT_TIC_TAC_TOE,
    MINIGAME_SELECT_TRANSLATE,
};

pub struct MinigameScenePlugin;

impl Plugin for MinigameScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MiniGame), setup_state)
            .add_systems(
                OnEnter(MiniGameState::Selecting),
                (setup_select_camera, setup_select_ui),
            )
            .add_systems(OnExit(MiniGameState::Selecting), cleanup_select)
            .add_systems(
                Update,
                (tick_input_selecting).run_if(in_state(MiniGameState::Selecting)),
            );
    }
}

fn setup_state(mut minigame_state: ResMut<NextState<MiniGameState>>) {
    minigame_state.set(MiniGameState::Selecting);
}

fn setup_select_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(palettes::minigame_select::BACKGROUND),
                ..default()
            },
            ..default()
        },
        MiniGameSelectCamera,
        MiniGameSelect,
    ));
}

fn setup_select_ui(mut commands: Commands, fonts: Res<FontAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            MiniGameSelect,
        ))
        .with_children(|parent| {
            for button_kind in MinigameButton::iter() {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(200.0),
                                height: Val::Px(100.0),
                                border: UiRect::all(Val::Px(5.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::new(
                                    Val::Px(0.),
                                    Val::Px(0.),
                                    Val::Px(5.),
                                    Val::Px(5.),
                                ),
                                ..default()
                            },
                            ..default()
                        },
                        ButtonHover::default()
                            .with_background(palettes::minigame_select::BUTTON_SET)
                            .with_border(palettes::minigame_select::BUTTON_BORDER_SET),
                        button_kind,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font: fonts.main_font.clone(),
                                    font_size: 40.0,
                                    color: Color::BLACK,
                                },
                            ),
                            KeyText {
                                keys: hashmap! { 0 => KeyString::Direct(match button_kind {
                                    MinigameButton::TicTacToe => MINIGAME_SELECT_TIC_TAC_TOE,
                                    MinigameButton::Sprint => MINIGAME_SELECT_SPRINT,
                                    MinigameButton::HigherLower => MINIGAME_SELECT_HIGHER_LOWER,
                                    MinigameButton::FourInRow => MINIGAME_SELECT_FOUR_IN_ROW,
                                    MinigameButton::EndlessShooter => MINIGAME_SELECT_ENDLESS_SHOOTER,
                                    MinigameButton::Rhythm => MINIGAME_SELECT_ENDLESS_RHYTHM,
                                    MinigameButton::Translate => MINIGAME_SELECT_TRANSLATE,
                                }.to_string()) },
                            },
                        ));
                    });
            }
        });
}

fn cleanup_select(mut commands: Commands, entities: Query<Entity, With<MiniGameSelect>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct MiniGameSelectCamera;

#[derive(Component)]
struct MiniGameSelect;

#[derive(Component, EnumIter, Copy, Clone, PartialEq, Eq, Hash)]
enum MinigameButton {
    TicTacToe,
    Sprint,
    HigherLower,
    FourInRow,
    EndlessShooter,
    Rhythm,
    Translate,
}

fn tick_input_selecting(
    mut minigame_state: ResMut<NextState<MiniGameState>>,
    query: Query<(&Interaction, &MinigameButton), Changed<Interaction>>,
) {
    for (interaction, button_kind) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        minigame_state.set(match *button_kind {
            MinigameButton::TicTacToe => MiniGameState::PlayingTicTakToe,
            MinigameButton::Sprint => MiniGameState::PlayingSprint,
            MinigameButton::HigherLower => MiniGameState::PlayingHigherLower,
            MinigameButton::FourInRow => MiniGameState::PlayingFourInRow,
            MinigameButton::EndlessShooter => MiniGameState::PlayingEndlessShooter,
            MinigameButton::Rhythm => MiniGameState::PlayingRhythm,
            MinigameButton::Translate => MiniGameState::PlayingTranslate,
        });
    }
}
