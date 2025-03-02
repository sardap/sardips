use bevy::prelude::*;
use shared_deps::bevy_turborand::{DelegatedRng, GenCore, GlobalRng};
use strum_macros::EnumIter;

use crate::{
    food::SpawnFoodEvent,
    money::{money_aberration_decimal_display, money_aberration_display, money_display, Wallet},
    palettes,
    player::{self, Player},
    stock_market::{
        BuyOrderBrief, Company, CompanyPerformance, CompanyRank, OrderBook, ShareHistory,
        SharePortfolio,
    },
};
use sardips_core::{
    assets::{DipdexImageAssets, FontAssets, ViewScreenImageAssets},
    button_hover::ButtonHover,
    despawn_all,
    food_core::FoodTemplateDatabase,
    money_core::Money,
    rgb_to_color,
    sounds::{PlaySoundEffect, SoundEffect},
    text_translation::{warp_recursive_value_key, KeyText},
    ui_utils::spawn_back_button,
    GameState,
};
use text_keys::{
    STOCK_BUY_SCENE_BUY_EXISTING_BUY_LINE, STOCK_BUY_SCENE_BUY_PLAYER_OPEN_OPEN_TITLE,
    STOCK_BUY_SCENE_EXPAND, STOCK_BUY_SCENE_FEATURE_BUY_BUTTON, STOCK_BUY_SCENE_FEATURE_BUY_OPEN,
    STOCK_BUY_SCENE_FEATURE_BUY_OPEN_NONE, STOCK_BUY_SCENE_FEATURE_EARNINGS,
    STOCK_BUY_SCENE_FEATURE_INDUSTRY_HEADER, STOCK_BUY_SCENE_FEATURE_INDUSTRY_PERCENT,
    STOCK_BUY_SCENE_FEATURE_MARKET_CAP, STOCK_BUY_SCENE_FEATURE_NET_ASSETS,
    STOCK_BUY_SCENE_FEATURE_ONE_Q_CHANGE, STOCK_BUY_SCENE_FEATURE_ONE_YEAR_CHANGE,
    STOCK_BUY_SCENE_FEATURE_PB_RATIO, STOCK_BUY_SCENE_FEATURE_PEG_RATIO,
    STOCK_BUY_SCENE_FEATURE_PERCENTILE, STOCK_BUY_SCENE_FEATURE_PE_RATIO,
    STOCK_BUY_SCENE_FEATURE_REVENUE, STOCK_BUY_SCENE_FEATURE_SELL_BUTTON,
    STOCK_BUY_SCENE_FEATURE_SELL_OPEN, STOCK_BUY_SCENE_FEATURE_SELL_OPEN_NONE,
    STOCK_BUY_SCENE_FEATURE_STOCK_PRICE, STOCK_BUY_SCENE_MARKET_CAP_HEADER,
    STOCK_BUY_SCENE_ONE_Q_CHANGE_HEADER, STOCK_BUY_SCENE_OWN_HEADER, STOCK_BUY_SCENE_STOCK_PRICE,
    STOCK_BUY_SCENE_STOCK_PRICE_HEADER, STOCK_BUY_SCENE_TICKER_HEADER, STOCK_BUY_SCENE_TITLE,
};

pub struct StockScenePlugin;

impl Plugin for StockScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(StockBuySceneState::default())
            .add_systems(
                OnEnter(GameState::StockBuy),
                (setup_camera, setup_state, setup_selecting_entity),
            )
            .add_systems(
                OnEnter(StockBuySceneState::SelectingCompany),
                setup_selecting_ui,
            )
            .add_systems(
                OnExit(StockBuySceneState::SelectingCompany),
                despawn_all::<Selecting>,
            )
            .add_systems(
                Update,
                (
                    tick_input,
                    exit_scene,
                    rotate_static,
                    buy_interaction,
                    expand_button_pressed,
                )
                    .run_if(in_state(StockBuySceneState::SelectingCompany)),
            )
            .add_systems(
                OnEnter(StockBuySceneState::FeatureCompany),
                setup_company_focus_screen,
            )
            .add_systems(
                OnExit(StockBuySceneState::FeatureCompany),
                despawn_all::<Feature>,
            )
            .add_systems(
                Update,
                (open_sell_button_interacted, open_buy_button_interacted)
                    .run_if(in_state(StockBuySceneState::FeatureCompany)),
            )
            .add_systems(
                Update,
                feature_back_pressed.run_if(in_state(StockBuySceneState::FeatureCompany)),
            )
            .add_systems(OnEnter(StockBuySceneState::Buy), setup_buy_screen)
            .add_systems(OnExit(StockBuySceneState::Buy), despawn_all::<BuyScreen>)
            .add_systems(
                Update,
                buy_sell_back_pressed.run_if(in_state(StockBuySceneState::Buy)),
            )
            .add_systems(OnExit(GameState::StockBuy), cleanup);
    }
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum StockBuySceneState {
    #[default]
    None,
    SelectingCompany,
    FeatureCompany,
    Sell,
    Buy,
}

fn setup_state(mut state: ResMut<NextState<StockBuySceneState>>) {
    state.set(StockBuySceneState::SelectingCompany);
}

fn setup_selecting_entity(mut commands: Commands, companies: Query<Entity, With<Company>>) {
    let company = companies.iter().next().unwrap();

    commands.spawn((StockScene, SelectedExpandedCompany(company)));
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
        StockSceneCamera,
        StockScene,
    ));
}

const GOOD_COLOR: Color = rgb_to_color!(22, 160, 133);
const BAD_COLOR: Color = rgb_to_color!(192, 57, 43);
const FOCUS_BACKGROUND_COLOR: Color = Color::srgba(0.7, 1.0, 0.7, 0.8);

#[derive(Component)]
struct Selecting;

fn setup_selecting_ui(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    font_assets: Res<FontAssets>,
    companies: Query<(Entity, &Company, &ShareHistory)>,
    player: Query<&SharePortfolio, With<Player>>,
) {
    let mut companies: Vec<_> = companies.iter().collect();
    companies.sort_by(|a, b| {
        let a_market_cap = a.1.existing_shares as i128 * a.2.cached_price as i128;
        let b_market_cap = b.1.existing_shares as i128 * b.2.cached_price as i128;
        b_market_cap.cmp(&a_market_cap)
    });

    let font = font_assets.monospace.clone();

    let share_portfolio = player.single();

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
            Selecting,
            StockScene,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font.clone(),
                        font_size: 50.0,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, STOCK_BUY_SCENE_TITLE),
            ));

            // Header row
            parent
                .spawn((NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    const ROW_TEXT_SIZE: f32 = 27.0;

                    // Ticker
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(70.0),
                                align_content: AlignContent::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },

                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, STOCK_BUY_SCENE_TICKER_HEADER),
                            ));
                        });

                    // Player Own
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(60.0),

                                ..default()
                            },

                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, STOCK_BUY_SCENE_OWN_HEADER),
                            ));
                        });

                    // Market cap
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(60.0),

                                ..default()
                            },

                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, STOCK_BUY_SCENE_MARKET_CAP_HEADER),
                            ));
                        });

                    // Stock price
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(90.0),

                                ..default()
                            },

                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, STOCK_BUY_SCENE_STOCK_PRICE_HEADER),
                            ));
                        });

                    // Change 1q
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Px(90.0),

                                ..default()
                            },

                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, STOCK_BUY_SCENE_ONE_Q_CHANGE_HEADER),
                            ));
                        });

                    // Expand (Need for spacing)
                    parent.spawn((NodeBundle {
                        style: Style {
                            width: Val::Px(70.0),
                            border: UiRect::all(Val::Px(2.0)),
                            margin: UiRect::all(Val::Px(5.0)),
                            ..default()
                        },
                        ..default()
                    },));
                });

            for (company_entity, company, share_history) in companies {
                parent
                    .spawn((NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(50.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(FOCUS_BACKGROUND_COLOR),
                        border_color: BorderColor(Color::BLACK),
                        ..default()
                    },))
                    .with_children(|parent| {
                        const ROW_TEXT_SIZE: f32 = 25.0;

                        // Ticker
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Val::Px(70.0),
                                    align_content: AlignContent::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },

                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn((TextBundle::from_section(
                                    &company.ticker,
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),));
                            });

                        // Player Own
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Val::Px(60.0),
                                    ..default()
                                },

                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn((TextBundle::from_section(
                                    money_aberration_display(
                                        share_portfolio.get_count(&company_entity) * 100,
                                    ),
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),));
                            });

                        // Market cap
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Val::Px(60.0),
                                    ..default()
                                },

                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "",
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: ROW_TEXT_SIZE,
                                            color: Color::BLACK,
                                        },
                                    ),
                                    KeyText::new().with_value(
                                        0,
                                        STOCK_BUY_SCENE_STOCK_PRICE,
                                        &[money_aberration_display(
                                            company.existing_shares as i128
                                                * share_history.cached_price as i128,
                                        )
                                        .as_str()],
                                    ),
                                ));
                            });

                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Val::Px(90.0),
                                    ..default()
                                },

                                ..default()
                            })
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "",
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: ROW_TEXT_SIZE,
                                            color: Color::BLACK,
                                        },
                                    ),
                                    KeyText::new().with_value(
                                        0,
                                        STOCK_BUY_SCENE_STOCK_PRICE,
                                        &[money_display(share_history.cached_price).as_str()],
                                    ),
                                ));
                            });

                        // Change 1q
                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Val::Px(90.0),
                                    ..default()
                                },

                                ..default()
                            })
                            .with_children(|parent| {
                                let last_performance = company.performance_history.last().unwrap();
                                let change =
                                    share_history.cached_price - last_performance.stock_price;
                                let (color, symbol) = if change > 0 {
                                    (GOOD_COLOR, "+")
                                } else if change < 0 {
                                    (BAD_COLOR, "")
                                } else {
                                    (Color::BLACK, "")
                                };
                                let percent_change =
                                    change as f32 / last_performance.stock_price as f32 * 100.0;

                                parent.spawn((TextBundle::from_section(
                                    format!("{}{:.2}%", symbol, percent_change),
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color: color,
                                    },
                                ),));
                            });

                        // Expand button
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(70.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        margin: UiRect::all(Val::Px(5.0)),
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    ..default()
                                },
                                ExpandButton(company_entity),
                                ButtonHover::default()
                                    .with_background(palettes::ui::BUTTON_SET)
                                    .with_border(palettes::ui::BUTTON_BORDER_SET),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "",
                                        TextStyle {
                                            font: font.clone(),
                                            font_size: ROW_TEXT_SIZE,
                                            color: Color::BLACK,
                                        },
                                    ),
                                    KeyText::new().with(0, STOCK_BUY_SCENE_EXPAND),
                                ));
                            });
                    });
            }

            spawn_back_button::<ExitScene>(
                parent,
                &fonts,
                &palettes::ui::BUTTON_SET,
                &palettes::ui::BUTTON_BORDER_SET,
            );
        });
}

fn cleanup(
    mut commands: Commands,
    entities: Query<Entity, With<StockScene>>,
    mut state: ResMut<NextState<StockBuySceneState>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }
    state.set(StockBuySceneState::None);
}

#[derive(Component)]
struct StockSceneCamera;

#[derive(Component)]
struct StockScene;

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
struct ExitScene;

fn exit_scene(
    mut game_state: ResMut<NextState<GameState>>,
    mut buy_state: ResMut<NextState<StockBuySceneState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<ExitScene>)>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            game_state.set(GameState::ViewScreen);
            buy_state.set(StockBuySceneState::None);
        }
    }
}

#[derive(Component)]
struct RotateStatic {
    timer: Timer,
}

impl Default for RotateStatic {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        }
    }
}

fn rotate_static(
    time: Res<Time>,
    mut rand: ResMut<GlobalRng>,
    mut rotate: Query<(&mut TextureAtlas, &mut RotateStatic)>,
) {
    let rand = rand.get_mut();

    for (mut layout, mut rotate) in rotate.iter_mut() {
        if rotate.timer.tick(time.delta()).just_finished() {
            layout.index = rand.gen_usize() % 64;
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

#[derive(Component)]
struct ExpandButton(Entity);

fn expand_button_pressed(
    mut state: ResMut<NextState<StockBuySceneState>>,
    buttons: Query<(&Interaction, &ExpandButton), Changed<Interaction>>,
    mut selected: Query<&mut SelectedExpandedCompany>,
) {
    let mut selected = selected.single_mut();

    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        selected.0 = button.0;
        state.set(StockBuySceneState::FeatureCompany);
    }
}

#[derive(Component)]
struct SelectedExpandedCompany(Entity);

#[derive(Component)]
struct Feature;

fn setup_company_focus_screen(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    order_book: Res<OrderBook>,
    player_portfolio: Query<&SharePortfolio, With<Player>>,
    selected: Query<&SelectedExpandedCompany>,
    companies: Query<(Entity, &Company, &ShareHistory)>,
) {
    let selected = selected.single().0;

    let ranking = CompanyRank::new_ranking(
        &companies
            .iter()
            .map(|(entity, company, share_history)| {
                (entity, CompanyPerformance::new(company, share_history))
            })
            .collect::<Vec<_>>(),
    );

    let player_portfolio = player_portfolio.single();

    let root = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(95.0),
                    height: Val::Percent(95.0),
                    padding: UiRect::all(Val::Px(3.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BackgroundColor(FOCUS_BACKGROUND_COLOR),
                border_color: BorderColor(Color::BLACK),
                ..default()
            },
            Feature,
            StockScene,
        ))
        .with_children(|parent| {
            let (_, company, share_history) = companies.get(selected).unwrap();

            let performance = CompanyPerformance::new(company, share_history);
            let last_history = company.history.last().unwrap().clone();
            let last_last_history = if company.performance_history.len() >= 2 {
                company.history[company.history.len() - 2].clone()
            } else {
                last_history.clone()
            };

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: 50.0,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, company.name_key()),
            ));

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: 27.0,
                        color: Color::BLACK,
                    },
                )
                .with_style(Style {
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    ..default()
                }),
                KeyText::new().with(0, company.description_key()),
            ));

            let divider = NodeBundle {
                style: Style {
                    height: Val::Px(30.0),
                    ..default()
                },
                ..default()
            };

            // industries
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: STATS_SIZE + 3.,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_INDUSTRY_HEADER),
            ));
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceAround,
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    for (percent, industry) in &company.industries {
                        parent.spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: STATS_SIZE,
                                    color: Color::BLACK,
                                },
                            ),
                            KeyText::new().with_value(
                                0,
                                STOCK_BUY_SCENE_FEATURE_INDUSTRY_PERCENT,
                                &[
                                    &warp_recursive_value_key(industry.name_key()),
                                    &format!("{:.0}", percent * 100.),
                                ],
                            ),
                        ));
                    }
                });

            parent.spawn(divider.clone());

            const STATS_SIZE: f32 = 25.0;

            // Price Stats
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceAround,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color: Color::BLACK,
                            },
                        ),
                        KeyText::new().with_value(
                            0,
                            STOCK_BUY_SCENE_FEATURE_MARKET_CAP,
                            &[
                                money_aberration_display(company.market_value(share_history))
                                    .as_str(),
                            ],
                        ),
                    ));

                    parent.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color: Color::BLACK,
                            },
                        ),
                        KeyText::new().with_value(
                            0,
                            STOCK_BUY_SCENE_FEATURE_STOCK_PRICE,
                            &[money_display(share_history.cached_price).as_str()],
                        ),
                    ));
                });

            // Change stats
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceAround,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle { ..default() })
                        .with_children(|parent| {
                            let (percent_change, color, symbol) = get_price_percent_change_set(
                                share_history,
                                company.performance_history.last().unwrap(),
                            );

                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font_assets.monospace.clone(),
                                        font_size: STATS_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_ONE_Q_CHANGE),
                            ));

                            parent.spawn((TextBundle::from_section(
                                format!("{}{:.2}%", symbol, percent_change),
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: STATS_SIZE,
                                    color,
                                },
                            ),));
                        });

                    if company.performance_history.len() < 4 {
                        return;
                    }

                    parent
                        .spawn(NodeBundle { ..default() })
                        .with_children(|parent| {
                            let (percent_change, color, symbol) = get_price_percent_change_set(
                                share_history,
                                &company.performance_history[company.performance_history.len() - 4],
                            );

                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font_assets.monospace.clone(),
                                        font_size: STATS_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_ONE_YEAR_CHANGE),
                            ));

                            parent.spawn((TextBundle::from_section(
                                format!("{}{:.2}%", symbol, percent_change),
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: STATS_SIZE,
                                    color,
                                },
                            ),));
                        });
                });

            parent.spawn(divider.clone());

            // Key figures
            {
                let (percent_change, color, symbol) =
                    get_percent_change_set(last_history.revenue, last_last_history.revenue);

                parent.spawn((
                    TextBundle::from_sections(vec![
                        TextSection::new(
                            "",
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color: Color::BLACK,
                            },
                        ),
                        TextSection::new(
                            money_aberration_decimal_display(last_history.revenue),
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color,
                            },
                        ),
                        TextSection::new(
                            format!(" {}{:.2}%", symbol, percent_change),
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color,
                            },
                        ),
                    ]),
                    KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_REVENUE),
                ));
            }

            {
                let last_earnings = last_history.revenue - last_history.expenses;
                let last_last_earnings = last_last_history.revenue - last_last_history.expenses;

                let (percent_change, color, symbol) =
                    get_percent_change_set(last_earnings, last_last_earnings);

                parent.spawn((
                    TextBundle::from_sections(vec![
                        TextSection::new(
                            "",
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color: Color::BLACK,
                            },
                        ),
                        TextSection::new(
                            money_aberration_decimal_display(last_earnings),
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color,
                            },
                        ),
                        TextSection::new(
                            format!(" {}{:.2}%", symbol, percent_change),
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color,
                            },
                        ),
                    ]),
                    KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_EARNINGS),
                ));
            }

            {
                let last_assets = last_history.assets;
                let last_last_assets = last_last_history.assets;

                let (percent_change, color, symbol) =
                    get_percent_change_set(last_assets, last_last_assets);

                parent.spawn((
                    TextBundle::from_sections(vec![
                        TextSection::new(
                            "",
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color: Color::BLACK,
                            },
                        ),
                        TextSection::new(
                            money_aberration_decimal_display(last_assets),
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color,
                            },
                        ),
                        TextSection::new(
                            format!(" {}{:.2}%", symbol, percent_change),
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: STATS_SIZE,
                                color,
                            },
                        ),
                    ]),
                    KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_NET_ASSETS),
                ));
            }

            parent.spawn(divider.clone());

            let rank = ranking.get(&selected).unwrap();

            parent.spawn((
                TextBundle::from_sections(vec![
                    TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    TextSection::new(
                        " ",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                ]),
                KeyText::new()
                    .with_value(
                        0,
                        STOCK_BUY_SCENE_FEATURE_PE_RATIO,
                        &[&format!("{:.2}", performance.pe_ratio)],
                    )
                    .with_value(
                        2,
                        STOCK_BUY_SCENE_FEATURE_PERCENTILE,
                        &[&format!("{:.0}", rank.pe_percentile * 100.)],
                    ),
            ));

            parent.spawn((
                TextBundle::from_sections(vec![
                    TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    TextSection::new(
                        " ",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                ]),
                KeyText::new()
                    .with_value(
                        0,
                        STOCK_BUY_SCENE_FEATURE_PB_RATIO,
                        &[&format!("{:.2}", performance.pb_ratio)],
                    )
                    .with_value(
                        2,
                        STOCK_BUY_SCENE_FEATURE_PERCENTILE,
                        &[&format!("{:.0}", rank.pb_percentile * 100.)],
                    ),
            ));

            parent.spawn((
                TextBundle::from_sections(vec![
                    TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    TextSection::new(
                        " ",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: STATS_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                ]),
                KeyText::new()
                    .with_value(
                        0,
                        STOCK_BUY_SCENE_FEATURE_PEG_RATIO,
                        &[&{
                            match performance.peg_ratio {
                                Some(peg) => format!("{:.2}", peg),
                                None => "N/A".to_string(),
                            }
                        }],
                    )
                    .with_value(
                        2,
                        STOCK_BUY_SCENE_FEATURE_PERCENTILE,
                        &[&format!("{:.0}", rank.peg_percentile * 100.)],
                    ),
            ));

            parent.spawn(divider.clone());

            struct OrderBrief {
                price: Money,
                quantity: u64,
            }

            let top_buy_order = {
                let mut found: Option<OrderBrief> = None;
                for buy_order in &order_book.buy_orders {
                    if buy_order.company == selected {
                        match &mut found {
                            Some(prev) => {
                                if buy_order.price < prev.price {
                                    prev.price = buy_order.price;
                                    prev.quantity = buy_order.remaining_quantity;
                                } else if buy_order.price == prev.price {
                                    prev.quantity += buy_order.remaining_quantity;
                                }
                            }
                            None => {
                                found = Some(OrderBrief {
                                    price: buy_order.price,
                                    quantity: buy_order.remaining_quantity,
                                });
                            }
                        }
                    }
                }

                found
            };

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    if let Some(buy_order) = top_buy_order {
                        parent.spawn((
                            TextBundle::from_sections(vec![TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: STATS_SIZE,
                                    color: Color::BLACK,
                                },
                            )]),
                            KeyText::new().with_value(
                                0,
                                STOCK_BUY_SCENE_FEATURE_BUY_OPEN,
                                &[
                                    &money_display(buy_order.price),
                                    buy_order.quantity.to_string().as_str(),
                                ],
                            ),
                        ));
                    } else {
                        parent.spawn((
                            TextBundle::from_sections(vec![TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: STATS_SIZE,
                                    color: Color::BLACK,
                                },
                            )]),
                            KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_BUY_OPEN_NONE),
                        ));
                    }

                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(60.0),
                                    height: Val::Px(30.0),
                                    margin: UiRect::all(Val::Px(5.0)),
                                    align_content: AlignContent::Center,
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                ..default()
                            },
                            OpenBuyButton,
                            ButtonHover::default()
                                .with_background(palettes::ui::BUTTON_SET)
                                .with_border(palettes::ui::BUTTON_BORDER_SET),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font_assets.monospace.clone(),
                                        font_size: STATS_SIZE,
                                        color: Color::BLACK,
                                    },
                                )
                                .with_style(Style {
                                    align_content: AlignContent::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                }),
                                KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_BUY_BUTTON),
                            ));
                        });
                });

            // Sell order
            let top_sell_order = {
                let mut found: Option<OrderBrief> = None;
                let fuck = order_book.get_sell_orders(selected);
                for sell_order in fuck {
                    match &mut found {
                        Some(prev) => {
                            if sell_order.price != prev.price {
                                break;
                            }

                            prev.quantity += sell_order.remaining_quantity;
                        }
                        None => {
                            found = Some(OrderBrief {
                                price: sell_order.price,
                                quantity: sell_order.remaining_quantity,
                            });
                        }
                    }
                }

                found
            };

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    if let Some(sell_order) = top_sell_order {
                        parent.spawn((
                            TextBundle::from_sections(vec![TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: STATS_SIZE,
                                    color: Color::BLACK,
                                },
                            )]),
                            KeyText::new().with_value(
                                0,
                                STOCK_BUY_SCENE_FEATURE_SELL_OPEN,
                                &[
                                    &money_display(sell_order.price),
                                    sell_order.quantity.to_string().as_str(),
                                ],
                            ),
                        ));
                    } else {
                        parent.spawn((
                            TextBundle::from_sections(vec![TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: STATS_SIZE,
                                    color: Color::BLACK,
                                },
                            )]),
                            KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_SELL_OPEN_NONE),
                        ));
                    }

                    if player_portfolio.get_count(&selected) > 0 {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(60.0),
                                        height: Val::Px(30.0),
                                        margin: UiRect::all(Val::Px(5.0)),
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    ..default()
                                },
                                OpenSellButton,
                                ButtonHover::default()
                                    .with_background(palettes::ui::BUTTON_SET)
                                    .with_border(palettes::ui::BUTTON_BORDER_SET),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "",
                                        TextStyle {
                                            font: font_assets.monospace.clone(),
                                            font_size: STATS_SIZE,
                                            color: Color::BLACK,
                                        },
                                    )
                                    .with_style(Style {
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    }),
                                    KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_SELL_BUTTON),
                                ));
                            });
                    }
                });

            spawn_back_button::<FeatureBack>(
                parent,
                &font_assets,
                &palettes::ui::BUTTON_SET,
                &palettes::ui::BUTTON_BORDER_SET,
            );
        })
        .id();

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
            Feature,
            StockScene,
        ))
        .push_children(&[root]);
}

#[derive(Component, Default)]
struct FeatureBack;

fn feature_back_pressed(
    mut buy_state: ResMut<NextState<StockBuySceneState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<FeatureBack>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            buy_state.set(StockBuySceneState::SelectingCompany);
        }
    }
}

fn get_price_percent_change_set(
    share_history: &ShareHistory,
    performance: &CompanyPerformance,
) -> (f32, Color, &'static str) {
    get_percent_change_set(share_history.cached_price, performance.stock_price)
}

fn get_percent_change_set<T: Into<Money>, J: Into<Money>>(
    new_value: J,
    old_value: T,
) -> (f32, Color, &'static str) {
    let old_value: Money = old_value.into();
    let new_value: Money = new_value.into();
    let change = new_value - old_value;
    let (color, symbol) = if change > 0 {
        (GOOD_COLOR, "+")
    } else if change < 0 {
        (BAD_COLOR, "-")
    } else {
        (Color::srgb(0., 0., 0.), "")
    };
    let percent_change = change as f32 / old_value as f32 * 100.0;

    (percent_change.abs(), color, symbol)
}

#[derive(Component)]
struct OpenSellButton;

fn open_sell_button_interacted(
    mut buy_state: ResMut<NextState<StockBuySceneState>>,
    buttons: Query<&Interaction, (With<OpenSellButton>, Changed<Interaction>)>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            buy_state.set(StockBuySceneState::Sell);
        }
    }
}

#[derive(Component)]
struct OpenBuyButton;

fn open_buy_button_interacted(
    mut buy_state: ResMut<NextState<StockBuySceneState>>,
    buttons: Query<&Interaction, (With<OpenBuyButton>, Changed<Interaction>)>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            buy_state.set(StockBuySceneState::Buy);
        }
    }
}

#[derive(Component)]
struct BuyScreen;

fn setup_buy_screen(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    order_book: Res<OrderBook>,
    selected_company: Query<&SelectedExpandedCompany>,
    player: Query<(Entity, &Wallet, &SharePortfolio), With<Player>>,
    companies: Query<(&Company, &ShareHistory)>,
) {
    let selected = selected_company.single().0;

    let (player_entity, wallet, share_portfolio) = player.single();

    let (company, share_history) = companies.get(selected).unwrap();

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(90.0),
                    height: Val::Percent(90.0),
                    padding: UiRect::all(Val::Percent(5.0)),
                    margin: UiRect::all(Val::Percent(5.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: BackgroundColor(FOCUS_BACKGROUND_COLOR),
                border_color: BorderColor(Color::BLACK),
                ..default()
            },
            BuyScreen,
            StockScene,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: 50.0,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, company.name_key()),
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        padding: UiRect::new(
                            Val::Percent(1.0),
                            Val::Percent(1.0),
                            Val::Percent(5.0),
                            Val::Percent(5.0),
                        ),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    const TITLE_SIZE: f32 = 40.;
                    const BODY_SIZE: f32 = 30.;

                    // Buy Side
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(50.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgba(0., 1., 1., 0.5)),
                            ..default()
                        })
                        .with_children(|parent| {
                            const MAX_BUY_ORDERS_SHOWN: usize = 10;

                            let mut open_buy_orders: Vec<_> = order_book
                                .buy_orders
                                .iter()
                                .filter(|order| order.company == selected)
                                .collect();
                            open_buy_orders.sort_by(|a, b| a.price.cmp(&b.price));

                            let mut non_player_buy_orders: Vec<BuyOrderBrief> = vec![];
                            let mut player_buy_orders = vec![];
                            {
                                for buy_order in open_buy_orders {
                                    if buy_order.buyer == player_entity {
                                        player_buy_orders.push(buy_order)
                                    } else {
                                        if let Some(last) = non_player_buy_orders.iter_mut().last()
                                        {
                                            if last.price == buy_order.price {
                                                last.quantity += buy_order.remaining_quantity;
                                                continue;
                                            }
                                        }
                                        non_player_buy_orders.push(BuyOrderBrief::new(
                                            buy_order.quantity,
                                            buy_order.price,
                                        ));
                                    }
                                }
                            }

                            // Title
                            parent.spawn((
                                TextBundle::from_section(
                                    "",
                                    TextStyle {
                                        font: font_assets.monospace.clone(),
                                        font_size: TITLE_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, text_keys::STOCK_BUY_SCENE_BUY_BUY_TITLE),
                            ));

                            // Buy form
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        height: Val::Px(70.0),
                                        width: Val::Percent(100.0),
                                        ..default()
                                    },
                                    background_color: BackgroundColor(Color::srgba(
                                        0.5, 0., 0.5, 0.5,
                                    )),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    // HERE writing buy form
                                });

                            // Players Open orders
                            if player_buy_orders.len() > 0 {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "",
                                        TextStyle {
                                            font: font_assets.monospace.clone(),
                                            font_size: BODY_SIZE,
                                            color: Color::BLACK,
                                        },
                                    ),
                                    KeyText::new()
                                        .with(0, STOCK_BUY_SCENE_BUY_PLAYER_OPEN_OPEN_TITLE),
                                ));

                                for order in player_buy_orders.iter().take(MAX_BUY_ORDERS_SHOWN) {
                                    parent.spawn((
                                        TextBundle::from_sections(vec![TextSection::new(
                                            "",
                                            TextStyle {
                                                font: font_assets.monospace.clone(),
                                                font_size: BODY_SIZE,
                                                color: Color::BLACK,
                                            },
                                        )]),
                                        KeyText::new().with_value(
                                            0,
                                            STOCK_BUY_SCENE_BUY_EXISTING_BUY_LINE,
                                            &[
                                                &format!("{: >4}", order.remaining_quantity),
                                                &money_display(order.price),
                                            ],
                                        ),
                                    ));
                                }
                            }

                            // Non Player open orders
                            let non_player_orders_to_show = MAX_BUY_ORDERS_SHOWN
                                .checked_sub(player_buy_orders.len())
                                .unwrap_or(0);

                            if non_player_buy_orders.len() > 0 {
                                parent.spawn((
                                    TextBundle::from_section(
                                        "",
                                        TextStyle {
                                            font: font_assets.monospace.clone(),
                                            font_size: BODY_SIZE,
                                            color: Color::BLACK,
                                        },
                                    ),
                                    KeyText::new()
                                        .with(0, text_keys::STOCK_BUY_SCENE_BUY_EXISTING_BUY_TITLE),
                                ));

                                for order in
                                    non_player_buy_orders.iter().take(non_player_orders_to_show)
                                {
                                    parent.spawn((
                                        TextBundle::from_sections(vec![TextSection::new(
                                            "",
                                            TextStyle {
                                                font: font_assets.monospace.clone(),
                                                font_size: BODY_SIZE,
                                                color: Color::BLACK,
                                            },
                                        )]),
                                        KeyText::new().with_value(
                                            0,
                                            STOCK_BUY_SCENE_BUY_EXISTING_BUY_LINE,
                                            &[
                                                &format!("{: >4}", order.quantity),
                                                &money_display(order.price),
                                            ],
                                        ),
                                    ));
                                }
                            } else if non_player_orders_to_show > 0 {
                                parent.spawn((
                                    TextBundle::from_sections(vec![TextSection::new(
                                        "",
                                        TextStyle {
                                            font: font_assets.monospace.clone(),
                                            font_size: BODY_SIZE,
                                            color: Color::BLACK,
                                        },
                                    )]),
                                    KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_BUY_OPEN_NONE),
                                ));
                            }
                        });

                    // Sell Side
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(50.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgba(1., 0., 1., 0.5)),
                            ..default()
                        })
                        .with_children(|parent| {});
                });

            spawn_back_button::<BuySellBack>(
                parent,
                &font_assets,
                &palettes::ui::BUTTON_SET,
                &palettes::ui::BUTTON_BORDER_SET,
            );
        });
}

#[derive(Component, Default)]
struct BuySellBack;

fn buy_sell_back_pressed(
    mut buy_state: ResMut<NextState<StockBuySceneState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<BuySellBack>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            buy_state.set(StockBuySceneState::FeatureCompany);
        }
    }
}
