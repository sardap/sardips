#![allow(clippy::needless_lifetimes)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(unexpected_cfgs)]
use std::cmp::Ordering;

use bevy::{prelude::*, render::view::RenderLayers};

use sardips_core::{
    assets::{self, FontAssets, HigherLowerAssets},
    autoscroll::AutoScroll,
    button_hover::{ButtonColorSet, ButtonHover},
    minigames_core::{
        MiniGameBackButton, MiniGameCompleted, MiniGameResult, MiniGameState, MiniGameType, Playing,
    },
    mood_core::{AutoSetMoodImage, MoodCategory, MoodImageIndexes},
    sounds::{PlaySoundEffect, SoundEffect},
};
use shared_deps::bevy_parallax::{
    CreateParallaxEvent, LayerComponent, LayerData, LayerSpeed, ParallaxCameraComponent,
};
use shared_deps::bevy_turborand::{DelegatedRng, GlobalRng};
use shared_deps::rand::{seq::SliceRandom, thread_rng};

pub struct HigherLowerPlugin;

impl Plugin for HigherLowerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(HigherLowerState::default())
            .add_event::<ChoiceEvent>()
            .add_systems(
                OnEnter(MiniGameState::PlayingHigherLower),
                (setup_camera, setup_game),
            )
            .add_systems(
                OnExit(MiniGameState::PlayingHigherLower),
                (send_complete, teardown).chain(),
            )
            .add_systems(OnEnter(HigherLowerState::GameOver), setup_game_over)
            .add_systems(
                Update,
                (button_pressed, handle_choice_event).run_if(in_state(HigherLowerState::Playing)),
            )
            .add_systems(
                Update,
                (update_display, update_buttons)
                    .run_if(in_state(MiniGameState::PlayingHigherLower)),
            );
    }
}

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Default)]
enum HigherLowerState {
    #[default]
    None,
    Playing,
    GameOver,
}

#[derive(Component)]
struct HigherLower;

#[derive(Component)]
struct Sign;

#[derive(Component)]
struct TargetNumber(u8);

#[derive(Component)]
struct GameState;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash, Default)]
enum GuessValue {
    #[default]
    None,
    Higher,
    Lower,
    Equal,
}

#[derive(Component, Default)]
struct Guess {
    value: GuessValue,
    count: i8,
}

#[derive(Component)]
struct HigherLowerPet;

#[derive(Component)]
struct Guessed;

fn setup_camera(mut commands: Commands, mut create_parallax: EventWriter<CreateParallaxEvent>) {
    let background_camera: Entity = commands
        .spawn((
            Camera2dBundle {
                camera: Camera {
                    order: 0,
                    ..default()
                },
                ..default()
            },
            ParallaxCameraComponent::default(),
            AutoScroll::new(Vec2::new(70., 0.)),
            HigherLower,
            RenderLayers::from_layers(&[0]),
        ))
        .id();

    create_parallax.send(CreateParallaxEvent {
        camera: background_camera,
        layers_data: vec![LayerData {
            speed: LayerSpeed::Horizontal(0.5),
            path: assets::HigherLowerAssets::BACKGROUND.to_string(),
            tile_size: UVec2::new(30, 30),
            cols: 1,
            rows: 1,
            scale: Vec2::splat(1.0),
            z: 0.0,
            ..Default::default()
        }],
    });

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        HigherLower,
        RenderLayers::from_layers(&[1]),
    ));
}

fn setup_game(
    mut commands: Commands,
    mut state: ResMut<NextState<HigherLowerState>>,
    mut rng: ResMut<GlobalRng>,
    fonts: Res<FontAssets>,
    assets: Res<HigherLowerAssets>,
    pet_sheet: Query<(&Handle<Image>, &TextureAtlas, &Sprite, &MoodImageIndexes), With<Playing>>,
) {
    state.set(HigherLowerState::Playing);

    commands.spawn((TargetNumber(rng.u8(0..10)), GameState, HigherLower));
    commands.spawn((Guess::default(), HigherLower));

    let (image, atlas, sprite, mood_images) = pet_sheet.single();

    commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_translation(Vec3::new(0., -50., 0.)),
                sprite: Sprite {
                    custom_size: sprite.custom_size,
                    ..default()
                },
                texture: image.clone(),
                ..default()
            },
            atlas.clone(),
            *mood_images,
            MoodCategory::Neutral,
            AutoSetMoodImage,
            RenderLayers::layer(1),
            HigherLowerPet,
            HigherLower,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    SpriteBundle {
                        transform: Transform::from_translation(Vec3::new(5., 140., -1.)),
                        texture: assets.sign.clone(),
                        ..default()
                    },
                    RenderLayers::layer(1),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        SpriteBundle {
                            transform: Transform::from_translation(Vec3::new(0., 60., 2.)),
                            texture: assets.sprites.clone(),
                            ..default()
                        },
                        TextureAtlas {
                            layout: assets.layout.clone(),
                            ..default()
                        },
                        RenderLayers::layer(1),
                        Sign,
                    ));
                });
        });

    // setup UI
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            HigherLower,
        ))
        .with_children(|parent| {
            const MAX_NUM: u8 = 15;

            let mut numbers = Vec::new();
            for number in 0..MAX_NUM {
                numbers.push(number);
            }

            numbers.shuffle(&mut thread_rng());

            for number in numbers {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Percent(9.0),
                                height: Val::Px(50.0),
                                border: UiRect::all(Val::Percent(0.5)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        },
                        ButtonHover::default()
                            .with_background(
                                ButtonColorSet::new(
                                    Color::Srgba(bevy::color::palettes::css::BEIGE),
                                    Color::WHITE,
                                    Color::Srgba(bevy::color::palettes::css::BEIGE),
                                )
                                .with_disabled(Color::Srgba(bevy::color::palettes::css::GRAY)),
                            )
                            .with_border(
                                ButtonColorSet::new(Color::BLACK, Color::WHITE, Color::WHITE)
                                    .with_disabled(Color::Srgba(bevy::color::palettes::css::GRAY)),
                            ),
                        TargetNumber(number),
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                number.to_string(),
                                TextStyle {
                                    font: fonts.main_font.clone(),
                                    color: Color::BLACK,
                                    font_size: 30.,
                                },
                            ),
                            ..default()
                        });
                    });
            }
        });
}

fn teardown(
    mut commands: Commands,
    mut state: ResMut<NextState<HigherLowerState>>,
    to_delete: Query<Entity, Or<(With<HigherLower>, With<LayerComponent>)>>,
) {
    state.set(HigherLowerState::None);

    for entity in to_delete.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn button_pressed(
    mut commands: Commands,
    mut events: EventWriter<ChoiceEvent>,
    mut buttons: Query<(Entity, &TargetNumber, &Interaction), Changed<Interaction>>,
) {
    for (entity, option, interaction) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        commands.entity(entity).remove::<Interaction>();
        commands.entity(entity).insert(Guessed);

        events.send(ChoiceEvent(option.0));
    }
}

fn update_buttons(
    target: Query<&TargetNumber, With<GameState>>,
    mut buttons: Query<
        (
            &TargetNumber,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&Guessed>,
        ),
        Without<Interaction>,
    >,
) {
    let goal = target.single();

    for (target, mut background, mut border, guessed) in buttons.iter_mut() {
        // Should be grey if never guessed
        if target.0 == goal.0 {
            *background = BackgroundColor(Color::Srgba(bevy::color::palettes::css::GREEN));
            *border = BorderColor(Color::Srgba(bevy::color::palettes::css::DARK_GREEN));
        } else if guessed.is_some() {
            *background = BackgroundColor(Color::Srgba(bevy::color::palettes::css::RED));
            *border = BorderColor(Color::Srgba(bevy::color::palettes::css::ORANGE_RED));
        } else {
            *background = BackgroundColor(Color::Srgba(bevy::color::palettes::css::GRAY));
            *border = BorderColor(Color::WHITE);
        }
    }
}

#[derive(Event, Deref)]
struct ChoiceEvent(u8);

const GUESS_COUNT: i8 = 3;

fn handle_choice_event(
    mut state: ResMut<NextState<HigherLowerState>>,
    mut events: EventReader<ChoiceEvent>,
    mut sounds: EventWriter<PlaySoundEffect>,
    mut guess: Query<&mut Guess>,
    target_number: Query<&TargetNumber, With<GameState>>,
) {
    let target = target_number.single();
    let mut guess = guess.single_mut();

    for event in events.read() {
        guess.value = match target.0.cmp(&event.0) {
            Ordering::Less => {
                sounds.send(PlaySoundEffect::new(SoundEffect::Lower));
                GuessValue::Lower
            }
            Ordering::Greater => {
                sounds.send(PlaySoundEffect::new(SoundEffect::Higher));
                GuessValue::Higher
            }
            Ordering::Equal => {
                sounds.send(PlaySoundEffect::new(SoundEffect::Correct));
                GuessValue::Equal
            }
        };

        guess.count += 1;
        if guess.value == GuessValue::Equal || guess.count >= GUESS_COUNT {
            state.set(HigherLowerState::GameOver);
        }
    }
}

fn update_display(
    mut mood: Query<&mut MoodCategory, With<HigherLowerPet>>,
    mut sign: Query<&mut TextureAtlas, With<Sign>>,
    guess: Query<&Guess>,
) {
    let mut mood = mood.single_mut();
    let guess = guess.single();

    *mood = match guess.value {
        GuessValue::None => MoodCategory::Neutral,
        GuessValue::Higher | GuessValue::Lower => match GUESS_COUNT - guess.count {
            0 => MoodCategory::Despairing,
            1 | 2 => MoodCategory::Sad,
            _ => MoodCategory::Neutral,
        },
        GuessValue::Equal => MoodCategory::Ecstatic,
    };

    let mut sign = sign.single_mut();
    sign.index = if GUESS_COUNT - guess.count == 0 {
        if guess.value == GuessValue::Equal {
            3
        } else {
            4
        }
    } else {
        match guess.value {
            GuessValue::None => 0,
            GuessValue::Higher => 1,
            GuessValue::Lower => 2,
            GuessValue::Equal => 3,
        }
    };
}

fn setup_game_over(
    mut commands: Commands,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
    mut buttons: Query<Entity, (With<TargetNumber>, With<Interaction>)>,
    layers: Query<Entity, With<LayerComponent>>,
    camera: Query<
        Entity,
        (
            With<ParallaxCameraComponent>,
            With<Camera>,
            With<HigherLower>,
        ),
    >,
    guess: Query<&Guess>,
) {
    for entity in buttons.iter_mut() {
        commands.entity(entity).remove::<Interaction>();
    }

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    top: Val::Px(0.01),
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            HigherLower,
        ))
        .with_children(|parent| {
            parent.spawn(MiniGameBackButton);
        });

    for entity in layers.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let guess = guess.single();

    create_parallax.send(CreateParallaxEvent {
        camera: camera.single(),
        layers_data: vec![LayerData {
            speed: LayerSpeed::Vertical(0.5),
            path: if guess.value == GuessValue::Equal {
                assets::HigherLowerAssets::WIN_BACKGROUND.to_string()
            } else {
                assets::HigherLowerAssets::LOSE_BACKGROUND.to_string()
            },
            tile_size: UVec2::new(30, 30),
            cols: 1,
            rows: 1,
            scale: Vec2::splat(1.0),
            z: 0.0,
            ..Default::default()
        }],
    });
}

fn send_complete(mut event_writer: EventWriter<MiniGameCompleted>, guess: Query<&Guess>) {
    let guess = guess.single();

    event_writer.send(MiniGameCompleted {
        game_type: MiniGameType::HigherLower,
        result: match guess.value {
            GuessValue::None => MiniGameResult::Incomplete,
            GuessValue::Higher | GuessValue::Lower => MiniGameResult::Lose,
            GuessValue::Equal => MiniGameResult::Win,
        },
    });
}
