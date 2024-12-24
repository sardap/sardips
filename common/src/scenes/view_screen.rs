use bevy::prelude::*;
use shared_deps::strum_macros::EnumIter;
use strum::IntoEnumIterator;

use crate::{
    assets::{FontAssets, GameImageAssets, ViewScreenImageAssets},
    button_hover::{ButtonHover, Selected},
    despawn_all,
    food::view::FoodView,
    interaction::{Hovering, MouseCamera},
    money::Wallet,
    name::NameTag,
    palettes,
    pet::view::PetView,
    player::Player,
    tools::poop_scooper::{create_poop_scooper, PoopScooper},
    view::EntityView,
    GameState, SimulationState,
};

use super::info_panel::{
    create_info_panel, InfoPanelPlugin, InfoPanelUpdate, InfoPanelsClear, PanelType,
};

pub struct ViewScreenPlugin;

impl Plugin for ViewScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InfoPanelPlugin);
        app.insert_state::<VSSubState>(VSSubState::default());
        app.add_systems(
            OnEnter(GameState::ViewScreen),
            (setup_ui, setup_background, setup_state),
        );
        app.add_systems(OnExit(GameState::ViewScreen), teardown);
        app.add_systems(
            Update,
            (menu_button_interaction, update_money_text).run_if(in_state(GameState::ViewScreen)),
        );
        app.add_systems(
            Update,
            (info_panel_handle_click)
                .run_if(in_state(GameState::ViewScreen).and_then(in_state(VSSubState::None))),
        );
        app.add_systems(
            OnEnter(VSSubState::ToolPoopScooper),
            setup_tool_poop_scooper,
        );
        app.add_systems(
            OnExit(VSSubState::ToolPoopScooper),
            despawn_all::<PoopScooper>,
        );
    }
}

#[derive(Debug, States, Component, EnumIter, Copy, Clone, PartialEq, Eq, Hash, Default)]
enum VSSubState {
    #[default]
    None,
    ToolPoopScooper,
}

#[derive(Debug, Component, EnumIter, Copy, Clone, PartialEq, Eq, Hash, Default)]
enum MenuOption {
    #[default]
    None,
    Food,
    Tools,
    MiniGames,
    Dipdex,
    Options,
}

impl MenuOption {
    pub fn get_index(&self) -> usize {
        match self {
            MenuOption::None => 0,
            MenuOption::Tools => 1,
            MenuOption::Food => 2,
            MenuOption::MiniGames => 3,
            MenuOption::Dipdex => 4,
            MenuOption::Options => 0,
        }
    }
}

#[derive(Component)]
struct ViewScreenUi;

fn setup_ui(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    view_screen_images: Res<ViewScreenImageAssets>,
) {
    // Top UI
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                top: Val::Px(0.),
                width: Val::Vw(100.),
                height: Val::Px(50.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: Val::Px(100.),
                            height: Val::Px(60.),
                            border: UiRect::new(Val::Px(2.), Val::Px(2.), Val::Px(0.), Val::Px(2.)),
                            ..default()
                        },
                        background_color: BackgroundColor(palettes::view_screen::TOP_UI),
                        border_color: BorderColor(palettes::view_screen::TOP_UI_BORDER),
                        ..default()
                    },
                    ViewScreenUi,
                ))
                .with_children(|parent| {
                    // Money
                    parent
                        .spawn(NodeBundle { ..default() })
                        .with_children(|parent| {
                            parent.spawn((
                                ImageBundle {
                                    style: Style {
                                        width: Val::Px(30.),
                                        height: Val::Px(30.),
                                        margin: UiRect::right(Val::Px(3.)),
                                        ..default()
                                    },
                                    image: UiImage::new(view_screen_images.top_icons.clone()),
                                    ..default()
                                },
                                TextureAtlas {
                                    layout: view_screen_images.top_icons_layout.clone(),
                                    index: 0,
                                },
                            ));

                            parent.spawn((
                                TextBundle::from_section(
                                    "0",
                                    TextStyle {
                                        font_size: 40.,
                                        color: Color::BLACK,
                                        font: fonts.main_font.clone(),
                                    },
                                ),
                                PlayerMoneyText,
                            ));
                        });
                });
        });

    // Bottom buttons
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    bottom: Val::Px(0.),
                    width: Val::Vw(100.),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    ..default()
                },
                ..default()
            },
            ViewScreenUi,
        ))
        .with_children(|parent| {
            // Render menu options
            for option in MenuOption::iter() {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                justify_self: JustifySelf::Center,
                                width: Val::Px(70.),
                                height: Val::Px(70.),
                                border: UiRect::all(Val::Px(4.)),
                                ..default()
                            },
                            ..default()
                        },
                        option,
                        ButtonHover::default()
                            .with_border(palettes::view_screen::BUTTON_BORDER_SET),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            ImageBundle {
                                style: Style {
                                    width: Val::Percent(100.),
                                    height: Val::Percent(100.),
                                    ..default()
                                },
                                image: UiImage::new(view_screen_images.view_buttons.clone()),
                                ..default()
                            },
                            TextureAtlas {
                                layout: view_screen_images.view_buttons_layout.clone(),
                                index: option.get_index(),
                            },
                        ));
                    });
            }
        });

    // Render info panel
    let entity = create_info_panel(&mut commands, &fonts, &view_screen_images);
    commands.entity(entity).insert(ViewScreenUi);
}

#[derive(Component)]
pub struct ViewScreenCamera;

fn setup_background(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(palettes::view_screen::BACKGROUND),
                ..default()
            },
            ..default()
        },
        ViewScreenCamera,
        MouseCamera,
    ));
}

fn setup_state(mut sim_state: ResMut<NextState<SimulationState>>) {
    sim_state.set(SimulationState::Running);
}

fn teardown(
    mut commands: Commands,
    mut sim_state: ResMut<NextState<SimulationState>>,
    query: Query<Entity, Or<(With<ViewScreenUi>, With<ViewScreenCamera>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    sim_state.set(SimulationState::Paused);
}

fn info_panel_handle_click(
    mut update_info_panel: EventWriter<InfoPanelUpdate>,
    mut info_panel_clear: EventWriter<InfoPanelsClear>,
    buttons: Res<ButtonInput<MouseButton>>,
    pets: Query<&EntityView, (With<PetView>, With<Hovering>)>,
    foods: Query<&EntityView, (With<FoodView>, With<Hovering>)>,
    mut name_tags: Query<&mut NameTag>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let mut clicked = false;

        for view in pets.iter() {
            update_info_panel.send(InfoPanelUpdate {
                entity: view.entity,
                panel_type: PanelType::Pet,
            });
            clicked = true;
        }

        for view in foods.iter() {
            update_info_panel.send(InfoPanelUpdate {
                entity: view.entity,
                panel_type: PanelType::Food,
            });
            clicked = true;
        }

        if !clicked {
            info_panel_clear.send(InfoPanelsClear);
        }

        // Clear all other texts
        for mut text in name_tags.iter_mut() {
            text.color = Color::BLACK;
        }
    }
}

fn menu_button_interaction(
    mut commands: Commands,
    mut vs_state: ResMut<NextState<VSSubState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<(Entity, &Interaction, &MenuOption)>,
) {
    let mut skip = None;

    for (entity, interaction, option) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        skip = Some(entity);

        match option {
            MenuOption::None => {
                vs_state.set(VSSubState::None);
            }
            MenuOption::Food => {
                vs_state.set(VSSubState::None);
                game_state.set(GameState::FoodBuy);
            }
            MenuOption::Tools => {
                vs_state.set(VSSubState::ToolPoopScooper);
            }
            MenuOption::MiniGames => {
                vs_state.set(VSSubState::None);
                game_state.set(GameState::MiniGame);
            }
            MenuOption::Dipdex => {
                vs_state.set(VSSubState::None);
                game_state.set(GameState::DipdexView);
            }
            MenuOption::Options => {
                vs_state.set(VSSubState::None);
            }
        }

        break;
    }

    if let Some(skip_entity) = skip {
        for (entity, _, _) in interaction_query.iter_mut() {
            if entity != skip_entity {
                commands.entity(entity).remove::<Selected>();
            } else {
                commands.entity(entity).insert(Selected);
            }
        }
    }
}

fn setup_tool_poop_scooper(mut commands: Commands, game_image_assets: Res<GameImageAssets>) {
    create_poop_scooper(&mut commands, &game_image_assets);
}

#[derive(Component)]
struct PlayerMoneyText;

fn update_money_text(
    wallet: Query<&Wallet, (With<Player>, Changed<Wallet>)>,
    mut text: Query<&mut Text, With<PlayerMoneyText>>,
) {
    let wallet = match wallet.get_single() {
        Ok(wallet) => wallet,
        Err(_) => return,
    };

    let mut text = text.single_mut();

    text.sections[0].value = wallet.to_string();
}
