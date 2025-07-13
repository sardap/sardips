use bevy::prelude::*;
use strum_macros::EnumIter;

use crate::{
    accessory::Accessory,
    inventory::Inventory,
    money::Wallet,
    palettes,
    pet::Pet,
    pet_display::{spawn_pet_preview, PetPreview},
    player::Player,
};
use sardips_core::{
    accessory_core::{AccessoryDiscoveredEntries, AccessoryTemplateDatabase},
    assets::{DipdexImageAssets, FontAssets},
    button_hover::{ButtonColorSet, ButtonHover},
    name::SpeciesName,
    rotate_static::RotateStatic,
    text_translation::KeyText,
    ui_utils::spawn_back_button,
    GameState,
};
use text_keys::{FOOD_BUY_SCENE_COST_LABEL, FOOD_BUY_SCENE_QTY_LABEL, FOOD_BUY_SCENE_TITLE};

pub struct BuyAccessoryScenePlugin;

impl Plugin for BuyAccessoryScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(BuyAccessorySceneState::default())
            .add_systems(OnEnter(GameState::BuyAccessory), setup_state)
            .add_systems(
                OnEnter(BuyAccessorySceneState::Selecting),
                (setup_camera, setup_ui, setup_selection),
            )
            .add_systems(
                Update,
                (
                    tick_input,
                    exit_accessory,
                    select_interaction,
                    update_qty_label,
                    active_selection_changed,
                )
                    .run_if(in_state(BuyAccessorySceneState::Selecting)),
            )
            .add_systems(OnExit(GameState::BuyAccessory), cleanup);
    }
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum BuyAccessorySceneState {
    #[default]
    None,
    Selecting,
}

fn setup_state(mut minigame_state: ResMut<NextState<BuyAccessorySceneState>>) {
    minigame_state.set(BuyAccessorySceneState::Selecting);
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
        BuyAccessoryScene,
    ));
}

const FUNDS_SIZE: f32 = 25.;
const COST_SIZE: f32 = 20.;

fn setup_selection(mut commands: Commands) {
    commands.spawn((ActiveSelection::default(), BuyAccessoryScene));
}

fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    dipdex_assets: Res<DipdexImageAssets>,
    font_assets: Res<FontAssets>,
    accessory_db: Res<AccessoryTemplateDatabase>,
    player: Query<(&AccessoryDiscoveredEntries, &Wallet), With<Player>>,
    active_pets: Query<&SpeciesName, With<Pet>>,
) {
    let (discovered, wallet) = player.single();
    let pet_template_name = active_pets.iter().next().unwrap().0.clone();

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
            BuyAccessoryScene,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.main_font.clone(),
                        font_size: 50.0,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, FOOD_BUY_SCENE_TITLE),
            ));

            // FUNDS
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.main_font.clone(),
                        font_size: FUNDS_SIZE,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with_value(
                    0,
                    text_keys::FOOD_BUY_SCENE_MONEY_LABEL,
                    &[wallet.balance.to_string().as_str()],
                ),
            ));

            let mut things: Vec<_> = discovered
                .entries
                .iter()
                .map(|name| accessory_db.get(name).unwrap())
                .collect();
            things.sort_by(|a, b| a.texture.cmp(&b.texture));

            const ROW_COUNT: usize = 4;
            let mut current_row_count = 0;
            let mut current_row = parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            });

            for template in things {
                current_row.with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100. / ROW_COUNT as f32),
                                padding: UiRect::all(Val::Px(10.)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_sections(vec![TextSection::new(
                                    "",
                                    TextStyle {
                                        font_size: COST_SIZE,
                                        color: Color::BLACK,
                                        font: font_assets.main_font.clone(),
                                    },
                                )]),
                                KeyText::new().with_value(
                                    0,
                                    FOOD_BUY_SCENE_COST_LABEL,
                                    &[template.cost.to_string().as_str()],
                                ),
                            ));

                            parent
                                .spawn((
                                    ButtonBundle {
                                        style: Style {
                                            width: Val::Px(60.),
                                            height: Val::Px(60.),
                                            margin: UiRect::new(
                                                Val::Px(5.),
                                                Val::Px(5.),
                                                Val::Px(0.),
                                                Val::Px(0.),
                                            ),
                                            border: UiRect::all(Val::Px(2.)),
                                            justify_content: JustifyContent::Center,
                                            align_content: AlignContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        border_color: BorderColor(Color::BLACK),
                                        background_color: BackgroundColor(Color::WHITE),
                                        ..default()
                                    },
                                    ButtonHover::default()
                                        .with_background(ButtonColorSet {
                                            normal: Color::srgba(0.9, 0.9, 0.9, 1.),
                                            hover: Color::srgba(0.3, 0.9, 0.3, 1.),
                                            pressed: Color::WHITE,
                                            disabled: Color::WHITE,
                                        })
                                        .with_border(ButtonColorSet {
                                            normal: Color::BLACK,
                                            hover: Color::BLACK,
                                            pressed: Color::BLACK,
                                            disabled: Color::BLACK,
                                        }),
                                    SelectButton {
                                        name: template.name.clone(),
                                    },
                                ))
                                .with_children(|parent| {
                                    let custom_size = template.texture_size;

                                    let (w, h) = if custom_size.x > custom_size.y {
                                        (Val::Percent(60.), Val::Auto)
                                    } else {
                                        (Val::Auto, Val::Percent(60.))
                                    };
                                    parent.spawn((ImageBundle {
                                        style: Style {
                                            width: w,
                                            height: h,
                                            ..default()
                                        },
                                        image: UiImage::new(asset_server.load(&template.texture)),
                                        ..default()
                                    },));

                                    // Spawn overlay
                                    parent.spawn((
                                        ImageBundle {
                                            style: Style {
                                                position_type: PositionType::Absolute,
                                                width: Val::Percent(90.),
                                                height: Val::Percent(90.),
                                                ..default()
                                            },
                                            image: UiImage::new(dipdex_assets.screen_noise.clone())
                                                .with_color(Color::srgba(1., 1., 1., 0.1)),
                                            ..default()
                                        },
                                        TextureAtlas {
                                            layout: dipdex_assets.screen_noise_layout.clone(),
                                            index: 0,
                                        },
                                        RotateStatic::default(),
                                    ));
                                });

                            parent.spawn((
                                TextBundle::from_sections(vec![TextSection::new(
                                    "",
                                    TextStyle {
                                        font_size: COST_SIZE,
                                        color: Color::BLACK,
                                        font: font_assets.main_font.clone(),
                                    },
                                )]),
                                KeyText::new().with_value(0, FOOD_BUY_SCENE_QTY_LABEL, &["0"]),
                                QtyLabel(template.name.clone()),
                            ));
                        });
                });

                current_row_count += 1;
                if current_row_count >= ROW_COUNT {
                    current_row_count = 0;
                    current_row = parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,

                            ..default()
                        },
                        ..default()
                    });
                }
            }

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(100.),
                        height: Val::Px(100.),
                        padding: UiRect::all(Val::Px(10.)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(palettes::PASTEL_GREEN),
                    ..default()
                })
                .with_children(|parent| {
                    // Spawn pet image
                    spawn_pet_preview(
                        parent,
                        PetPreview::new(pet_template_name).with_max_size(100.),
                    )
                    .insert(DressUpView);
                    // Spawn overlay
                    parent.spawn((
                        ImageBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(90.),
                                height: Val::Percent(90.),
                                ..default()
                            },
                            image: UiImage::new(dipdex_assets.screen_noise.clone())
                                .with_color(Color::srgba(1., 1., 1., 0.05)),
                            ..default()
                        },
                        TextureAtlas {
                            layout: dipdex_assets.screen_noise_layout.clone(),
                            index: 0,
                        },
                        RotateStatic::default(),
                    ));
                });

            spawn_back_button::<ExitBuyAccessory>(
                parent,
                &font_assets,
                &palettes::ui::BUTTON_SET,
                &palettes::ui::BUTTON_BORDER_SET,
            );
        });
}

#[derive(Component)]
struct QtyLabel(String);

fn update_qty_label(
    changed_inventory: Query<Entity, (With<Player>, Changed<Inventory>)>,
    label_new: Query<Entity, Added<QtyLabel>>,
    mut labels: Query<(&mut KeyText, &QtyLabel)>,
    inventory: Query<&Inventory, With<Player>>,
) {
    if changed_inventory.iter().count() == 0 && label_new.iter().count() == 0 {
        return;
    }

    let inventory = inventory.single();

    // A real fuck it section
    let accessory: Vec<_> = inventory.get_accessories().collect();

    for (mut text, qty) in &mut labels {
        let count = accessory.iter().filter(|i| i.template == qty.0).count();

        text.replace_value(0, 0, count.to_string());
    }
}

#[derive(Component, Default)]
struct ExitBuyAccessory;

fn exit_accessory(
    mut game_state: ResMut<NextState<GameState>>,
    mut buy_state: ResMut<NextState<BuyAccessorySceneState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<ExitBuyAccessory>)>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            game_state.set(GameState::ViewScreen);
            buy_state.set(BuyAccessorySceneState::None);
        }
    }
}

#[derive(Component)]
struct SelectButton {
    name: String,
}

#[derive(Component, Default)]
struct ActiveSelection {
    pub selected: Option<String>,
}

fn select_interaction(
    select_buttons: Query<(&Interaction, &SelectButton), Changed<Interaction>>,
    mut selection: Query<&mut ActiveSelection>,
) {
    let mut selection = selection.single_mut();

    for (interaction, button) in &select_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        selection.selected = Some(button.name.clone());
    }
}

fn active_selection_changed(
    selection: Query<&ActiveSelection, Changed<ActiveSelection>>,
    mut display: Query<&mut PetPreview>,
) {
    let selection = match selection.get_single() {
        Ok(x) => x,
        Err(_) => return,
    };

    let mut display = display.single_mut();

    match &selection.selected {
        Some(name) => {
            display.replace_accessory(Accessory::new(name));
        }
        None => {
            display.clear_accessory();
        }
    }
}

fn cleanup(
    mut commands: Commands,
    entities: Query<Entity, With<BuyAccessoryScene>>,
    mut state: ResMut<NextState<BuyAccessorySceneState>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    state.set(BuyAccessorySceneState::None);
}

#[derive(Component)]
struct TemplateSceneCamera;

#[derive(Component)]
struct BuyAccessoryScene;

#[derive(Component, EnumIter, Copy, Clone, PartialEq, Eq, Hash)]
enum TemplateSceneButton {}

fn tick_input(query: Query<(&Interaction, &TemplateSceneButton), Changed<Interaction>>) {
    for (interaction, _) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
    }
}

#[derive(Component)]
struct DressUpView;

#[derive(Component)]
struct BuyButton;

fn buy_button_interaction(
    select_buttons: Query<(&Interaction, &SelectButton), Changed<Interaction>>,
    mut selection: Query<&mut ActiveSelection>,
) {
    let mut selection = selection.single_mut();

    for (interaction, button) in &select_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        selection.selected = Some(button.name.clone());
    }
}

// TODO make a little dress up thing when you select it shows it on the guy and maybe you can add spewers
// Maybe add a seprate section for spwpewers
