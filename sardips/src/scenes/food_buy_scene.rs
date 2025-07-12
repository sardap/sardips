use bevy::prelude::*;
use strum_macros::EnumIter;

use crate::{
    food::{FoodDiscoveredEntries, SpawnFoodEvent},
    money::Wallet,
    palettes,
    player::Player,
};
use sardips_core::{
    assets::{DipdexImageAssets, FontAssets, ViewScreenImageAssets},
    button_hover::{ButtonColorSet, ButtonHover},
    food_core::FoodTemplateDatabase,
    rotate_static::RotateStatic,
    sounds::{PlaySoundEffect, SoundEffect},
    text_translation::KeyText,
    ui_utils::spawn_back_button,
    GameState,
};
use text_keys::{FOOD_BUY_SCENE_COST_LABEL, FOOD_BUY_SCENE_MONEY_LABEL, FOOD_BUY_SCENE_TITLE};

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
                (tick_input, exit_dipdex, buy_interaction)
                    .run_if(in_state(FoodBuySceneState::Selecting)),
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

const FUNDS_SIZE: f32 = 25.;
const COST_SIZE: f32 = 20.;

fn setup_ui(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    asset_server: Res<AssetServer>,
    dipdex_assets: Res<DipdexImageAssets>,
    view_assets: Res<ViewScreenImageAssets>,
    font_assets: Res<FontAssets>,
    food_db: Res<FoodTemplateDatabase>,
    player: Query<(&FoodDiscoveredEntries, &Wallet), With<Player>>,
) {
    let (discovered_food, wallet) = player.single();

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

            // FUNDS
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: fonts.main_font.clone(),
                        font_size: FUNDS_SIZE,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with_value(
                    0,
                    FOOD_BUY_SCENE_MONEY_LABEL,
                    &[wallet.balance.to_string().as_str()],
                ),
            ));

            let mut food: Vec<_> = discovered_food
                .entries
                .iter()
                .map(|name| food_db.get(name).unwrap())
                .collect();
            food.sort_by(|a, b| a.name.cmp(&b.name));

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

            for template in food {
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
                                    BuyButton {
                                        name: template.name.clone(),
                                    },
                                ))
                                .with_children(|parent| {
                                    let custom_size =
                                        template.sprite_size.vec2(template.texture_size);

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

                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        width: Val::Percent(100.),
                                        flex_direction: FlexDirection::Row,
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    let mut sensations =
                                        template.sensations.iter().collect::<Vec<_>>();
                                    sensations.sort();
                                    for sensation in &sensations {
                                        parent.spawn((
                                            ImageBundle {
                                                style: Style {
                                                    width: Val::Px(26.4),
                                                    height: Val::Px(36.),
                                                    margin: UiRect::all(Val::Px(2.)),
                                                    ..default()
                                                },
                                                image: UiImage::new(
                                                    view_assets.food_sensation.clone(),
                                                ),
                                                ..default()
                                            },
                                            TextureAtlas {
                                                layout: view_assets.food_sensation_layout.clone(),
                                                index: sensation.icon_index(),
                                            },
                                        ));
                                    }
                                });
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

            spawn_back_button::<ExitBuyFood>(
                parent,
                &fonts,
                &palettes::ui::BUTTON_SET,
                &palettes::ui::BUTTON_BORDER_SET,
            );
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
enum FoodBuySceneButton {
    Quit,
}

fn tick_input(query: Query<(&Interaction, &FoodBuySceneButton), Changed<Interaction>>) {
    for (interaction, _) in query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
    }
}

#[derive(Component, Default)]
struct ExitBuyFood;

fn exit_dipdex(
    mut game_state: ResMut<NextState<GameState>>,
    mut buy_state: ResMut<NextState<FoodBuySceneState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<ExitBuyFood>)>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            game_state.set(GameState::ViewScreen);
            buy_state.set(FoodBuySceneState::None);
        }
    }
}

#[derive(Component)]
struct BuyButton {
    name: String,
}

fn buy_interaction(
    food_db: Res<FoodTemplateDatabase>,
    mut sounds: EventWriter<PlaySoundEffect>,
    mut spawn_food: EventWriter<SpawnFoodEvent>,
    mut wallet: Query<&mut Wallet, With<Player>>,
    buy_buttons: Query<(&Interaction, &BuyButton), Changed<Interaction>>,
) {
    let mut wallet = wallet.single_mut();

    for (interaction, button) in buy_buttons.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let template = food_db.get(&button.name).unwrap();
        if template.cost > wallet.balance {
            sounds.send(PlaySoundEffect::new(SoundEffect::Error));
            continue;
        }
        wallet.balance -= template.cost;
        info!("Buying {}", template.name);

        // Figure out food spawn I can just spawn a food entity right?
        spawn_food.send(SpawnFoodEvent::new(&template.name));
    }
}
