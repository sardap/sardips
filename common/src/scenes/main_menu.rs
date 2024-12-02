use bevy::prelude::*;
use shared_deps::bevy_parallax::{
    CreateParallaxEvent, LayerComponent, LayerData, LayerSpeed, ParallaxCameraComponent,
};

use crate::{
    assets::{self, FontAssets},
    autoscroll::AutoScroll,
    button_hover::ButtonHover,
    palettes,
    text_translation::KeyText,
    GameState,
};

use text_keys;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), (setup_ui, setup_background));
        app.add_systems(OnExit(GameState::MainMenu), teardown);

        app.add_systems(Update, (play_button).run_if(in_state(GameState::MainMenu)));
    }
}

#[derive(Component)]
struct MainMenuCamera;

fn setup_background(mut commands: Commands, mut create_parallax: EventWriter<CreateParallaxEvent>) {
    let camera: Entity = commands
        .spawn((
            Camera2dBundle::default(),
            ParallaxCameraComponent::default(),
            AutoScroll::new(Vec2::new(150.0, 150.0)),
            MainMenuCamera,
        ))
        .id();

    create_parallax.send(CreateParallaxEvent {
        camera,
        layers_data: vec![LayerData {
            speed: LayerSpeed::Bidirectional(0.9, 0.9),
            path: assets::BackgroundTexturesAssets::MENU_BACKGROUND.to_string(),
            tile_size: UVec2::new(53, 53),
            cols: 1,
            rows: 1,
            scale: Vec2::splat(1.0),
            z: 0.0,
            ..Default::default()
        }],
    });
}

#[derive(Component)]
struct MainMenuUiItem;

#[derive(Component)]
struct PlayButton;

fn setup_ui(mut commands: Commands, fonts: Res<FontAssets>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    top: Val::Percent(0.),
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                ..default()
            },
            MainMenuUiItem,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: fonts.main_font.clone(),
                        font_size: 70.0,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, text_keys::MAIN_MENU_TITLE),
                Label,
            ));

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(300.),
                            height: Val::Px(70.),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(100.)),
                            border: UiRect::all(Val::Px(5.)),
                            ..default()
                        },
                        ..default()
                    },
                    ButtonHover::default()
                        .with_background(palettes::ui::BUTTON_SET)
                        .with_border(palettes::ui::BUTTON_BORDER_SET),
                    PlayButton,
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
                        KeyText::new().with(0, text_keys::MAIN_MENU_PLAY_BUTTON),
                    ));
                });
        });

    commands.spawn((
        TextBundle::from_section(
            format!("v{}", crate::VERSION),
            TextStyle {
                font: fonts.main_font.clone(),
                font_size: 50.0,
                color: bevy::color::palettes::css::DARK_GREEN.into(),
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.),
            left: Val::Px(10.),
            ..default()
        }),
        MainMenuUiItem,
    ));
}

fn teardown(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<MainMenuCamera>,
            With<MainMenuUiItem>,
            With<LayerComponent>,
        )>,
    >,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn play_button(
    mut game_state: ResMut<NextState<GameState>>,
    button: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
) {
    let button = button.get_single();
    if let Ok(Interaction::Pressed) = button {
        game_state.set(GameState::LoadViewScreen);
    }
}
