use std::time::Duration;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use sardips_core::{assets::FontAssets, GameState};

use crate::{
    simulation::Simulated,
    stock_market::{Company, QuarterManger, ShareHistory},
};

pub struct StockTickerPlugin;

impl Plugin for StockTickerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(StockTickerState::None)
            .add_systems(OnEnter(GameState::ViewScreen), spawn_stock_ticker_panel)
            .add_systems(
                Update,
                (
                    populate_stock_ticker_rows,
                    update_stock_ticker_rows,
                    update_stock_ticker_quarter_progress,
                )
                    .run_if(resource_exists::<FontAssets>),
            );
    }
}

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum StockTickerState {
    #[default]
    None,
    Showing,
}

#[derive(Component)]
pub struct StockTickerCamera;

#[derive(Component)]
struct StockTickerPanelRoot;

fn spawn_stock_ticker_panel(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    existing: Query<Entity, With<StockTickerPanelRoot>>,
) {
    if existing.iter().count() > 0 {
        return;
    }

    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(200., 40.))),
                material: materials.add(Color::srgba_u8(0, 0, 0, 128)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            Simulated,
            StockTickerPanelRoot,
            StockTickerRowHolder,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2dBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: fonts.monospace.clone(),
                            font_size: 20.0,
                            color: Color::srgb_u8(255, 195, 0),
                        },
                    ),
                    transform: Transform::from_xyz(0.0, 15., 1.0),
                    ..default()
                },
                StockTickerQuarterProgress,
            ));
            parent.spawn((
                Text2dBundle {
                    transform: Transform::from_xyz(0.0, -3.0, 1.0),
                    ..default()
                },
                StockTickerRow::default(),
            ));
        });
}

#[derive(Component)]
struct StockTickerRowHolder;

#[derive(Component)]
struct StockTickerRow {
    offset: i32,
    companies: Vec<Entity>,
    timer: Timer,
}

impl Default for StockTickerRow {
    fn default() -> Self {
        Self {
            offset: 0,
            companies: Vec::new(),
            timer: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
        }
    }
}

#[derive(Default)]
struct PopulatedFirstTime {
    populated_once: bool,
}

fn populate_stock_ticker_rows(
    mut local: Local<PopulatedFirstTime>,
    panel: Query<&Children, With<StockTickerRowHolder>>,
    mut rows: Query<&mut StockTickerRow>,
    any_company_changed: Query<Entity, Or<(Changed<Company>, Added<Company>)>>,
    companies_removed: RemovedComponents<Company>,
    companies: Query<(Entity, &Company)>,
) {
    if !local.populated_once
        && any_company_changed.iter().count() == 0
        && companies_removed.is_empty()
    {
        return;
    }

    const COMPANY_PER_ROW: usize = 5;

    let mut companies: Vec<_> = companies.iter().collect();
    companies.sort_by(|(_, a), (_, b)| a.ticker.cmp(&b.ticker));

    local.populated_once = !companies.is_empty();

    for children in panel.iter() {
        let mut companies = companies.clone();
        for row in children.iter() {
            if let Ok(mut row) = rows.get_mut(*row) {
                row.companies.clear();
                for _ in 0..COMPANY_PER_ROW {
                    let (entity, _) = match companies.pop() {
                        Some(c) => c,
                        None => break,
                    };
                    row.companies.push(entity);
                }
            }
        }
    }
}

fn update_stock_ticker_rows(
    time: Res<Time>,
    fonts: Res<FontAssets>,
    panel: Query<&Children, With<StockTickerRowHolder>>,
    mut rows: Query<(&mut StockTickerRow, &mut Text)>,
    companies: Query<(&Company, &ShareHistory)>,
) {
    let mut companies = companies.iter().collect::<Vec<_>>();
    companies.sort_by(|(a, _), (b, _)| a.ticker.cmp(&b.ticker));

    for child in &panel {
        for child in child.iter() {
            if let Ok((mut row, mut row_text)) = rows.get_mut(*child) {
                const SLICE_SIZE: usize = 20;
                let mut text_color = Vec::new();
                let mut text = String::new();

                for (company, share_history) in &companies {
                    let last_price = company
                        .performance_history
                        .last()
                        .map(|p| p.stock_price)
                        .unwrap_or(share_history.cached_price);

                    let real_price = share_history.cached_price as f32 / 100.;

                    let real_change = (share_history.cached_price - last_price) as f32;
                    let change_symbol = if real_change > 0. { "+" } else { "" };
                    let percent_change = real_change / last_price as f32;

                    let mut to_push = String::new();

                    to_push.push(' ');

                    for _ in 0..(5 - company.ticker.len()) {
                        to_push.push(' ');
                    }
                    to_push.push_str(&company.ticker);
                    to_push.push(' ');

                    to_push.push_str(change_symbol);
                    let price_str = if real_price > 10. {
                        format!("{:.0}", real_price)
                    } else {
                        format!("{:.2}", real_price)
                    };
                    to_push.push_str(&price_str);
                    to_push.push(' ');

                    to_push.push_str(&format!("{:.2}", (real_change / 100.).abs()));
                    to_push.push(' ');

                    to_push.push_str(&format!("{:.2}", percent_change));

                    to_push.push(' ');

                    let color = if real_change > 0. {
                        Color::srgb_u8(0, 255, 0)
                    } else if real_change < 0. {
                        Color::srgb_u8(255, 0, 0)
                    } else {
                        Color::srgb_u8(255, 195, 0)
                    };

                    // Should probably use a range thing here
                    for _ in 0..to_push.len() {
                        text_color.push(color);
                    }

                    text += &to_push;
                }

                if text.is_empty() {
                    continue;
                }

                if row.timer.tick(time.delta()).just_finished() {
                    row.offset = row.offset.checked_add(1).unwrap_or(0);
                }

                // Fuck it define 20 sections?
                while row_text.sections.len() < SLICE_SIZE {
                    row_text.sections.push(TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: fonts.monospace.clone(),
                            font_size: 20.0,
                            color: Color::srgb_u8(255, 195, 0),
                        },
                    });
                }

                let letters: Vec<_> = text.chars().collect();

                let mut sub_offset = (row.offset as usize) % (letters.len() - 1);
                for i in 0..SLICE_SIZE {
                    row_text.sections[i].value = letters[sub_offset].to_string();
                    row_text.sections[i].style.color = text_color[sub_offset];
                    sub_offset = (sub_offset + 1) % (letters.len() - 1);
                }
            }
        }
    }
}

#[derive(Component)]
struct StockTickerQuarterProgress;

fn update_stock_ticker_quarter_progress(
    quarter_manager: Option<Res<QuarterManger>>,
    mut text: Query<&mut Text, With<StockTickerQuarterProgress>>,
) {
    let quarter_manager = match quarter_manager {
        Some(q) => q,
        None => return,
    };

    for mut text in text.iter_mut() {
        text.sections[0].value = quarter_manager.to_string();
    }
}
