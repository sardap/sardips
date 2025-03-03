use std::{collections::HashMap, str::FromStr};

use crate::{
    food::{Food, SpawnFoodEvent},
    money::Wallet,
    pet::{
        dipdex::DipdexDiscoveredEntries, evolve::ShouldEvolve, poop::spawn_poop,
        template::SpawnPetEvent, Pet,
    },
    player::Player,
    simulation::SimTimeScale,
};
use bevy::prelude::*;
use sardips_core::{
    food_core::FoodTemplateDatabase,
    money_core::Money,
    name::EntityName,
    particles::SPARKS,
    pet_core::{PetTemplateDatabase, DEFAULT_POOP_TEXTURE},
    text_database::Language,
    text_translation::SelectedLanguageTag,
    GameState,
};
use shared_deps::bevy_turborand::{DelegatedRng, GlobalRng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Sim time scale
        app.insert_state(DevConsoleState::default())
            .add_event::<DevConsoleCommand>()
            .add_systems(OnExit(GameState::Loading), setup_debug_text)
            .add_systems(
                Update,
                (
                    update_sim_time_scale_debug_text,
                    toggle_dev_console,
                    action_dev_console_command,
                ),
            )
            .add_systems(OnEnter(DevConsoleState::Open), spawn_dev_console)
            .add_systems(OnEnter(DevConsoleState::Closed), teardown_dev_console)
            .add_systems(
                Update,
                (update_dev_console_text, dev_console_text_input)
                    .run_if(in_state(DevConsoleState::Open)),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum DevConsoleState {
    #[default]
    Closed,
    Open,
}

#[derive(Component)]
struct DebugText;

const SIM_TIME_TEXT_SECTION: i32 = 2;

fn setup_debug_text(mut commands: Commands) {
    const SIZE: f32 = 20.0;

    let title_text_style = TextStyle {
        font_size: SIZE,
        color: Color::Srgba(bevy::color::palettes::css::DARK_GREEN),
        ..default()
    };
    let value_text_style = TextStyle {
        font_size: SIZE,
        color: Color::Srgba(bevy::color::palettes::css::GREEN),
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

// Dev console
fn toggle_dev_console(
    mut next_state: ResMut<NextState<DevConsoleState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<DevConsoleState>>,
) {
    if keyboard_input.just_pressed(KeyCode::AltLeft) {
        info!("Toggling dev console");
        next_state.set(match **state {
            DevConsoleState::Closed => DevConsoleState::Open,
            DevConsoleState::Open => DevConsoleState::Closed,
        });
    }
}

#[derive(Component)]
struct DevConsole;

#[derive(Component, Default)]
struct DevConsoleInput {
    pub text: String,
}

#[derive(Component)]
enum DevConsoleHistoryEntry {
    UserInput(String),
    CommandOutput(String),
}

#[derive(Component, Default)]
struct DevConsoleHistory {
    pub history: Vec<DevConsoleHistoryEntry>,
}

impl DevConsoleHistory {
    pub fn push_command_output<T: ToString>(&mut self, output: T) {
        self.history
            .push(DevConsoleHistoryEntry::CommandOutput(output.to_string()));
    }

    pub fn get_last_user_input(&self) -> Option<&String> {
        self.history.iter().rev().find_map(|entry| match entry {
            DevConsoleHistoryEntry::UserInput(input) => Some(input),
            _ => None,
        })
    }
}

fn spawn_dev_console(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Vh(80.),
                    width: Val::Vw(100.),
                    height: Val::Vh(20.),
                    border: UiRect::top(Val::Px(5.)),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0., 0., 0., 0.75)),
                border_color: BorderColor(Color::BLACK),
                ..default()
            },
            DevConsole,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_sections(vec![
                    TextSection::new(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::Srgba(bevy::color::palettes::css::GREEN),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::Srgba(bevy::color::palettes::css::DARK_GRAY),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::Srgba(bevy::color::palettes::css::GREEN),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::Srgba(bevy::color::palettes::css::GREEN),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "> ",
                        TextStyle {
                            font_size: 20.0,
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::Srgba(bevy::color::palettes::css::GREEN),
                            ..default()
                        },
                    ),
                    TextSection::new(
                        "",
                        TextStyle {
                            font_size: 20.0,
                            color: Color::Srgba(bevy::color::palettes::css::LIMEGREEN),
                            ..default()
                        },
                    ),
                ]),
                DevConsoleInput::default(),
                DevConsoleHistory::default(),
            ));
        });
}

fn teardown_dev_console(mut commands: Commands, dev_console: Query<Entity, With<DevConsole>>) {
    for entity in dev_console.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

struct CursorFlashTimer {
    timer: Timer,
    showing: bool,
}

impl Default for CursorFlashTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            showing: true,
        }
    }
}

fn update_dev_console_text(
    time: Res<Time>,
    mut cursor_flash_timer: Local<CursorFlashTimer>,
    mut dev_console_text: Query<(&mut Text, &DevConsoleInput, &DevConsoleHistory)>,
) {
    let (mut text, dev_console_text, history) = match dev_console_text.get_single_mut() {
        Ok(text) => text,
        Err(_) => return,
    };

    // Get last 4 lines of history
    let history = history.history.iter().rev().take(4).collect::<Vec<_>>();
    for i in 0..4 {
        text.sections[i].value = "\n".to_owned();
    }
    for (i, entry) in history.iter().enumerate() {
        let i = 4 - i - 1;
        match entry {
            DevConsoleHistoryEntry::UserInput(input) => {
                text.sections[i].value = input.clone();
                text.sections[i].style.color = Color::Srgba(bevy::color::palettes::css::DARK_GREEN);
            }
            DevConsoleHistoryEntry::CommandOutput(output) => {
                text.sections[i].value = output.clone();
                text.sections[i].style.color = Color::WHITE;
            }
        }

        text.sections[i].value.push('\n');
    }

    // User input is 5
    text.sections[5].value = dev_console_text.text.clone();

    // Cursor is 6 and flashes
    if cursor_flash_timer.timer.tick(time.delta()).just_finished() {
        cursor_flash_timer.showing = !cursor_flash_timer.showing;
    }
    text.sections[6].value = if cursor_flash_timer.showing { "_" } else { "" }.to_string();
}

lazy_static! {
    static ref KEY_MAP: HashMap<KeyCode, String> = {
        let mut map = HashMap::new();

        map.insert(KeyCode::Digit0, "0".to_string());
        map.insert(KeyCode::Digit1, "1".to_string());
        map.insert(KeyCode::Digit2, "2".to_string());
        map.insert(KeyCode::Digit3, "3".to_string());
        map.insert(KeyCode::Digit4, "4".to_string());
        map.insert(KeyCode::Digit5, "5".to_string());
        map.insert(KeyCode::Digit6, "6".to_string());
        map.insert(KeyCode::Digit7, "7".to_string());
        map.insert(KeyCode::Digit8, "8".to_string());
        map.insert(KeyCode::Digit9, "9".to_string());
        map.insert(KeyCode::KeyA, "a".to_string());
        map.insert(KeyCode::KeyB, "b".to_string());
        map.insert(KeyCode::KeyC, "c".to_string());
        map.insert(KeyCode::KeyD, "d".to_string());
        map.insert(KeyCode::KeyE, "e".to_string());
        map.insert(KeyCode::KeyF, "f".to_string());
        map.insert(KeyCode::KeyG, "g".to_string());
        map.insert(KeyCode::KeyH, "h".to_string());
        map.insert(KeyCode::KeyI, "i".to_string());
        map.insert(KeyCode::KeyJ, "j".to_string());
        map.insert(KeyCode::KeyK, "k".to_string());
        map.insert(KeyCode::KeyL, "l".to_string());
        map.insert(KeyCode::KeyM, "m".to_string());
        map.insert(KeyCode::KeyN, "n".to_string());
        map.insert(KeyCode::KeyO, "o".to_string());
        map.insert(KeyCode::KeyP, "p".to_string());
        map.insert(KeyCode::KeyQ, "q".to_string());
        map.insert(KeyCode::KeyR, "r".to_string());
        map.insert(KeyCode::KeyS, "s".to_string());
        map.insert(KeyCode::KeyT, "t".to_string());
        map.insert(KeyCode::KeyU, "u".to_string());
        map.insert(KeyCode::KeyV, "v".to_string());
        map.insert(KeyCode::KeyW, "w".to_string());
        map.insert(KeyCode::KeyX, "x".to_string());
        map.insert(KeyCode::KeyY, "y".to_string());
        map.insert(KeyCode::KeyZ, "z".to_string());
        map.insert(KeyCode::Minus, "-".to_string());
        map.insert(KeyCode::Equal, "=".to_string());
        map.insert(KeyCode::Space, " ".to_string());
        map.insert(KeyCode::Period, ".".to_string());

        map
    };
    static ref KEY_UPPERCASE_MAP: HashMap<KeyCode, String> = {
        let mut map = HashMap::new();

        map.insert(KeyCode::Digit0, ")".to_string());
        map.insert(KeyCode::Digit1, "!".to_string());
        map.insert(KeyCode::Digit2, "@".to_string());
        map.insert(KeyCode::Digit3, "#".to_string());
        map.insert(KeyCode::Digit4, "$".to_string());
        map.insert(KeyCode::Digit5, "%".to_string());
        map.insert(KeyCode::Digit6, "^".to_string());
        map.insert(KeyCode::Digit7, "&".to_string());
        map.insert(KeyCode::Digit8, "*".to_string());
        map.insert(KeyCode::Digit9, "(".to_string());
        map.insert(KeyCode::Minus, "_".to_string());
        map.insert(KeyCode::Equal, "+".to_string());
        map.insert(KeyCode::Period, ">".to_string());

        map
    };
}

fn common_prefix(matches: &[&str]) -> String {
    let mut common_prefix = String::new();
    let chars = matches[0].chars();
    for c in chars {
        if matches.iter().all(|m| m.starts_with(&common_prefix)) {
            common_prefix.push(c);
        } else {
            break;
        }
    }

    if !common_prefix.is_empty() {
        common_prefix.pop();
    }

    common_prefix
}

fn command_auto_complete(
    command: &str,
    sub_command: &str,
    food_db: &Option<Res<FoodTemplateDatabase>>,
) -> Option<String> {
    let split = match command {
        DevConsoleCommand::SPAWN_FOOD_COMMAND => {
            if let Some(food_db) = food_db {
                let matches = food_db
                    .templates
                    .iter()
                    .filter(|template| template.name.starts_with(sub_command))
                    .map(|template| template.name.as_str())
                    .collect::<Vec<_>>();
                match matches.len().cmp(&1) {
                    std::cmp::Ordering::Less => None,
                    std::cmp::Ordering::Equal => Some(matches[0].to_string()),
                    std::cmp::Ordering::Greater => Some(common_prefix(&matches)),
                }
            } else {
                None
            }
        }
        _ => None,
    };

    split.map(|matched| format!("{} {}", command, matched))
}

fn dev_console_text_input(
    input: Res<ButtonInput<KeyCode>>,
    food_db: Option<Res<FoodTemplateDatabase>>,
    mut dev_console_text: Query<(&mut DevConsoleInput, &mut DevConsoleHistory)>,
    mut dev_console_commands: EventWriter<DevConsoleCommand>,
) {
    let (mut text, mut history) = dev_console_text.single_mut();

    for pressed in input.get_just_pressed() {
        if *pressed == KeyCode::Backspace {
            text.text.pop();
            continue;
        }

        if *pressed == KeyCode::Tab {
            let splits = text.text.split_whitespace().collect::<Vec<_>>();
            let possible = if splits.len() == 1 {
                let matches = DEV_CONSOLE_COMMANDS
                    .iter()
                    .filter(|command| command.starts_with(splits[0]))
                    .map(|command| command.as_str())
                    .collect::<Vec<_>>();
                match matches.len().cmp(&1) {
                    std::cmp::Ordering::Less => None,
                    std::cmp::Ordering::Equal => Some(matches[0].to_string()),
                    std::cmp::Ordering::Greater => Some(common_prefix(&matches)),
                }
            } else if splits.len() == 2 {
                command_auto_complete(splits[0], splits[1], &food_db)
            } else {
                None
            };

            if let Some(matched) = possible {
                text.text = matched.to_string();
            }
        }

        if *pressed == KeyCode::ArrowUp {
            if let Some(last_input) = history.get_last_user_input() {
                text.text = last_input.clone();
            }
        }

        if input.pressed(KeyCode::ShiftLeft) {
            if let Some(key) = KEY_UPPERCASE_MAP.get(pressed) {
                text.text.push_str(key);
            } else if let Some(key) = KEY_MAP.get(pressed) {
                text.text.push_str(key.to_uppercase().as_str());
            }
        } else if let Some(key) = KEY_MAP.get(pressed) {
            text.text.push_str(key);
        }
    }

    if input.just_pressed(KeyCode::Enter) {
        let splits = text.text.split_whitespace().collect::<Vec<_>>();
        if splits.is_empty() {
            return;
        }

        match splits[0] {
            DevConsoleCommand::SET_SIM_TIME_SCALE_COMMAND => {
                if splits.len() > 1 {
                    if let Ok(scale) = splits[1].parse::<f32>() {
                        dev_console_commands.send(DevConsoleCommand::SetSimTimeScale(scale));
                    }
                }
            }
            DevConsoleCommand::SPAWN_PET_COMMAND => {
                if splits.len() > 1 {
                    dev_console_commands.send(DevConsoleCommand::SpawnPet(splits[1].to_string()));
                }
            }
            DevConsoleCommand::EVOLVE_PET_COMMAND => {
                if splits.len() > 1 {
                    dev_console_commands.send(DevConsoleCommand::EvolvePet(splits[1].to_string()));
                }
            }
            DevConsoleCommand::SPAWN_FOOD_COMMAND => {
                if splits.len() > 1 {
                    dev_console_commands.send(DevConsoleCommand::SpawnFood(splits[1].to_string()));
                }
            }
            DevConsoleCommand::SPAWN_POOP_COMMAND => {
                dev_console_commands.send(DevConsoleCommand::SpawnPoop);
            }
            DevConsoleCommand::CLEAR_ALL_PETS_COMMAND => {
                dev_console_commands.send(DevConsoleCommand::ClearAllPets);
            }
            DevConsoleCommand::CLEAR_ALL_FOODS_COMMAND => {
                dev_console_commands.send(DevConsoleCommand::ClearAllFoods);
            }
            DevConsoleCommand::CHANGE_LANGUAGE_COMMAND => {
                if splits.len() > 1 {
                    dev_console_commands
                        .send(DevConsoleCommand::ChangeLanguage(splits[1].to_string()));
                }
            }
            DevConsoleCommand::DISCOVER_COMPLETE_DIPDEX_COMMAND => {
                dev_console_commands.send(DevConsoleCommand::DiscoverCompleteDipdex);
            }
            DevConsoleCommand::SPAWN_SPEWER_COMMAND => {
                dev_console_commands.send(DevConsoleCommand::SpawnSpewer);
            }
            DevConsoleCommand::GIVE_MONEY_COMMAND => {
                if splits.len() > 1 {
                    if let Ok(money) = splits[1].parse::<Money>() {
                        dev_console_commands.send(DevConsoleCommand::GiveMoney(money));
                    }
                } else {
                    dev_console_commands.send(DevConsoleCommand::GiveMoney(100));
                }
            }
            _ => {
                error!("Unknown command: {}", splits[0]);
                history.push_command_output(format!("Unknown command: \"{}\"", splits[0]));
            }
        }

        history
            .history
            .push(DevConsoleHistoryEntry::UserInput(text.text.clone()));
        text.text.clear();
    }
}

#[derive(Event, EnumIter)]
enum DevConsoleCommand {
    SetSimTimeScale(f32),
    SpawnPet(String),
    EvolvePet(String),
    SpawnFood(String),
    SpawnSpewer,
    SpawnPoop,
    ClearAllPets,
    ClearAllFoods,
    ChangeLanguage(String),
    DiscoverCompleteDipdex,
    GiveMoney(Money),
}

impl DevConsoleCommand {
    const SET_SIM_TIME_SCALE_COMMAND: &'static str = "set_sim_time_scale";
    const SPAWN_PET_COMMAND: &'static str = "spawn_pet";
    const EVOLVE_PET_COMMAND: &'static str = "evolve_pet";
    const SPAWN_FOOD_COMMAND: &'static str = "spawn_food";
    const SPAWN_POOP_COMMAND: &'static str = "spawn_poop";
    const CLEAR_ALL_PETS_COMMAND: &'static str = "clear_all_pets";
    const CLEAR_ALL_FOODS_COMMAND: &'static str = "clear_all_foods";
    const CHANGE_LANGUAGE_COMMAND: &'static str = "change_language";
    const DISCOVER_COMPLETE_DIPDEX_COMMAND: &'static str = "discover_complete_dipdex";
    const SPAWN_SPEWER_COMMAND: &'static str = "spawn_spewer";
    const GIVE_MONEY_COMMAND: &'static str = "give_money";

    pub const fn command_str(&self) -> &'static str {
        match self {
            DevConsoleCommand::SetSimTimeScale(_) => Self::SET_SIM_TIME_SCALE_COMMAND,
            DevConsoleCommand::SpawnPet(_) => Self::SPAWN_PET_COMMAND,
            DevConsoleCommand::EvolvePet(_) => Self::EVOLVE_PET_COMMAND,
            DevConsoleCommand::SpawnFood(_) => Self::SPAWN_FOOD_COMMAND,
            DevConsoleCommand::SpawnPoop => Self::SPAWN_POOP_COMMAND,
            DevConsoleCommand::ClearAllPets => Self::CLEAR_ALL_PETS_COMMAND,
            DevConsoleCommand::ClearAllFoods => Self::CLEAR_ALL_FOODS_COMMAND,
            DevConsoleCommand::ChangeLanguage(_) => Self::CHANGE_LANGUAGE_COMMAND,
            DevConsoleCommand::DiscoverCompleteDipdex => Self::DISCOVER_COMPLETE_DIPDEX_COMMAND,
            DevConsoleCommand::SpawnSpewer => Self::SPAWN_SPEWER_COMMAND,
            DevConsoleCommand::GiveMoney(_) => Self::GIVE_MONEY_COMMAND,
        }
    }

    pub fn commands() -> Vec<String> {
        let mut result = Vec::new();
        for command in DevConsoleCommand::iter() {
            result.push(command.command_str().to_string());
        }
        result
    }
}

lazy_static! {
    static ref DEV_CONSOLE_COMMANDS: Vec<String> = DevConsoleCommand::commands();
}

fn action_dev_console_command(
    mut commands: Commands,
    pet_db: Option<Res<PetTemplateDatabase>>,
    mut dev_commands: EventReader<DevConsoleCommand>,
    mut spawn_pets: EventWriter<SpawnPetEvent>,
    mut spawn_food: EventWriter<SpawnFoodEvent>,
    mut sim_time_scale: ResMut<SimTimeScale>,
    mut rng: ResMut<GlobalRng>,
    mut history: Query<&mut DevConsoleHistory>,
    mut language: Query<&mut Language, With<SelectedLanguageTag>>,
    mut dipdex: Query<&mut DipdexDiscoveredEntries>,
    pets: Query<(Entity, &EntityName), With<Pet>>,
    foods: Query<Entity, With<Food>>,
    mut player_wallet: Query<&mut Wallet, With<Player>>,
) {
    let mut history = match history.get_single_mut() {
        Ok(history) => history,
        Err(_) => return,
    };

    for dev_command in dev_commands.read() {
        match dev_command {
            DevConsoleCommand::SetSimTimeScale(scale) => {
                sim_time_scale.0 = *scale;
                history.push_command_output(format!("Set sim time scale to {}", scale));
            }
            DevConsoleCommand::SpawnPet(name) => {
                spawn_pets.send(SpawnPetEvent::Blank((Vec2::new(0., 0.), name.clone())));
                history.push_command_output(format!("Spawned pet: {}", name));
            }
            DevConsoleCommand::EvolvePet(species_name) => {
                for (pet, name) in pets.iter() {
                    commands
                        .entity(pet)
                        .insert(ShouldEvolve::new(species_name.clone()));
                    history.push_command_output(format!("Evolving {} to: {}", name, species_name));
                }
            }
            DevConsoleCommand::SpawnFood(name) => {
                spawn_food.send(SpawnFoodEvent::new(name));
                history.push_command_output(format!("Spawned food: {}", name));
            }
            DevConsoleCommand::SpawnPoop => {
                let x = rng.i32(-200..200) as f32 + rng.f32();
                let y = rng.i32(-300..300) as f32 + rng.f32();
                spawn_poop(&mut commands, 1.0, Vec2::new(x, y), DEFAULT_POOP_TEXTURE);
                history.push_command_output(format!("Spawned poop at {},{}", x, y));
            }
            DevConsoleCommand::ClearAllPets => {
                for (pet, _) in pets.iter() {
                    commands.entity(pet).despawn_recursive();
                }
                history.push_command_output("Cleared all pets".to_string());
            }
            DevConsoleCommand::ClearAllFoods => {
                for food in foods.iter() {
                    commands.entity(food).despawn_recursive();
                }
                history.push_command_output("Cleared all foods".to_string());
            }
            DevConsoleCommand::ChangeLanguage(lang) => match Language::from_str(lang) {
                Ok(lang) => {
                    let mut selected_language = language.single_mut();
                    *selected_language = lang;
                    history.push_command_output(format!("Changed language to: {:?}", lang));
                }
                Err(_) => {
                    history.push_command_output(format!("Unknown language: {}", lang));
                }
            },
            DevConsoleCommand::DiscoverCompleteDipdex => {
                if let Some(pet_db) = &pet_db {
                    for mut dipdex in dipdex.iter_mut() {
                        for pet in pet_db.iter() {
                            dipdex.entries.insert(pet.species_name.clone());
                        }
                    }
                    history.push_command_output("Discovered all pets in dipdex");
                } else {
                    error!("Pet database not available");
                }
            }
            DevConsoleCommand::SpawnSpewer => {
                commands.spawn((
                    Transform::from_translation(Vec3::new(0., 0., 0.)),
                    SPARKS.clone(),
                ));
            }
            DevConsoleCommand::GiveMoney(amount) => {
                for mut wallet in player_wallet.iter_mut() {
                    wallet.balance += *amount;
                }
            }
        }
    }
}
