use bevy::prelude::*;
use sardips_core::{
    button_hover::{ButtonHover, Selected},
    name::NameTag,
    view::EntityView,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    food::view::FoodView,
    money::Wallet,
    palettes,
    pet::view::PetView,
    player::Player,
    simulation::{SimulationState, SimulationViewState},
    tools::poop_scooper::{create_poop_scooper, PoopScooper},
};
use sardips_core::{
    assets::{FontAssets, GameImageAssets, ViewScreenImageAssets},
    despawn_all,
    interaction::{Hovering, MouseCamera},
    GameState,
};

use super::info_panel::{create_info_panel, InfoPanelUpdate, InfoPanelsClear, PanelType};

pub struct ViewScreenPlugin;

impl Plugin for ViewScreenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state::<VSSubState>(VSSubState::default());
        app.add_systems(
            OnEnter(GameState::ViewScreen),
            (setup_ui, setup_camera, setup_state, setup_selector_trackers),
        );
        app.add_systems(OnExit(GameState::ViewScreen), teardown);
        app.add_systems(
            Update,
            (menu_button_interaction, update_money_text).run_if(in_state(GameState::ViewScreen)),
        );
        app.add_systems(
            Update,
            (info_panel_handle_click, toggle_interactions)
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
    Stocks,
    BuyAccessory,
    Options,
}

impl MenuOption {
    pub fn get_texture_index(&self) -> usize {
        match self {
            MenuOption::None => 0,
            MenuOption::Tools => 1,
            MenuOption::Food => 2,
            MenuOption::MiniGames => 3,
            MenuOption::Dipdex => 4,
            MenuOption::Stocks => 5,
            MenuOption::Options => 0,
            MenuOption::BuyAccessory => 0,
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
    let ui_background_camera: Entity = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 0,
                    ..default()
                },
                ..default()
            },
            ViewScreenUi,
        ))
        .id();

    commands
        .spawn((
            TargetCamera(ui_background_camera),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },

                ..default()
            },
            ViewScreenUi,
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                style: Style {
                    // width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                image: UiImage::new(view_screen_images.background.clone())
                    .with_color(Color::srgba(1., 1., 1., 0.3)),
                ..default()
            });
        });

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
            let percent = 90.0 / MenuOption::iter().len() as f32;

            for option in MenuOption::iter() {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                justify_self: JustifySelf::Center,
                                width: Val::Vw(percent),
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
                                index: option.get_texture_index(),
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

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                clear_color: ClearColorConfig::Custom(palettes::view_screen::BACKGROUND),
                ..default()
            },
            ..default()
        },
        ViewScreenCamera,
        MouseCamera,
    ));
}

fn setup_state(
    mut sim_state: ResMut<NextState<SimulationState>>,
    mut sim_view_state: ResMut<NextState<SimulationViewState>>,
) {
    sim_state.set(SimulationState::Running);
    sim_view_state.set(SimulationViewState::Visible);
}

fn setup_selector_trackers(mut commands: Commands, pet_selected: Query<Entity, With<SelectedPet>>) {
    if pet_selected.iter().len() == 0 {
        commands.spawn(SelectedPet { entity: None });
    }
}

fn teardown(
    mut commands: Commands,
    mut sim_view_state: ResMut<NextState<SimulationViewState>>,
    mut sim_state: ResMut<NextState<SimulationState>>,
    query: Query<Entity, Or<(With<ViewScreenUi>, With<ViewScreenCamera>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    sim_state.set(SimulationState::Paused);
    sim_view_state.set(SimulationViewState::Invisible);
}

fn info_panel_handle_click(
    mut selected_pet: Query<&mut SelectedPet>,
    mut update_info_panel: EventWriter<InfoPanelUpdate>,
    mut info_panel_clear: EventWriter<InfoPanelsClear>,
    buttons: Res<ButtonInput<MouseButton>>,
    pets: Query<&EntityView, (With<PetView>, With<Hovering>)>,
    foods: Query<&EntityView, (With<FoodView>, With<Hovering>)>,
    mut name_tags: Query<&mut NameTag>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let mut selected_pet = selected_pet.single_mut();

        let mut clicked = false;

        for view in pets.iter() {
            update_info_panel.send(InfoPanelUpdate {
                entity: view.entity,
                panel_type: PanelType::Pet,
            });
            selected_pet.entity = Some(view.entity);
            info!("Selected pet: {:?}", view.entity);
            clicked = true;
        }

        for view in foods.iter() {
            update_info_panel.send(InfoPanelUpdate {
                entity: view.entity,
                panel_type: PanelType::Food,
            });
            clicked = true;
        }

        // Should be clearing selection but Need to figure out how to consume clicks
        // So when clicking on the button it doesn't run this function
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
            MenuOption::Stocks => {
                vs_state.set(VSSubState::None);
                game_state.set(GameState::StockBuy);
            }
            MenuOption::Options => {
                vs_state.set(VSSubState::None);
            }
            MenuOption::BuyAccessory => {
                vs_state.set(VSSubState::None);
                game_state.set(GameState::BuyAccessory);
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

#[derive(Component)]
pub struct SelectedPet {
    entity: Option<Entity>,
}

impl SelectedPet {
    pub fn get_entity(&self) -> Option<Entity> {
        self.entity
    }
}

fn toggle_interactions(
    mut commands: Commands,
    pets: Query<&EntityView, With<PetView>>,
    menu_options: Query<(Entity, &MenuOption, Option<&Interaction>)>,
) {
    let have_pet = pets.iter().len() > 0;

    for (entity, option, interaction) in &menu_options {
        if *option == MenuOption::MiniGames {
            if have_pet && interaction.is_none() {
                commands.entity(entity).insert(Interaction::None);
            } else if !have_pet && interaction.is_some() {
                commands.entity(entity).remove::<Interaction>();
            }
        }
    }
}
