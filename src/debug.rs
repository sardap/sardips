use bevy::prelude::*;
use rand::seq::IteratorRandom;
use strum::IntoEnumIterator;

use crate::{
    food::{template::FoodTemplateDatabase, SpawnFoodEvent},
    money::Wallet,
    player::Player,
    simulation::{SimTimeScale, SimulationState},
    text_database::Language,
    text_translation::SelectedLanguageTag,
    GameState,
};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Sim time scale
        app.add_systems(OnExit(GameState::Loading), setup_debug_text)
            .add_systems(Update, update_sim_time_scale_debug_text)
            .add_systems(
                Update,
                (sim_time_scale_input, free_money, spawn_random_food)
                    .run_if(in_state(SimulationState::Running)),
            )
            .add_systems(Update, change_language);
    }
}

#[derive(Component)]
struct DebugText;

fn setup_debug_text(mut commands: Commands) {
    const SIZE: f32 = 20.0;

    let title_text_style = TextStyle {
        font_size: SIZE,
        color: Color::DARK_GREEN,
        ..default()
    };
    let value_text_style = TextStyle {
        font_size: SIZE,
        color: Color::GREEN,
        ..default()
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_sections(vec![
                    TextSection::new("DEBUG:\n", title_text_style.clone()),
                    // Sim time
                    TextSection::new("SIM_TIME:", title_text_style.clone()),
                    TextSection::new("", value_text_style.clone()),
                    TextSection::new("\n", value_text_style.clone()),
                ]),
                DebugText,
            ));
        });
}

const SIM_TIME_TEXT_SECTION: i32 = 2;

fn sim_time_scale_input(
    mut sim_time_scale: ResMut<SimTimeScale>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let time_scale_step: f32 = match sim_time_scale.0 {
        0.0..=1.0 => 0.1,
        1.0..=10.0 => 1.0,
        10.0..=120.0 => 10.0,
        120.0..=1000.0 => 100.0,
        1000.0..=10000.0 => 1000.0,
        _ => 10000.0,
    };

    if keyboard_input.just_pressed(KeyCode::Equal) {
        sim_time_scale.0 += time_scale_step;
    }

    if keyboard_input.just_pressed(KeyCode::Minus) {
        sim_time_scale.0 -= time_scale_step;
    }

    sim_time_scale.0 = sim_time_scale.0.clamp(0., 100000.);
}

fn update_sim_time_scale_debug_text(
    sim_time_scale: Res<SimTimeScale>,
    mut debug_text: Query<&mut Text, With<DebugText>>,
) {
    let mut debug_text = match debug_text.get_single_mut() {
        Ok(debug_text) => debug_text,
        Err(_) => return,
    };

    debug_text.sections[SIM_TIME_TEXT_SECTION as usize].value = format!("{:.1}", sim_time_scale.0);
}

fn change_language(
    mut language: Query<&mut Language, With<SelectedLanguageTag>>,
    buttons: Res<ButtonInput<KeyCode>>,
) {
    if buttons.just_pressed(KeyCode::KeyL) {
        let mut selected_language = language.single_mut();

        let languages = Language::iter().collect::<Vec<_>>();
        // Find the current language index
        let current_language_index = languages
            .iter()
            .position(|&lang| lang == *selected_language)
            .unwrap();
        // Cycle to the next language
        let next_language_index = (current_language_index + 1) % languages.len();
        *selected_language = languages[next_language_index];
    }
}

fn free_money(
    mut wallet: Query<&mut Wallet, With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        let mut wallet = wallet.single_mut();
        wallet.balance += 1000;
    }
}

fn spawn_random_food(
    mut food_events: EventWriter<SpawnFoodEvent>,
    food_template_db: Res<FoodTemplateDatabase>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        let random_food = food_template_db
            .iter()
            .choose(&mut rand::thread_rng())
            .unwrap();

        food_events.send(SpawnFoodEvent::new(&random_food.name));
    }
}
