use bevy::prelude::*;
use strum::IntoEnumIterator;

use sardips_core::{
    assets::{DipdexImageAssets, FontAssets, ViewScreenImageAssets},
    button_hover::ButtonHover,
    food_core::FoodSensationRating,
    mood_core::{AutoSetMoodImage, MoodCategory, MoodImageIndexes, SatisfactionRating},
    name::SpeciesName,
    pet_core::{PetTemplate, PetTemplateDatabase},
    rotate_static::RotateStatic,
    text_database::TextDatabase,
    text_translation::KeyText,
    GameState,
};

use crate::{palettes, pet::dipdex::DipdexDiscoveredEntries, player::Player, pet_display::{spawn_pet_preview, PetPreview}};
use text_keys::{self, BACK};

pub struct DipdexScenePlugin;

impl Plugin for DipdexScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(DipdexState::default())
            .add_systems(
                OnEnter(GameState::DipdexView),
                (setup_state, setup_ui_roots, setup_camera),
            )
            .add_systems(OnExit(GameState::DipdexView), cleanup)
            .add_systems(OnEnter(DipdexState::Selecting), show_node::<DipdexPageNode>)
            .add_systems(OnExit(DipdexState::Selecting), hide_node::<DipdexPageNode>)
            .add_systems(
                Update,
                (
                    show_dex_page,
                    next_page_button,
                    exit_dipdex,
                    open_full_dip_view,
                )
                    .run_if(in_state(DipdexState::Selecting)),
            )
            .add_systems(
                OnEnter(DipdexState::Entry),
                (update_dipdex_entry_view, show_node::<DipdexEntryView>),
            )
            .add_systems(OnExit(DipdexState::Entry), hide_node::<DipdexEntryView>)
            .add_systems(
                Update,
                (
                    exit_entry_view,
                    update_entry_image,
                    disable_selected_mood_button,
                )
                    .run_if(in_state(DipdexState::Entry)),
            );
    }
}

pub const TITLE_SIZE: f32 = 50.;
pub const HEADER_SIZE: f32 = 40.;
pub const SUBHEADER_SIZE: f32 = 30.;
pub const SUB_SUBHEADER_SIZE: f32 = 25.;
pub const BODY_SIZE: f32 = 20.;

pub const KNOWN_BACKGROUND: Color = Color::srgb(0.737, 0.961, 0.737);

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum DipdexState {
    #[default]
    None,
    Selecting,
    Entry,
}

fn setup_state(mut commands: Commands, mut dipdex_state: ResMut<NextState<DipdexState>>) {
    commands.spawn((DexPage::default(), DipdexView));

    dipdex_state.set(DipdexState::Selecting);
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
        DipdexSelectCamera,
        DipdexView,
    ));
}

const PAGE_ENTRY_COUNT: usize = 5;

fn setup_ui_roots(mut commands: Commands, font_assets: Res<FontAssets>) {
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
            DipdexView,
            DipdexPageNode,
        ))
        .with_children(|parent| {
            parent
                .spawn((NodeBundle {
                    style: Style {
                        width: Val::Percent(80.0),
                        height: Val::Percent(80.0),
                        overflow: Overflow::clip_y(),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },))
                .with_children(|parent| {
                    for i in 0..5 {
                        parent.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Vw(100.),
                                    height: Val::Px(90.),
                                    margin: UiRect::new(
                                        Val::Px(10.),
                                        Val::Px(10.),
                                        Val::Px(0.),
                                        Val::Px(0.),
                                    ),
                                    border: UiRect::new(
                                        Val::Px(0.),
                                        Val::Px(0.),
                                        Val::Px(5.),
                                        Val::Px(5.),
                                    ),
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                border_color: BorderColor(palettes::dipdex_view::ENTRY_BACKGROUND),
                                background_color: BackgroundColor(
                                    palettes::dipdex_view::ENTRY_BORDER,
                                ),
                                ..default()
                            },
                            DexEntryNode { index: i },
                        ));
                    }
                });

            // Page buttons
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Px(50.),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    for i in 0..2 {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Percent(50.),
                                        height: Val::Percent(100.),
                                        ..default()
                                    },
                                    ..default()
                                },
                                ButtonHover::default()
                                    .with_background(palettes::dipdex_view::BUTTON_SET)
                                    .with_border(palettes::dipdex_view::BUTTON_BORDER_SET),
                                DexPageChangeButton(if i == 0 { -1 } else { 1 }),
                            ))
                            .with_children(|parent| {
                                parent.spawn((TextBundle {
                                    text: Text::from_section(
                                        if i == 0 { "<" } else { ">" },
                                        TextStyle {
                                            font_size: 40.0,
                                            color: Color::BLACK,
                                            font: font_assets.main_font.clone(),
                                        },
                                    ),
                                    ..default()
                                },));
                            });
                    }
                });

            // Back button
            spawn_back_button::<ExitDipdex>(parent, &font_assets);
        });

    // Entry view
    commands.spawn((
        NodeBundle {
            style: Style {
                display: Display::None,
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        DipdexView,
        DipdexEntryView { showing: None },
    ));
}

fn spawn_back_button<T: Component + Default>(parent: &mut ChildBuilder, font_assets: &FontAssets) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(200.),
                    height: Val::Px(50.),
                    margin: UiRect::top(Val::Px(20.)),
                    ..default()
                },
                ..default()
            },
            ButtonHover::default()
                .with_background(palettes::dipdex_view::BUTTON_SET)
                .with_border(palettes::dipdex_view::BUTTON_BORDER_SET),
            T::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::BLACK,
                            font: font_assets.main_font.clone(),
                        },
                    ),
                    ..default()
                },
                KeyText::new().with(0, BACK),
            ));
        });
}

fn make_dipdex_list_entry(
    builder: &mut ChildBuilder,
    font_assets: &FontAssets,
    dipdex_assets: &DipdexImageAssets,
    template: &PetTemplate,
    discovered: bool,
) {
    // Number
    builder.spawn(TextBundle {
        style: Style {
            width: Val::Px(70.),
            margin: UiRect::left(Val::Px(10.)),
            ..default()
        },
        text: Text::from_section(
            format!("{:0>3}", template.pre_calculated.number),
            TextStyle {
                font_size: TITLE_SIZE,
                color: Color::BLACK,
                font: font_assets.main_font.clone(),
            },
        ),
        ..default()
    });

    // Image
    builder
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(80.),
                height: Val::Px(80.),
                margin: UiRect::new(Val::Px(5.), Val::Px(5.), Val::Px(0.), Val::Px(0.)),
                border: UiRect::all(Val::Px(2.)),
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            border_color: BorderColor(Color::BLACK),
            background_color: BackgroundColor(KNOWN_BACKGROUND),
            ..default()
        })
        .with_children(|parent| {
            if discovered {
                spawn_pet_preview(
                    parent,
                    PetPreview::new(template.species_name.clone()),
                );
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
            } else {
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        ..default()
                    },
                    image: UiImage::new(dipdex_assets.unknown.clone()),
                    ..default()
                });
            }
        });

    // NAME HERE
    let mut entity = builder.spawn((TextBundle {
        style: Style {
            width: Val::Px(270.),
            ..default()
        },
        text: Text::from_section(
            "?????",
            TextStyle {
                font_size: HEADER_SIZE,
                color: Color::BLACK,
                font: font_assets.main_font.clone(),
            },
        ),
        ..default()
    },));

    if discovered {
        entity.insert(KeyText::new().with(0, SpeciesName::new(&template.species_name).name_key()));
    }

    let mut entity = builder.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(60.),
                height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                justify_self: JustifySelf::Center,
                align_self: AlignSelf::Center,
                ..default()
            },
            ..default()
        },
        ButtonHover::default()
            .with_background(palettes::dipdex_view::BUTTON_ENTRY_SET)
            .with_border(palettes::dipdex_view::BUTTON_ENTRY_BORDER_SET),
        DipdexEntryOpenButton(template.species_name.clone()),
    ));

    entity.with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text::from_section(
                "^",
                TextStyle {
                    font_size: 60.0,
                    color: Color::BLACK,
                    font: font_assets.main_font.clone(),
                },
            ),
            ..default()
        });
    });

    if !discovered {
        entity.remove::<Interaction>();
    }
}

fn cleanup(
    mut commands: Commands,
    mut state: ResMut<NextState<DipdexState>>,
    entities: Query<Entity, With<DipdexView>>,
) {
    for entity in entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    state.set(DipdexState::None);
}

#[derive(Component)]
struct DipdexSelectCamera;

#[derive(Component)]
struct DipdexView;

#[derive(Component)]
struct DipdexPageNode;

#[derive(Component, Default)]
struct ExitDipdex;

fn exit_dipdex(
    mut game_state: ResMut<NextState<GameState>>,
    mut dipdex_state: ResMut<NextState<DipdexState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<ExitDipdex>)>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            game_state.set(GameState::ViewScreen);
            dipdex_state.set(DipdexState::None);
        }
    }
}

#[derive(Component, Default)]
struct ExitEntryView;

fn exit_entry_view(
    mut dipdex_state: ResMut<NextState<DipdexState>>,
    query: Query<&Interaction, (Changed<Interaction>, With<ExitEntryView>)>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Pressed {
            dipdex_state.set(DipdexState::Selecting);
        }
    }
}

#[derive(Component)]
struct DexEntryNode {
    index: i32,
}

#[derive(Component, Default)]
struct DexPage {
    page: i32,
}

fn show_dex_page(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    dex_page: Query<&DexPage, Changed<DexPage>>,
    dex_entires: Query<(Entity, &DexEntryNode)>,
    pet_db: Res<PetTemplateDatabase>,
    dipdex_image: Res<DipdexImageAssets>,
    discovered: Query<&DipdexDiscoveredEntries, With<Player>>,
) {
    let page = match dex_page.get_single() {
        Ok(page) => page,
        Err(_) => return,
    };

    let discovered = discovered.single();

    let mut entires = dex_entires.iter().collect::<Vec<_>>();
    entires.sort_by_key(|(_, node)| node.index);

    let current_page = pet_db
        .iter()
        .skip(page.page as usize * 5)
        .take(5)
        .collect::<Vec<_>>();

    for (i, (entity, _)) in entires.iter().enumerate() {
        let mut entity = commands.entity(*entity);
        entity.despawn_descendants();
        if let Some(template) = current_page.get(i) {
            entity.with_children(|parent| {
                make_dipdex_list_entry(
                    parent,
                    &font_assets,
                    &dipdex_image,
                    template,
                    discovered.entries.contains(&template.species_name),
                );
            });
        }
    }
}

#[derive(Component)]
struct DexPageChangeButton(i32);

fn next_page_button(
    pet_db: Res<PetTemplateDatabase>,
    mut page: Query<&mut DexPage>,
    buttons: Query<(Entity, &DexPageChangeButton, &Interaction), Changed<Interaction>>,
) {
    let mut page = page.single_mut();

    for (_, button, interaction) in buttons.iter() {
        if *interaction == Interaction::Pressed {
            page.page += button.0;

            let max = (pet_db.iter().count() / PAGE_ENTRY_COUNT) as i32;
            if page.page < 0 {
                page.page = max
            }

            if page.page > max {
                page.page = 0
            }
        }
    }
}

#[derive(Component)]
struct DipdexEntryView {
    showing: Option<String>,
}

#[derive(Component)]
struct DipdexEntryOpenButton(String);

fn open_full_dip_view(
    mut state: ResMut<NextState<DipdexState>>,
    mut styles: Query<&mut Style>,
    page: Query<Entity, With<DipdexPageNode>>,
    mut entry_view: Query<(Entity, &mut DipdexEntryView)>,
    buttons: Query<(&Interaction, &DipdexEntryOpenButton), Changed<Interaction>>,
) {
    for (interaction, button) in buttons.iter() {
        if *interaction == Interaction::Pressed {
            state.set(DipdexState::Entry);
            {
                let entity = page.single();
                styles.get_mut(entity).unwrap().display = Display::None;
            }

            {
                let (entity, mut entry) = entry_view.single_mut();
                styles.get_mut(entity).unwrap().display = Display::Flex;
                entry.showing = Some(button.0.clone());
            }
        }
    }
}

fn update_dipdex_entry_view(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    pet_db: Res<PetTemplateDatabase>,
    view_screen_images: Res<ViewScreenImageAssets>,
    dipdex_assets: Res<DipdexImageAssets>,
    text_db: Res<TextDatabase>,
    view_assets: Res<ViewScreenImageAssets>,
    entry_view: Query<(Entity, &DipdexEntryView)>,
) {
    let (entity, entry) = entry_view.single();
    commands.entity(entity).despawn_descendants();

    let showing = entry.showing.as_ref().unwrap();
    let template = pet_db.get_by_name(showing).unwrap();

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            TextBundle::from_sections(vec![TextSection::new(
                "",
                TextStyle {
                    font_size: TITLE_SIZE,
                    color: Color::BLACK,
                    font: font_assets.main_font.clone(),
                },
            )]),
            KeyText::new().with(0, SpeciesName::new(&template.species_name).name_key()),
        ));

        // Mood selection panel
        {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(250.),
                        height: Val::Px(250.),
                        border: UiRect::all(Val::Px(5.)),
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(KNOWN_BACKGROUND),
                    border_color: BorderColor(Color::BLACK),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(80.),
                                height: Val::Percent(80.),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            // Make it fill 90% of width or height
                            spawn_pet_preview(
                                parent,
                                PetPreview::new(template.species_name.clone())
                                    .with_max_size(90.),
                            ).insert((
                                MoodImageIndexes::new(&template.image_set.column_mood_map),
                                PetEntryImage,
                                MoodCategory::default(),
                                AutoSetMoodImage,
                            ));
                        });

                    parent.spawn((
                        ImageBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(95.),
                                height: Val::Percent(95.),
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

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    const BUTTONS: [MoodButton; 5] = [
                        MoodButton(MoodCategory::Despairing),
                        MoodButton(MoodCategory::Sad),
                        MoodButton(MoodCategory::Neutral),
                        MoodButton(MoodCategory::Happy),
                        MoodButton(MoodCategory::Ecstatic),
                    ];

                    for button in BUTTONS {
                        let satisfaction: SatisfactionRating = button.0.into();

                        parent
                            .spawn((
                                ButtonBundle {
                                    style: Style {
                                        width: Val::Px(30.),
                                        height: Val::Px(30.),
                                        margin: UiRect::new(
                                            Val::Px(5.),
                                            Val::Px(5.),
                                            Val::Px(5.),
                                            Val::Px(5.),
                                        ),
                                        border: UiRect::new(
                                            Val::Px(5.),
                                            Val::Px(5.),
                                            Val::Px(5.),
                                            Val::Px(5.),
                                        ),
                                        ..default()
                                    },
                                    ..default()
                                },
                                ButtonHover::default()
                                    .with_background(palettes::dipdex_view::BUTTON_SET)
                                    .with_border(palettes::dipdex_view::BUTTON_BORDER_SET),
                                button,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    ImageBundle {
                                        image: UiImage::new(view_screen_images.moods.clone()),
                                        ..default()
                                    },
                                    TextureAtlas {
                                        layout: view_screen_images.moods_layout.clone(),
                                        index: satisfaction.atlas_index(),
                                    },
                                ));
                            });
                    }
                });
        }

        // Dipdex description
        {
            // Check if text exists
            let key = SpeciesName::new(&template.species_name).dipdex_description_key();
            let key = if text_db.exists(&key) {
                key
            } else {
                text_keys::DIPDEX_DESCRIPTION_DOES_NOT_EXIST.to_string()
            };

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: SUBHEADER_SIZE,
                        color: Color::BLACK,
                        font: font_assets.main_font.clone(),
                    },
                ),
                KeyText::new().with(0, text_keys::DIPDEX_DESCRIPTION_HEADER),
            ));

            parent.spawn((
                TextBundle {
                    style: Style {
                        width: Val::Percent(70.),
                        // justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font_size: BODY_SIZE,
                            color: Color::BLACK,
                            font: font_assets.main_font.clone(),
                        },
                    ),
                    ..default()
                },
                KeyText::new().with(0, key),
            ));
        }

        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font_size: SUBHEADER_SIZE,
                    color: Color::BLACK,
                    font: font_assets.main_font.clone(),
                },
            ),
            KeyText::new().with(0, text_keys::DIPDEX_STATS_HEADER),
        ));

        // Food sensation
        if let Some(stomach) = &template.stomach {
            let stomach_size = {
                let size_key = stomach.size.key();
                let title_key = text_keys::DIPDEX_STOMACH_SIZE;

                format!("~{title_key}~: ~{size_key}~")
            };
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: SUB_SUBHEADER_SIZE,
                        color: Color::BLACK,
                        font: font_assets.main_font.clone(),
                    },
                ),
                KeyText::new().with_format(0, stomach_size),
            ));

            if !stomach.sensations.is_empty() {
                parent
                    .spawn(NodeBundle {
                        style: Style {
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
                                    font_size: SUB_SUBHEADER_SIZE,
                                    color: Color::BLACK,
                                    font: font_assets.main_font.clone(),
                                },
                            )]),
                            KeyText::new().with(0, text_keys::DIPDEX_FOOD_SENSATION_TITLE),
                        ));

                        for rating in FoodSensationRating::iter() {
                            let mut sensations: Vec<_> = stomach
                                .sensations
                                .iter()
                                .filter(|(_, s)| **s == rating)
                                .map(|(t, _)| t)
                                .collect();
                            sensations.sort();

                            if sensations.is_empty() {
                                continue;
                            }

                            let rating_satisfaction: SatisfactionRating = rating.into();

                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        // align_items: AlignItems::Center,
                                        // justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        ImageBundle {
                                            style: Style {
                                                width: Val::Px(28.8),
                                                height: Val::Px(28.8),
                                                margin: UiRect::all(Val::Px(5.)),
                                                ..default()
                                            },
                                            image: UiImage::new(view_screen_images.moods.clone()),
                                            ..default()
                                        },
                                        TextureAtlas {
                                            layout: view_screen_images.moods_layout.clone(),
                                            index: rating_satisfaction.atlas_index(),
                                        },
                                    ));

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
                        }
                    });
            }
        }

        // Speed
        {
            let key: String = format!(
                "~{title}~: ~{speed}~",
                title = text_keys::DIPDEX_SPEED_TITLE,
                speed = template.speed.key()
            );

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: SUB_SUBHEADER_SIZE,
                        color: Color::BLACK,
                        font: font_assets.main_font.clone(),
                    },
                ),
                KeyText::new().with_format(0, key),
            ));
        }

        // Poop info
        {
            let mut key = format!("~{title}~: ", title = text_keys::DIPDEX_POOP_TITLE);
            key.push('~');
            if let Some(pooper) = &template.pooper {
                key.push_str(pooper.interval.key());
            } else {
                key.push_str(text_keys::DIPDEX_DOES_NOT_POOP);
            }
            key.push('~');

            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: SUB_SUBHEADER_SIZE,
                        color: Color::BLACK,
                        font: font_assets.main_font.clone(),
                    },
                ),
                KeyText::new().with_format(0, key),
            ));
        }

        // Cleanliness
        {
            parent.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font_size: SUB_SUBHEADER_SIZE,
                        color: Color::BLACK,
                        font: font_assets.main_font.clone(),
                    },
                ),
                KeyText::new().with(
                    0,
                    if template.cleanliness.is_none() {
                        text_keys::DIPDEX_CARES_ABOUT_CLEANLINESS
                    } else {
                        text_keys::DIPDEX_DOES_NOT_CARE_ABOUT_CLEANLINESS
                    },
                ),
            ));
        }

        // Back button
        spawn_back_button::<ExitEntryView>(parent, &font_assets);
    });
}

fn hide_node<T: Component>(mut view: Query<&mut Style, With<T>>) {
    if let Ok(mut style) = view.get_single_mut() {
        style.display = Display::None;
    }
}

fn show_node<T: Component>(mut view: Query<&mut Style, With<T>>) {
    let mut style = view.single_mut();

    style.display = Display::Flex;
}

#[derive(Component)]
struct PetEntryImage;

#[derive(Component)]
struct MoodButton(MoodCategory);

fn update_entry_image(
    buttons: Query<(Entity, &MoodButton, &Interaction), Changed<Interaction>>,
    mut pet_image: Query<&mut MoodCategory, With<PetEntryImage>>,
) {
    let mut mood = pet_image.single_mut();

    for (_, button, interaction) in buttons.iter() {
        if *interaction == Interaction::Pressed {
            *mood = button.0;
        }
    }
}

fn disable_selected_mood_button(
    mut commands: Commands,
    interactions: Query<&Interaction, With<MoodButton>>,
    buttons: Query<(Entity, &MoodButton)>,
    pet_image: Query<&MoodCategory, (With<PetEntryImage>, Changed<MoodCategory>)>,
) {
    let mood = match pet_image.get_single() {
        Ok(mood) => mood,
        Err(_) => return,
    };

    for (entity, button_mood) in buttons.iter() {
        if button_mood.0 == *mood {
            commands.entity(entity).remove::<Interaction>();
        } else if interactions.get(entity).is_err() {
            commands.entity(entity).insert(Interaction::default());
        }
    }
}
