use bevy::prelude::*;
use strum_macros::EnumIter;

use crate::palettes;
use sardips_core::{assets::FontAssets, text_translation::KeyText, GameState};
use text_keys::FOOD_BUY_SCENE_TITLE;

pub struct FoodBuyScenePlugin;

impl Plugin for FoodBuyScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(FoodBuySceneState::default())
            .add_systems(OnEnter(GameState::FoodBuy), setup_state)
            .add_systems(
                OnEnter(FoodBuySceneState::Selecting),
                (setup_camera, setup_ui),
            )
            .add_systems(
                Update,
                (tick_input).run_if(in_state(FoodBuySceneState::Selecting)),
            )
            .add_systems(OnExit(GameState::FoodBuy), cleanup);
    }
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum FoodBuySceneState {
    #[default]
    None,
    Selecting,
}

fn setup_state(mut state: ResMut<NextState<FoodBuySceneState>>) {
    state.set(FoodBuySceneState::Selecting);
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
        FoodBuySceneCamera,
        FoodBuyScene,
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
            FoodBuyScene,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: fonts.main_font.clone(),
                        font_size: 50.0,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, FOOD_BUY_SCENE_TITLE),
            ));
        });
}

fn cleanup(
    mut commands: Commands,
    entities: Query<Entity, With<FoodBuyScene>>,
    mut state: ResMut<NextState<FoodBuySceneState>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    state.set(FoodBuySceneState::None);
}

#[derive(Component)]
struct FoodBuySceneCamera;

#[derive(Component)]
struct FoodBuyScene;

#[derive(Component, EnumIter, Copy, Clone, PartialEq, Eq, Hash)]
enum FoodBuySceneButton {}

fn tick_input(query: Query<(&Interaction, &FoodBuySceneButton), Changed<Interaction>>) {
    for (interaction, _) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
    }
}
