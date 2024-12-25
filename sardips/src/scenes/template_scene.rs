use bevy::prelude::*;
use shared_deps::strum_macros::EnumIter;

use crate::palettes;
use sardips_core::{
    assets::FontAssets,
    button_hover::ButtonHover,
    text_translation::{KeyString, KeyText},
    GameState,
};
use text_keys::MINIGAME_SELECT_FOUR_IN_ROW;

pub struct TemplateScenePlugin;

impl Plugin for TemplateScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(TemplateSceneState::default())
            .add_systems(OnEnter(GameState::Template), setup_state)
            .add_systems(
                OnEnter(TemplateSceneState::Selecting),
                (setup_camera, setup_ui),
            )
            .add_systems(
                Update,
                (tick_input).run_if(in_state(TemplateSceneState::Selecting)),
            )
            .add_systems(OnExit(GameState::Template), cleanup);
    }
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum TemplateSceneState {
    #[default]
    None,
    Selecting,
}

fn setup_state(mut minigame_state: ResMut<NextState<TemplateSceneState>>) {
    minigame_state.set(TemplateSceneState::Selecting);
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(palettes::minigame_select::BACKGROUND),
                ..default()
            },
            ..default()
        },
        TemplateSceneCamera,
        TemplateScene,
    ));
}

fn setup_ui(mut commands: Commands, fonts: Res<FontAssets>) {
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
            TemplateScene,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(100.0),
                            border: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::new(Val::Px(0.), Val::Px(0.), Val::Px(5.), Val::Px(5.)),
                            ..default()
                        },
                        ..default()
                    },
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
                                font_size: 40.0,
                                color: Color::BLACK,
                            },
                        ),
                        KeyText {
                            keys: hashmap! { 0 => KeyString::Direct(MINIGAME_SELECT_FOUR_IN_ROW.to_string()) },
                        },
                    ));
                });
        });
}

fn cleanup(
    mut commands: Commands,
    entities: Query<Entity, With<TemplateScene>>,
    mut state: ResMut<NextState<TemplateSceneState>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    state.set(TemplateSceneState::None);
}

#[derive(Component)]
struct TemplateSceneCamera;

#[derive(Component)]
struct TemplateScene;

#[derive(Component, EnumIter, Copy, Clone, PartialEq, Eq, Hash)]
enum TemplateSceneButton {}

fn tick_input(query: Query<(&Interaction, &TemplateSceneButton), Changed<Interaction>>) {
    for (interaction, _) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
    }
}
