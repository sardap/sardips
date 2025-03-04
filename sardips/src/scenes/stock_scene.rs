use std::cmp::Ordering;

use bevy::prelude::*;
use shared_deps::bevy_turborand::{DelegatedRng, GenCore, GlobalRng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    money::{money_aberration_decimal_display, money_aberration_display, money_display, Wallet},
    palettes,
    player::Player,
    stock_market::{
        BuyOrder, Company, CompanyPerformance, CompanyRank, NewOrder, OrderBook, OrderBrief,
        SellOrder, ShareHistory, SharePortfolio,
    },
};
use sardips_core::{
    assets::FontAssets,
    button_hover::ButtonHover,
    despawn_all,
    money_core::Money,
    rgb_to_color,
    text_translation::{warp_recursive_value_key, KeyText},
    ui_utils::spawn_back_button,
    GameState,
};
use text_keys::{
    STOCK_BUY_SCENE_BUY_EXISTING_BUY_LINE, STOCK_BUY_SCENE_BUY_MODE,
    STOCK_BUY_SCENE_BUY_PLAYER_OPEN_BUY_TITLE, STOCK_BUY_SCENE_BUY_REMOVE_ORDER_BUTTON,
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
    STOCK_BUY_SCENE_ONE_Q_CHANGE_HEADER, STOCK_BUY_SCENE_OWN_HEADER, STOCK_BUY_SCENE_SELL_MODE,
    STOCK_BUY_SCENE_STOCK_PRICE, STOCK_BUY_SCENE_STOCK_PRICE_HEADER, STOCK_BUY_SCENE_TICKER_HEADER,
    STOCK_BUY_SCENE_TITLE,
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
                (tick_input, exit_scene, rotate_static, expand_button_pressed)
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
                (
                    buy_sell_back_pressed,
                    update_player_money_text,
                    update_player_stocks_text,
                    update_stock_price_input,
                    update_stock_price_input_text,
                    update_stock_quantity_input,
                    disable_stock_quantity_input_buttons,
                    update_player_stock_quantity_input_text,
                    update_stock_total_price_text,
                    buy_stock_button_interacted,
                    update_buy_order_list,
                    disable_buy_button,
                    remove_order_button,
                    toggle_buy_sell_mode,
                    update_but_sell_select_mode_text,
                    update_buy_sell_button_text,
                    update_sell_orders,
                )
                    .run_if(in_state(StockBuySceneState::Buy)),
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
                                let (color, symbol) = match change.cmp(&0) {
                                    Ordering::Less => (BAD_COLOR, ""),
                                    Ordering::Equal => (Color::BLACK, ""),
                                    Ordering::Greater => (GOOD_COLOR, "+"),
                                };
                                let percent_change =
                                    change as f32 / last_performance.stock_price as f32 * 100.0;

                                parent.spawn((TextBundle::from_section(
                                    format!("{}{:.2}%", symbol, percent_change),
                                    TextStyle {
                                        font: font.clone(),
                                        font_size: ROW_TEXT_SIZE,
                                        color,
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

const BODY_SIZE: f32 = 30.;

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
                            Some(prev) => match buy_order.price.cmp(&prev.price) {
                                Ordering::Less => {}
                                Ordering::Equal => {
                                    prev.quantity += buy_order.remaining_quantity;
                                }
                                Ordering::Greater => {
                                    prev.price = buy_order.price;
                                    prev.quantity = buy_order.remaining_quantity;
                                }
                            },
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
    let (color, symbol) = match change.cmp(&0) {
        Ordering::Greater => (GOOD_COLOR, "+"),
        Ordering::Less => (BAD_COLOR, "-"),
        Ordering::Equal => (Color::BLACK, ""),
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
    selected_company: Query<&SelectedExpandedCompany>,
    player: Query<(&Wallet, &SharePortfolio), With<Player>>,
    companies: Query<(&Company, &ShareHistory)>,
) {
    let selected = selected_company.single().0;

    let (wallet, share_portfolio) = player.single();

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
            const HEADING_SIZE: f32 = 50.0;
            const PLAYER_INFO_SIZE: f32 = 30.0;

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: HEADING_SIZE,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, company.name_key()),
            ));

            // Money Section
            parent.spawn((
                (
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: PLAYER_INFO_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    KeyText::new().with_value(
                        0,
                        text_keys::STOCK_BUY_SCENE_BUY_PLAYER_MONEY,
                        &[money_display(wallet.balance).as_str()],
                    ),
                ),
                PlayerBuySceneMoney,
            ));

            // Player Shares Section
            parent.spawn((
                (
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: PLAYER_INFO_SIZE,
                            color: Color::BLACK,
                        },
                    ),
                    KeyText::new().with_value(
                        0,
                        text_keys::STOCK_BUY_SCENE_BUY_PLAYER_OWNS_STOCKS,
                        &[&share_portfolio.get_count(&selected).to_string()],
                    ),
                ),
                PlayerBuySceneStocks,
            ));

            // Buy / Sell form
            parent
                .spawn(NodeBundle {
                    style: Style {
                        // height: Val::Px(200.0),
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.5, 0., 0.5, 0.5)),
                    ..default()
                })
                .with_children(|parent| {
                    // Mode selector
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    border: UiRect::all(Val::Px(2.0)),
                                    margin: UiRect::all(Val::Px(5.0)),
                                    align_content: AlignContent::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                ..default()
                            },
                            BuySellModeSelectButton::default(),
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
                                        font_size: BODY_SIZE,
                                        color: Color::BLACK,
                                    },
                                ),
                                KeyText::new().with(0, STOCK_BUY_SCENE_BUY_MODE),
                                BuySelectModeText,
                            ));
                        });

                    let price_input_id = parent
                        .spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: BODY_SIZE,
                                    color: Color::BLACK,
                                },
                            ),
                            KeyText::new().with_value(0, STOCK_BUY_SCENE_STOCK_PRICE, &[""]),
                            StockPriceInput {
                                current: share_history.cached_price,
                            },
                        ))
                        .id();

                    let button_width =
                        Val::Percent(100. / StockPriceInputKind::iter().count() as f32);

                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceAround,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            for kind in StockPriceInputKind::iter() {
                                parent
                                    .spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: button_width,
                                                height: Val::Px(30.0),
                                                align_content: AlignContent::Center,
                                                margin: UiRect::all(Val::Px(2.0)),
                                                padding: UiRect::all(Val::Px(2.0)),
                                                justify_content: JustifyContent::Center,
                                                border: UiRect::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        ButtonHover::default()
                                            .with_background(palettes::ui::BUTTON_SET)
                                            .with_border(palettes::ui::BUTTON_BORDER_SET),
                                        kind,
                                        StockPriceInputButton {
                                            for_price: price_input_id,
                                        },
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            TextBundle::from_section(
                                                "",
                                                TextStyle {
                                                    font: font_assets.monospace.clone(),
                                                    font_size: 20.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            KeyText::new().with(0, kind.get_text_key()),
                                        ));
                                    });
                            }
                        });

                    // Quantity
                    let qty_input_id = parent
                        .spawn((
                            TextBundle::from_section(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: BODY_SIZE,
                                    color: Color::BLACK,
                                },
                            ),
                            KeyText::new().with_value(
                                0,
                                text_keys::STOCK_BUY_SCENE_BUY_BUY_QUANTITY,
                                &[""],
                            ),
                            StockQuantityInput { current: 1 },
                        ))
                        .id();

                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceAround,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            for kind in StockPriceInputKind::iter() {
                                parent
                                    .spawn((
                                        ButtonBundle {
                                            style: Style {
                                                width: button_width,
                                                height: Val::Px(30.0),
                                                align_content: AlignContent::Center,
                                                margin: UiRect::all(Val::Px(2.0)),
                                                padding: UiRect::all(Val::Px(2.0)),
                                                justify_content: JustifyContent::Center,
                                                border: UiRect::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                        ButtonHover::default()
                                            .with_background(palettes::ui::BUTTON_SET)
                                            .with_border(palettes::ui::BUTTON_BORDER_SET),
                                        kind,
                                        StockQuantityInputButton {
                                            for_quantity: qty_input_id,
                                        },
                                    ))
                                    .with_children(|parent| {
                                        parent.spawn((
                                            TextBundle::from_section(
                                                "",
                                                TextStyle {
                                                    font: font_assets.monospace.clone(),
                                                    font_size: 20.,
                                                    color: Color::BLACK,
                                                },
                                            ),
                                            KeyText::new().with(0, kind.get_text_key()),
                                        ));
                                    });
                            }
                        });

                    // Total price display
                    parent.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: font_assets.monospace.clone(),
                                font_size: BODY_SIZE,
                                color: Color::BLACK,
                            },
                        ),
                        KeyText::new().with_value(
                            0,
                            text_keys::STOCK_BUY_SCENE_BUY_BUY_TOTAL,
                            &[""],
                        ),
                        StockTotalPriceDisplay {
                            price_input: price_input_id,
                            quantity_input: qty_input_id,
                        },
                    ));

                    // Buy Button
                    parent
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    width: Val::Px(70.0),
                                    height: Val::Px(30.0),
                                    margin: UiRect::all(Val::Px(5.0)),
                                    align_content: AlignContent::Center,
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                ..default()
                            },
                            BuySellStockButton {
                                price_input: price_input_id,
                                quantity_input: qty_input_id,
                            },
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
                                        font_size: 25.,
                                        color: Color::BLACK,
                                    },
                                )
                                .with_style(Style {
                                    align_content: AlignContent::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                }),
                                KeyText::new(),
                                BuySellButtonText,
                            ));
                        });
                });

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
                            parent.spawn((
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
                                BuyOrderList,
                            ));
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
                        .with_children(|parent| {
                            parent.spawn((
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
                                SellOrderList,
                            ));
                        });
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

#[derive(Component)]
struct PlayerBuySceneMoney;

fn update_player_money_text(
    player: Query<&Wallet, (With<Player>, Changed<Wallet>)>,
    mut text: Query<&mut KeyText, With<PlayerBuySceneMoney>>,
) {
    let wallet = match player.get_single() {
        Ok(wallet) => wallet,
        Err(_) => return,
    };

    for mut text in &mut text {
        text.replace_value(0, 0, money_display(wallet.balance));
    }
}

#[derive(Component)]
struct PlayerBuySceneStocks;

fn update_player_stocks_text(
    player: Query<&SharePortfolio, (With<Player>, Changed<SharePortfolio>)>,
    selected: Query<&SelectedExpandedCompany>,
    mut text: Query<&mut KeyText, With<PlayerBuySceneStocks>>,
) {
    let portfolio = match player.get_single() {
        Ok(portfolio) => portfolio,
        Err(_) => return,
    };

    let selected = selected.single().0;

    for mut text in &mut text {
        text.replace_value(0, 0, portfolio.get_count(&selected).to_string());
    }
}

#[derive(Component)]
struct StockPriceInput {
    current: Money,
}

#[derive(Component, EnumIter, Copy, Clone)]
enum StockPriceInputKind {
    MuchHigher,
    Higher,
    Lower,
    MuchLower,
}

impl StockPriceInputKind {
    fn get_text_key(&self) -> &'static str {
        match self {
            StockPriceInputKind::MuchHigher => text_keys::STOCK_BUY_SCENE_BUY_BUY_MUCH_HIGHER,
            StockPriceInputKind::Higher => text_keys::STOCK_BUY_SCENE_BUY_BUY_HIGHER,
            StockPriceInputKind::Lower => text_keys::STOCK_BUY_SCENE_BUY_BUY_LOWER,
            StockPriceInputKind::MuchLower => text_keys::STOCK_BUY_SCENE_BUY_BUY_MUCH_LOWER,
        }
    }
}

#[derive(Component)]
struct StockPriceInputButton {
    for_price: Entity,
}

fn update_stock_price_input(
    mut input: Query<&mut StockPriceInput>,
    selected: Query<&SelectedExpandedCompany>,
    companies: Query<&ShareHistory, With<Company>>,
    buttons: Query<
        (&Interaction, &StockPriceInputKind, &StockPriceInputButton),
        Changed<Interaction>,
    >,
) {
    let selected = selected.single().0;

    for (interaction, kind, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let mut input = input.get_mut(button.for_price).unwrap();

        let share_history = companies.get(selected).unwrap();

        let small_increase = (share_history.cached_price as f32 * 0.01).ceil() as Money;
        let large_increase = (share_history.cached_price as f32 * 0.05).ceil() as Money;

        match kind {
            StockPriceInputKind::MuchHigher => {
                input.current += large_increase;
            }
            StockPriceInputKind::Higher => {
                input.current += small_increase;
            }
            StockPriceInputKind::Lower => {
                input.current = (input.current - small_increase).max(0);
            }
            StockPriceInputKind::MuchLower => {
                input.current = (input.current - large_increase).max(0);
            }
        }
    }
}

fn update_stock_price_input_text(
    mut text: Query<
        (&mut KeyText, &StockPriceInput),
        Or<(Changed<StockPriceInput>, Added<StockPriceInput>)>,
    >,
) {
    for (mut text, input) in &mut text {
        text.replace_value(0, 0, money_display(input.current));
    }
}

#[derive(Component)]
struct StockQuantityInput {
    current: u64,
}

fn update_player_stock_quantity_input_text(
    mut text: Query<(&mut KeyText, &StockQuantityInput), Changed<StockQuantityInput>>,
) {
    for (mut text, input) in &mut text {
        text.replace_value(0, 0, input.current.to_string());
    }
}

#[derive(Component)]
struct StockQuantityInputButton {
    for_quantity: Entity,
}

fn update_stock_quantity_input(
    mut input: Query<&mut StockQuantityInput>,
    buttons: Query<
        (
            &Interaction,
            &StockPriceInputKind,
            &StockQuantityInputButton,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, kind, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let mut input = input.get_mut(button.for_quantity).unwrap();

        let small_increase = 1;
        let large_increase = 10;

        match kind {
            StockPriceInputKind::MuchHigher => {
                input.current += large_increase;
            }
            StockPriceInputKind::Higher => {
                input.current += small_increase;
            }
            StockPriceInputKind::Lower => {
                input.current = input.current.saturating_sub(small_increase)
            }
            StockPriceInputKind::MuchLower => {
                input.current = input.current.saturating_sub(large_increase);
            }
        }

        if input.current == 0 {
            input.current = 1;
        }
    }
}

fn disable_stock_quantity_input_buttons(
    mut commands: Commands,
    with_interaction: Query<Entity, (With<StockQuantityInputButton>, With<Interaction>)>,
    without_interaction: Query<Entity, (With<StockQuantityInputButton>, Without<Interaction>)>,
    input: Query<&StockQuantityInput>,
    buttons: Query<(Entity, &StockPriceInputKind, &StockQuantityInputButton)>,
    player_share_portfolio: Query<&SharePortfolio, With<Player>>,
    selected: Query<&SelectedExpandedCompany>,
    buy_sell_mode: Query<&BuySellModeSelectButton>,
) {
    let buy_mode = buy_sell_mode.single();
    let selected = selected.single().0;
    let player_portfolio = player_share_portfolio.single();

    for (entity, kind, button) in &buttons {
        let current = input.get(button.for_quantity).unwrap().current;

        let mut enabled = true;

        match *kind {
            StockPriceInputKind::Lower | StockPriceInputKind::MuchLower => {
                if current <= 1 {
                    enabled = false;
                }
            }
            StockPriceInputKind::Higher | StockPriceInputKind::MuchHigher => {
                if *buy_mode == BuySellModeSelectButton::Sell
                    && player_portfolio.get_count(&selected) < current + 1
                {
                    enabled = false;
                }
            }
        }

        if enabled && without_interaction.get(entity).is_ok() {
            commands.entity(entity).insert(Interaction::None);
        } else if !enabled && with_interaction.get(entity).is_ok() {
            commands.entity(entity).remove::<Interaction>();
        }
    }
}

#[derive(Component)]
struct StockTotalPriceDisplay {
    price_input: Entity,
    quantity_input: Entity,
}

fn update_stock_total_price_text(
    price_input: Query<&StockPriceInput>,
    quantity_input: Query<&StockQuantityInput>,
    mut text: Query<(&mut KeyText, &StockTotalPriceDisplay)>,
) {
    for (mut text, display) in &mut text {
        let price = price_input.get(display.price_input).unwrap().current;
        let quantity = quantity_input.get(display.quantity_input).unwrap().current;

        text.replace_value(0, 0, money_display(price * quantity as i64));
    }
}

#[derive(Component)]
struct BuyOrderList;

#[derive(Default)]
struct LocalUpdateBuyOrders {
    orders_len: usize,
}

const ORDER_LIST_BODY_SIZE: f32 = 30.;
const MAX_BUY_ORDERS_SHOWN: usize = 4;

fn update_buy_order_list(
    mut local: Local<LocalUpdateBuyOrders>,
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    order_book: Res<OrderBook>,
    should_update: Query<Entity, Or<(Added<BuyOrderList>, Changed<SharePortfolio>)>>,
    buy_order_list: Query<Entity, With<BuyOrderList>>,
    selected: Query<&SelectedExpandedCompany>,
    player: Query<Entity, With<Player>>,
) {
    if should_update.iter().count() == 0 && local.orders_len == order_book.buy_orders.len() {
        return;
    }

    let buy_order_list = buy_order_list.single();

    local.orders_len = order_book.buy_orders.len();

    commands.entity(buy_order_list).despawn_descendants();

    let selected = selected.single().0;
    let player_entity = player.single();

    commands.entity(buy_order_list).with_children(|parent| {
        let open_buy_orders: Vec<_> = order_book
            .buy_orders
            .iter()
            .filter(|order| order.company == selected)
            .collect();

        let mut non_player_buy_orders: Vec<OrderBrief> = vec![];
        let mut player_buy_orders = vec![];
        {
            for buy_order in open_buy_orders {
                if buy_order.buyer == player_entity {
                    player_buy_orders.push(buy_order)
                } else {
                    if let Some(last) = non_player_buy_orders.iter_mut().last() {
                        if last.price == buy_order.price {
                            last.quantity += buy_order.remaining_quantity;
                            continue;
                        }
                    }
                    non_player_buy_orders
                        .push(OrderBrief::new(buy_order.quantity, buy_order.price));
                }
            }
        }

        non_player_buy_orders.sort_by(|a, b| a.price.cmp(&b.price));

        // Players Open orders
        if !player_buy_orders.is_empty() {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: ORDER_LIST_BODY_SIZE - 3.,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, STOCK_BUY_SCENE_BUY_PLAYER_OPEN_BUY_TITLE),
            ));

            for order in player_buy_orders.iter().take(MAX_BUY_ORDERS_SHOWN) {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_sections(vec![TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: ORDER_LIST_BODY_SIZE,
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

                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(30.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    ..default()
                                },
                                RemoveOrderButton(order.id),
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
                                            font_size: ORDER_LIST_BODY_SIZE,
                                            color: Color::BLACK,
                                        },
                                    )
                                    .with_style(Style {
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    }),
                                    KeyText::new().with(0, STOCK_BUY_SCENE_BUY_REMOVE_ORDER_BUTTON),
                                ));
                            });
                    });
            }
        }

        // Non Player open orders
        let non_player_orders_to_show =
            MAX_BUY_ORDERS_SHOWN.saturating_sub(player_buy_orders.len());

        if !non_player_buy_orders.is_empty() {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: ORDER_LIST_BODY_SIZE,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, text_keys::STOCK_BUY_SCENE_BUY_EXISTING_BUY_TITLE),
            ));

            for order in non_player_buy_orders.iter().take(non_player_orders_to_show) {
                parent.spawn((
                    TextBundle::from_sections(vec![TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: ORDER_LIST_BODY_SIZE,
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
                        font_size: ORDER_LIST_BODY_SIZE,
                        color: Color::BLACK,
                    },
                )]),
                KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_BUY_OPEN_NONE),
            ));
        }
    });
}

#[derive(Component)]
struct BuySellStockButton {
    price_input: Entity,
    quantity_input: Entity,
}

fn buy_stock_button_interacted(
    mut order_book: ResMut<OrderBook>,
    selected: Query<&SelectedExpandedCompany>,
    price_input: Query<&StockPriceInput>,
    quantity_input: Query<&StockQuantityInput>,
    mut player: Query<(Entity, &mut Wallet, &mut SharePortfolio), With<Player>>,
    buttons: Query<(&Interaction, &BuySellStockButton), Changed<Interaction>>,
    buy_sell_mode: Query<&BuySellModeSelectButton>,
) {
    let (interaction, buy_button) = match buttons.get_single() {
        Ok(button) => button,
        Err(_) => return,
    };
    if *interaction != Interaction::Pressed {
        return;
    }

    let buy_sell_mode = buy_sell_mode.single();
    let price = price_input.get(buy_button.price_input).unwrap().current;
    let quantity = quantity_input
        .get(buy_button.quantity_input)
        .unwrap()
        .current;
    let (player_entity, mut player_wallet, mut share_portfolio) = player.single_mut();

    match *buy_sell_mode {
        BuySellModeSelectButton::Buy => {
            let selected = selected.single().0;

            let total_price = price * quantity as i64;
            if total_price > player_wallet.balance {
                return;
            }

            player_wallet.balance -= total_price;

            order_book.add(NewOrder::Buy(BuyOrder::new(
                selected,
                quantity,
                price,
                player_entity,
            )));
        }
        BuySellModeSelectButton::Sell => {
            let selected = selected.single().0;

            share_portfolio.remove_shares(selected, quantity);

            order_book.add(NewOrder::Sell(SellOrder::new(
                selected,
                quantity,
                price,
                player_entity,
            )));
        }
    }
}

fn disable_buy_button(
    mut commands: Commands,
    price_input: Query<&StockPriceInput>,
    quantity_input: Query<&StockQuantityInput>,
    player_wallet: Query<&Wallet, With<Player>>,
    share_portfolio: Query<&SharePortfolio, With<Player>>,
    selected: Query<&SelectedExpandedCompany>,
    buttons: Query<(Entity, &BuySellStockButton)>,
    without_interaction: Query<Entity, (With<BuySellStockButton>, Without<Interaction>)>,
    with_interaction: Query<Entity, (With<BuySellStockButton>, With<Interaction>)>,
    current_mode: Query<&BuySellModeSelectButton>,
) {
    let current_mode = current_mode.single();
    let selected = selected.single().0;
    let (entity, buy_button) = buttons.single();
    let quantity = quantity_input
        .get(buy_button.quantity_input)
        .unwrap()
        .current;

    match *current_mode {
        BuySellModeSelectButton::Sell => {
            let player_share_portfolio = share_portfolio.single();
            let count = player_share_portfolio.get_count(&selected);
            if (quantity == 0 || count < quantity) && with_interaction.get(entity).is_ok() {
                commands.entity(entity).remove::<Interaction>();
            } else if count > quantity && without_interaction.get(entity).is_ok() {
                commands.entity(entity).insert(Interaction::default());
            }
        }
        BuySellModeSelectButton::Buy => {
            let player_wallet = player_wallet.single();
            let price = price_input.get(buy_button.price_input).unwrap().current;

            let total_price = price * quantity as i64;

            if total_price < player_wallet.balance && without_interaction.get(entity).is_ok() {
                commands.entity(entity).insert(Interaction::default());
            } else if total_price > player_wallet.balance && with_interaction.get(entity).is_ok() {
                commands.entity(entity).remove::<Interaction>();
            }
        }
    }
}

#[derive(Component)]
struct RemoveOrderButton(u64);

fn remove_order_button(
    mut order_book: ResMut<OrderBook>,
    buttons: Query<(&Interaction, &RemoveOrderButton), Changed<Interaction>>,
) {
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        order_book.remove_order(button.0);
    }
}

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
enum BuySellModeSelectButton {
    #[default]
    Buy,
    Sell,
}

fn toggle_buy_sell_mode(
    selected: Query<&SelectedExpandedCompany>,
    player: Query<&SharePortfolio, With<Player>>,
    companies: Query<&ShareHistory, With<Company>>,
    mut price_input: Query<&mut StockPriceInput>,
    mut quantity_input: Query<&mut StockQuantityInput>,
    mut buttons: Query<(&Interaction, &mut BuySellModeSelectButton), Changed<Interaction>>,
) {
    for (interaction, mut button) in &mut buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let selected = selected.single().0;
        let share_portfolio = player.single();

        let share_history = companies.get(selected).unwrap();

        for mut input in &mut price_input {
            input.current = share_history.cached_price;
        }

        let next_mode = match *button {
            BuySellModeSelectButton::Buy => BuySellModeSelectButton::Sell,
            BuySellModeSelectButton::Sell => BuySellModeSelectButton::Buy,
        };

        for mut input in &mut quantity_input {
            input.current = match next_mode {
                BuySellModeSelectButton::Buy => 1,
                BuySellModeSelectButton::Sell => share_portfolio.get_count(&selected).min(1),
            };
        }

        *button = next_mode;
    }
}

#[derive(Component)]
struct BuySelectModeText;

fn update_but_sell_select_mode_text(
    buy_select_mode: Query<
        (&BuySellModeSelectButton, &Children),
        Or<(
            Changed<BuySellModeSelectButton>,
            Added<BuySellModeSelectButton>,
        )>,
    >,
    mut text: Query<&mut KeyText, With<BuySelectModeText>>,
) {
    for (mode, children) in &buy_select_mode {
        for child in children {
            if let Ok(mut text) = text.get_mut(*child) {
                *text = KeyText::new().with(
                    0,
                    match mode {
                        BuySellModeSelectButton::Buy => STOCK_BUY_SCENE_BUY_MODE,
                        BuySellModeSelectButton::Sell => STOCK_BUY_SCENE_SELL_MODE,
                    },
                );
            }
        }
    }
}

#[derive(Component)]
struct BuySellButtonText;

fn update_buy_sell_button_text(
    buy_sell_mode: Query<
        &BuySellModeSelectButton,
        Or<(
            Added<BuySellModeSelectButton>,
            Changed<BuySellModeSelectButton>,
        )>,
    >,
    mut text: Query<&mut KeyText, With<BuySellButtonText>>,
) {
    for mode in &buy_sell_mode {
        for mut text in &mut text {
            *text = KeyText::new().with(
                0,
                match mode {
                    BuySellModeSelectButton::Buy => STOCK_BUY_SCENE_FEATURE_BUY_BUTTON,
                    BuySellModeSelectButton::Sell => STOCK_BUY_SCENE_FEATURE_SELL_BUTTON,
                },
            );
        }
    }
}

#[derive(Component)]
struct SellOrderList;

fn update_sell_orders(
    mut local: Local<LocalUpdateBuyOrders>,
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    order_book: Res<OrderBook>,
    should_update: Query<Entity, Or<(Added<SellOrderList>, Changed<SharePortfolio>)>>,
    sell_order_list: Query<Entity, With<SellOrderList>>,
    selected: Query<&SelectedExpandedCompany>,
    player: Query<Entity, With<Player>>,
) {
    if should_update.iter().count() == 0 && local.orders_len == order_book.total_sell_orders() {
        return;
    }

    let sell_order_list = sell_order_list.single();

    local.orders_len = order_book.total_sell_orders();

    commands.entity(sell_order_list).despawn_descendants();

    let selected = selected.single().0;
    let player_entity = player.single();

    commands.entity(sell_order_list).with_children(|parent| {
        let open_sell_orders = order_book.get_sell_orders(selected);

        let mut non_player_sell_orders: Vec<OrderBrief> = vec![];
        let mut player_sell_orders = vec![];
        {
            for sell_order in open_sell_orders {
                if sell_order.seller == player_entity {
                    player_sell_orders.push(sell_order)
                } else {
                    if let Some(last) = non_player_sell_orders.iter_mut().last() {
                        if last.price == sell_order.price {
                            last.quantity += sell_order.remaining_quantity;
                            continue;
                        }
                    }
                    non_player_sell_orders
                        .push(OrderBrief::new(sell_order.quantity, sell_order.price));
                }
            }
        }

        non_player_sell_orders.sort_by(|a, b| a.price.cmp(&b.price));

        // Players Open orders
        if !player_sell_orders.is_empty() {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: ORDER_LIST_BODY_SIZE - 3.,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, text_keys::STOCK_BUY_SCENE_BUY_PLAYER_OPEN_SELL_TITLE),
            ));

            for order in player_sell_orders.iter().take(MAX_BUY_ORDERS_SHOWN) {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn((
                            TextBundle::from_sections(vec![TextSection::new(
                                "",
                                TextStyle {
                                    font: font_assets.monospace.clone(),
                                    font_size: ORDER_LIST_BODY_SIZE,
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

                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(30.0),
                                        border: UiRect::all(Val::Px(2.0)),
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    ..default()
                                },
                                RemoveOrderButton(order.id),
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
                                            font_size: ORDER_LIST_BODY_SIZE,
                                            color: Color::BLACK,
                                        },
                                    )
                                    .with_style(Style {
                                        align_content: AlignContent::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    }),
                                    KeyText::new().with(0, STOCK_BUY_SCENE_BUY_REMOVE_ORDER_BUTTON),
                                ));
                            });
                    });
            }
        }

        // Non Player open orders
        let non_player_orders_to_show =
            MAX_BUY_ORDERS_SHOWN.saturating_sub(player_sell_orders.len());

        if !non_player_sell_orders.is_empty() {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font_assets.monospace.clone(),
                        font_size: ORDER_LIST_BODY_SIZE,
                        color: Color::BLACK,
                    },
                ),
                KeyText::new().with(0, text_keys::STOCK_BUY_SCENE_BUY_EXISTING_SELL_TITLE),
            ));

            for order in non_player_sell_orders
                .iter()
                .take(non_player_orders_to_show)
            {
                parent.spawn((
                    TextBundle::from_sections(vec![TextSection::new(
                        "",
                        TextStyle {
                            font: font_assets.monospace.clone(),
                            font_size: ORDER_LIST_BODY_SIZE,
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
                        font_size: ORDER_LIST_BODY_SIZE,
                        color: Color::BLACK,
                    },
                )]),
                KeyText::new().with(0, STOCK_BUY_SCENE_FEATURE_SELL_OPEN_NONE),
            ));
        }
    });
}
