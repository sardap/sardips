use bevy::prelude::*;

use crate::{
    age::Age,
    assets::{FontAssets, ViewScreenImageAssets},
    food::{Food, FoodFillFactor, FoodSensations},
    name::{HasNameTag, NameTag, SpeciesName},
    palettes,
    pet::{
        mood::{Mood, MoodCategory, SatisfactionRating},
        Pet,
    },
    text_database::text_keys::{
        UI_PET_INFO_PANEL_AGE, UI_PET_INFO_PANEL_SPECIES, UI_PET_PANEL_NO_THOUGHT,
    },
    text_translation::KeyText,
    thinking::Thought,
    GameState,
};

pub struct InfoPanelPlugin;

impl Plugin for InfoPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InfoPanelUpdate>();
        app.add_event::<InfoPanelsClear>();
        app.add_systems(
            Update,
            (
                clear_panels,
                update_info_panel,
                pet_panel_selection_valid,
                (
                    update_food_panel_sensation_text,
                    update_food_panel_visibility,
                    update_food_panel_fill_factor,
                    update_pet_panel_visibility,
                    update_pet_panel_species,
                    update_pet_panel_hunger_mood,
                    update_pet_panel_cleanliness_mood,
                    update_pet_panel_fun_mood,
                    update_pet_thought,
                    update_pet_age,
                    update_overall_mood,
                ),
            )
                .chain()
                .run_if(in_state(GameState::ViewScreen)),
        );
    }
}

#[derive(Component)]
pub struct InfoPanel;

#[derive(Event)]
pub struct InfoPanelsClear;

fn clear_panels(
    mut info_panels_clear: EventReader<InfoPanelsClear>,
    mut panels: Query<&mut Visibility, With<InfoPanel>>,
) {
    if info_panels_clear.is_empty() {
        return;
    }
    info_panels_clear.clear();

    for mut panel in panels.iter_mut() {
        *panel = Visibility::Hidden;
    }
}

#[derive(Event)]
pub struct InfoPanelUpdate {
    pub entity: Entity,
    pub panel_type: PanelType,
}

pub enum PanelType {
    Pet,
    Food,
}

const INFO_PANEL_TEXT_SIZE: f32 = 25.0;

#[derive(Component)]
pub struct RootInfoPanel;

#[derive(Component)]
pub struct PetInfoPanel {
    target: Option<Entity>,
}

#[derive(Component)]
pub struct FoodInfoPanel {
    target: Option<Entity>,
}

#[derive(Component)]
struct FoodInfoPanelSensation;

#[derive(Component)]
struct FoodInfoPanelFillFactor;

#[derive(Component)]
struct PetInfoPanelSpecies;

#[derive(Component)]
struct PetInfoPanelThought;

#[derive(Component)]
struct PetInfoPanelAgeText;

#[derive(Component, Default)]
struct PetInfoPanelOverallMood;

#[derive(Component, Default)]
struct PetInfoPanelOverallMoodImage;

#[derive(Component, Default)]
struct PetInfoPanelMoodHunger;

#[derive(Component, Default)]
struct PetInfoPanelMoodHungerImage;

#[derive(Component, Default)]
struct PetInfoPanelMoodCleanliness;

#[derive(Component, Default)]
struct PetInfoPanelMoodCleanlinessImage;

#[derive(Component, Default)]
struct PetInfoPanelMoodFun;

#[derive(Component, Default)]
struct PetInfoPanelMoodFunImage;

pub fn create_info_panel(
    commands: &mut Commands,
    fonts: &FontAssets,
    view_screen_images: &ViewScreenImageAssets,
) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    top: Val::Px(0.),
                    width: Val::Vw(100.),
                    height: Val::Px(70.),
                    position_type: PositionType::Absolute,
                    border: UiRect::bottom(Val::Px(7.)),
                    ..default()
                },
                border_color: BorderColor(palettes::view_screen::STATUS_BAR_BORDER),
                background_color: BackgroundColor(palettes::view_screen::STATUS_BAR),
                visibility: Visibility::Hidden,
                ..default()
            },
            RootInfoPanel,
            InfoPanel,
        ))
        .with_children(|parent| {
            let style = Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.),
                margin: UiRect::all(Val::Px(10.)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            };

            let child_element_style = Style {
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::new(Val::Px(5.), Val::Px(5.), Val::Px(0.), Val::Px(0.)),
                ..default()
            };

            parent
                .spawn((
                    NodeBundle {
                        style: style.clone(),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    PetInfoPanel { target: None },
                    InfoPanel,
                ))
                .with_children(|parent| {
                    // Top row
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Vw(100.),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: child_element_style.clone(),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_sections(vec![TextSection::new(
                                            "",
                                            TextStyle {
                                                font: fonts.main_font.clone(),
                                                font_size: INFO_PANEL_TEXT_SIZE,
                                                color: Color::BLACK,
                                                ..default()
                                            },
                                        )]),
                                        KeyText::new(),
                                        Label,
                                        PetInfoPanelSpecies,
                                    ));
                                });

                            parent
                                .spawn(NodeBundle {
                                    style: child_element_style.clone(),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        TextBundle::from_sections(vec![
                                            TextSection::new(
                                                "",
                                                TextStyle {
                                                    font: fonts.main_font.clone(),
                                                    font_size: INFO_PANEL_TEXT_SIZE,
                                                    color: Color::BLACK,
                                                    ..default()
                                                },
                                            ),
                                            TextSection::new(
                                                "",
                                                TextStyle {
                                                    font: fonts.main_font.clone(),
                                                    font_size: INFO_PANEL_TEXT_SIZE,
                                                    color: Color::BLACK,
                                                    ..default()
                                                },
                                            ),
                                        ]),
                                        KeyText::new().with(0, UI_PET_INFO_PANEL_AGE),
                                        PetInfoPanelAgeText,
                                    ));
                                });

                            spawn_panel::<PetInfoPanelOverallMood, PetInfoPanelOverallMoodImage>(
                                parent,
                                &child_element_style,
                                view_screen_images,
                                0,
                            );

                            spawn_panel::<PetInfoPanelMoodHunger, PetInfoPanelMoodHungerImage>(
                                parent,
                                &child_element_style,
                                view_screen_images,
                                1,
                            );

                            spawn_panel::<
                                PetInfoPanelMoodCleanliness,
                                PetInfoPanelMoodCleanlinessImage,
                            >(
                                parent, &child_element_style, view_screen_images, 2
                            );

                            spawn_panel::<PetInfoPanelMoodFun, PetInfoPanelMoodFunImage>(
                                parent,
                                &child_element_style,
                                view_screen_images,
                                3,
                            );
                        });

                    parent.spawn((
                        TextBundle::from_section(
                            "",
                            TextStyle {
                                font: fonts.main_font.clone(),
                                font_size: INFO_PANEL_TEXT_SIZE,
                                color: Color::BLACK,
                                ..default()
                            },
                        ),
                        KeyText::new().with(0, UI_PET_PANEL_NO_THOUGHT),
                        PetInfoPanelThought,
                    ));
                });

            parent
                .spawn((
                    NodeBundle {
                        style: style.clone(),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    FoodInfoPanel { target: None },
                    InfoPanel,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: child_element_style.clone(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "Sensations:",
                                    TextStyle {
                                        font: fonts.main_font.clone(),
                                        font_size: INFO_PANEL_TEXT_SIZE,
                                        color: Color::BLACK,
                                        ..default()
                                    },
                                ),
                                Label,
                                FoodInfoPanelSensation,
                            ));
                        });

                    parent
                        .spawn(NodeBundle {
                            style: child_element_style.clone(),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "Fill Factor:",
                                    TextStyle {
                                        font: fonts.main_font.clone(),
                                        font_size: INFO_PANEL_TEXT_SIZE,
                                        color: Color::BLACK,
                                        ..default()
                                    },
                                ),
                                Label,
                                FoodInfoPanelFillFactor,
                            ));
                        });
                });
        })
        .id()
}

fn spawn_panel<T: Component + Default, J: Component + Default>(
    parent: &mut ChildBuilder,
    child_element_style: &Style,
    view_screen_images: &ViewScreenImageAssets,
    icon_index: usize,
) {
    let image_style = Style {
        width: Val::Px(20.0),
        margin: UiRect::all(Val::Px(5.0)),
        ..default()
    };

    parent
        .spawn((
            NodeBundle {
                style: child_element_style.clone(),
                ..default()
            },
            T::default(),
        ))
        .with_children(|parent| {
            parent.spawn(AtlasImageBundle {
                style: image_style.clone(),
                texture_atlas: TextureAtlas {
                    layout: view_screen_images.mood_icons_layout.clone(),
                    index: icon_index,
                },
                image: UiImage::new(view_screen_images.mood_icons.clone()),
                ..default()
            });

            parent.spawn((
                AtlasImageBundle {
                    style: image_style.clone(),
                    texture_atlas: TextureAtlas {
                        layout: view_screen_images.moods_layout.clone(),
                        ..default()
                    },
                    image: UiImage::new(view_screen_images.moods.clone()),
                    ..default()
                },
                J::default(),
            ));
        });
}

fn update_info_panel(
    mut info_panel_update: EventReader<InfoPanelUpdate>,
    has_name_tag: Query<&HasNameTag>,
    mut name_tags: Query<(Entity, &mut NameTag)>,
    mut info_panel: Query<&mut Visibility, With<RootInfoPanel>>,
    mut food_info_panel: Query<&mut FoodInfoPanel>,
    mut pet_info_panel: Query<&mut PetInfoPanel>,
) {
    let mut info_panel_vis = info_panel.single_mut();
    let mut food_info_panel = food_info_panel.single_mut();
    let mut pet_info_panel = pet_info_panel.single_mut();

    if let Some(event) = info_panel_update.read().next() {
        // Update name tags
        if let Ok(has_name_tag) = has_name_tag.get(event.entity) {
            if let Ok((_, mut text)) = name_tags.get_mut(has_name_tag.name_tag_entity) {
                text.color = Color::RED;
            }
        }

        *info_panel_vis = Visibility::Visible;

        match event.panel_type {
            PanelType::Pet => {
                pet_info_panel.target = Some(event.entity);
                food_info_panel.target = None;
            }
            PanelType::Food => {
                food_info_panel.target = Some(event.entity);
                pet_info_panel.target = None;
            }
        }
    }

    info_panel_update.clear();
}

fn pet_panel_selection_valid(
    mut pet_info_panel: Query<&mut PetInfoPanel>,
    pets: Query<Entity, With<Pet>>,
) {
    let mut panel = match pet_info_panel.get_single_mut() {
        Ok(val) => val,
        Err(_) => return,
    };

    if let Some(target) = panel.target {
        if pets.get(target).is_err() {
            panel.target = None;
        }
    }
}

fn update_food_panel_visibility(
    mut food_info_panel: Query<(&mut Visibility, &FoodInfoPanel), Changed<FoodInfoPanel>>,
) {
    let (mut visibility, panel) = match food_info_panel.get_single_mut() {
        Ok(val) => val,
        Err(_) => return,
    };
    *visibility = if panel.target.is_some() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn update_food_panel_sensation_text(
    fonts: Res<FontAssets>,
    food_info_panel: Query<&FoodInfoPanel, Changed<FoodInfoPanel>>,
    foods: Query<&FoodSensations, With<Food>>,
    mut text: Query<&mut Text, With<FoodInfoPanelSensation>>,
) {
    let food_info_panel = match food_info_panel.get_single() {
        Ok(val) => val,
        Err(_) => return,
    };

    let food_entity = match food_info_panel.target {
        Some(entity) => entity,
        None => return,
    };

    let mut text = text.single_mut();

    let food_sensations = foods.get(food_entity).unwrap();

    // Handle food clicked
    while text.sections.len() > 1 {
        text.sections.pop();
    }

    let font_size = text.sections[0].style.font_size;

    let mut sensation_text = " ".to_string();

    for (i, sensation) in food_sensations.values.iter().enumerate() {
        // Get first two letters of sensation
        sensation_text.push_str(sensation.short_string());
        if i < food_sensations.values.len() - 1 {
            sensation_text.push_str(", ");
        }
    }

    text.sections.push(TextSection {
        value: sensation_text,
        style: TextStyle {
            font: fonts.main_font.clone(),
            font_size,
            color: Color::BLACK,
        },
    });
}

fn update_food_panel_fill_factor(
    fonts: Res<FontAssets>,
    food_info_panel: Query<&FoodInfoPanel, Changed<FoodInfoPanel>>,
    foods: Query<&FoodFillFactor, With<Food>>,
    mut text: Query<&mut Text, With<FoodInfoPanelFillFactor>>,
) {
    let food_info_panel = match food_info_panel.get_single() {
        Ok(val) => val,
        Err(_) => return,
    };

    let food_entity = match food_info_panel.target {
        Some(entity) => entity,
        None => return,
    };

    let mut text = text.single_mut();

    let fill_factor = foods.get(food_entity).unwrap();

    while text.sections.len() > 1 {
        text.sections.pop();
    }

    let font_size = text.sections[0].style.font_size;

    text.sections.push(TextSection {
        value: format!(" {:.0}", fill_factor.0.round()),
        style: TextStyle {
            font: fonts.main_font.clone(),
            font_size,
            color: Color::BLACK,
        },
    });
}

fn update_pet_panel_visibility(
    mut pet_info_panel: Query<(&mut Visibility, &PetInfoPanel), Changed<PetInfoPanel>>,
) {
    let (mut visibility, panel) = match pet_info_panel.get_single_mut() {
        Ok(val) => val,
        Err(_) => return,
    };

    *visibility = if panel.target.is_some() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn update_pet_panel_species(
    pet_info_panel: Query<&PetInfoPanel, Changed<PetInfoPanel>>,
    pets: Query<&SpeciesName, With<Pet>>,
    mut text: Query<&mut KeyText, With<PetInfoPanelSpecies>>,
) {
    let pet_info_panel = match pet_info_panel.get_single() {
        Ok(val) => val,
        Err(_) => return,
    };

    let pet_entity = match pet_info_panel.target {
        Some(entity) => entity,
        None => return,
    };

    let mut text = text.single_mut();

    let species_name = pets.get(pet_entity).unwrap();

    text.set(0, species_name.key());
}

macro_rules! update_pet_panel_mood {
    ($function_name:ident, $mood_field:ident, $node:ty, $layout:ty) => {
        fn $function_name(
            pet_info_panel: Query<&PetInfoPanel>,
            pets: Query<&Mood, With<Pet>>,
            mut node: Query<&mut Style, With<$node>>,
            mut layouts: Query<&mut TextureAtlas, With<$layout>>,
        ) {
            let pet_info_panel = match pet_info_panel.get_single() {
                Ok(val) => val,
                Err(_) => return,
            };

            let pet_entity = match pet_info_panel.target {
                Some(entity) => entity,
                None => return,
            };

            let mut node_style = match node.get_single_mut() {
                Ok(val) => val,
                Err(_) => return,
            };

            let mut atlas = layouts.single_mut();

            let mood = pets.get(pet_entity).unwrap();

            if let Some(mood) = &mood.$mood_field {
                node_style.display = Display::DEFAULT;
                atlas.index = mood.satisfaction.atlas_index();
            } else {
                node_style.display = Display::None;
            }
        }
    };
}

fn update_overall_mood(
    pet_info_panel: Query<&PetInfoPanel>,
    pets: Query<&MoodCategory, With<Pet>>,
    mut layouts: Query<&mut TextureAtlas, With<PetInfoPanelOverallMoodImage>>,
) {
    let pet_info_panel = match pet_info_panel.get_single() {
        Ok(val) => val,
        Err(_) => return,
    };

    let pet_entity = match pet_info_panel.target {
        Some(entity) => entity,
        None => return,
    };

    let mut atlas = layouts.single_mut();

    let mood_category = pets.get(pet_entity).unwrap();

    let satisfaction: SatisfactionRating = (*mood_category).into();

    atlas.index = satisfaction.atlas_index();
}

update_pet_panel_mood!(
    update_pet_panel_hunger_mood,
    hunger,
    PetInfoPanelMoodHunger,
    PetInfoPanelMoodHungerImage
);
update_pet_panel_mood!(
    update_pet_panel_cleanliness_mood,
    cleanliness,
    PetInfoPanelMoodCleanliness,
    PetInfoPanelMoodCleanlinessImage
);
update_pet_panel_mood!(
    update_pet_panel_fun_mood,
    fun,
    PetInfoPanelMoodFun,
    PetInfoPanelMoodFunImage
);

fn update_pet_thought(
    pet_info_panel: Query<&PetInfoPanel, Changed<PetInfoPanel>>,
    pets: Query<&Thought, With<Pet>>,
    mut text: Query<&mut KeyText, With<PetInfoPanelThought>>,
) {
    let pet_info_panel = match pet_info_panel.get_single() {
        Ok(val) => val,
        Err(_) => return,
    };

    let pet_entity = match pet_info_panel.target {
        Some(entity) => entity,
        None => return,
    };

    let mut text = text.single_mut();

    let thought = pets.get(pet_entity).unwrap();

    let next_text = match &thought.text {
        Some(thought) => thought,
        None => UI_PET_PANEL_NO_THOUGHT,
    }
    .to_string();

    if text.keys[&0] != next_text {
        text.keys.insert(0, next_text);
    }
}

fn update_pet_age(
    pet_info_panel: Query<&PetInfoPanel>,
    pets: Query<&Age, With<Pet>>,
    mut text: Query<&mut Text, With<PetInfoPanelAgeText>>,
) {
    let pet_info_panel = match pet_info_panel.get_single() {
        Ok(val) => val,
        Err(_) => return,
    };

    let pet_entity = match pet_info_panel.target {
        Some(entity) => entity,
        None => return,
    };

    let mut text = text.single_mut();

    let age = pets.get(pet_entity).unwrap();

    // Hours lived
    text.sections[1].value = age.lived_for_text();
}
