use std::collections::HashMap;

use bevy::{
    asset,
    ecs::system::EntityCommands,
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};
use sardips_core::{
    accessory_core::{self, AccessoryTemplate, AccessoryTemplateDatabase},
    pet_core::PetTemplateDatabase,
    sprite_utils::get_adjusted_size,
};

use crate::{
    accessory::{Accessory, Wearer},
    layering,
};

pub struct PetPreviewPlugin;

impl Plugin for PetPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (setup_sardip_display, cleanup_render_objects)
                .run_if(resource_exists::<PetTemplateDatabase>),
        );
    }
}

pub fn spawn_pet_preview<'a>(
    builder: &'a mut ChildBuilder<'_>,
    display: PetPreview,
) -> EntityCommands<'a> {
    builder.spawn((
        ImageBundle {
            style: Style {
                width: Val::Percent(display.max_size),
                height: Val::Percent(display.max_size),
                ..default()
            },
            ..default()
        },
        display,
    ))
}

#[derive(Component)]
pub struct PetPreview {
    pub pet_template: String,
    pub max_size: f32,
    pub accessory: Vec<Accessory>,
}

impl PetPreview {
    pub fn new(pet_template: String) -> Self {
        Self {
            pet_template,
            max_size: 60.0,
            accessory: Vec::new(),
        }
    }

    pub fn with_max_size(mut self, max_size: f32) -> Self {
        self.max_size = max_size;
        self
    }

    pub fn replace_accessory(&mut self, accessory: Accessory) {
        self.accessory.clear();
        self.accessory.push(accessory);
    }

    pub fn clear_accessory(&mut self) {
        self.accessory.clear();
    }
}

#[derive(Component)]
struct PetDisplayHolder {
    own: Vec<Entity>,
    layer: usize,
}

#[derive(Component)]
struct PartOfPreviewRender {
    owner: Entity,
}

impl PartOfPreviewRender {
    pub fn new(owner: Entity) -> Self {
        Self { owner }
    }
}

#[derive(Component)]
struct AccessoryDisplay;

struct LocalSetup {
    last_layer: u32,
}

impl Default for LocalSetup {
    fn default() -> Self {
        Self {
            last_layer: layering::game_layers::PET_PREVIEW.start,
        }
    }
}

impl LocalSetup {
    fn get_next_layer(&mut self) -> usize {
        self.last_layer += 1;
        if self.last_layer > layering::game_layers::PET_PREVIEW.end {
            self.last_layer = layering::game_layers::PET_PREVIEW.start;
        }
        self.last_layer as usize
    }
}

fn setup_sardip_display(
    mut local: Local<LocalSetup>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pet_db: Res<PetTemplateDatabase>,
    accessory_db: Res<AccessoryTemplateDatabase>,
    mut images: ResMut<Assets<Image>>,
    mut updated: Query<
        (Entity, &PetPreview, &mut UiImage, Option<&PetDisplayHolder>),
        Or<(Changed<PetPreview>, Added<PetPreview>)>,
    >,
) {
    for (entity, preview, mut ui_image, display_holder) in &mut updated {
        let pet_template = &preview.pet_template;
        let pet_template = match pet_db.get_by_name(&pet_template) {
            Some(template) => template,
            None => {
                error!("Failed to find pet template: {}", pet_template);
                continue;
            }
        };

        info!("Spawning pet view");

        if let Some(display_holder) = display_holder {
            for entity in &display_holder.own {
                commands.entity(*entity).despawn_recursive();
            }
        }

        // This is the texture that will be rendered to.
        let size = Extent3d {
            width: (pet_template.pre_calculated.custom_size.x * 1.3) as u32 + 50,
            height: (pet_template.pre_calculated.custom_size.y * 1.3) as u32 + 50,
            ..default()
        };

        let mut render_image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0],
            TextureFormat::Bgra8UnormSrgb,
            RenderAssetUsages::default(),
        );

        render_image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT;

        let image_handle = images.add(render_image);

        ui_image.texture = image_handle.clone();

        let render_layer = local.get_next_layer();

        let mut new_entities = Vec::new();

        new_entities.push(
            commands
                .spawn((
                    RenderLayers::layer(render_layer),
                    PartOfPreviewRender::new(entity),
                    Camera2dBundle {
                        camera: Camera {
                            target: image_handle.clone().into(),
                            clear_color: Color::NONE.into(),
                            ..default()
                        },
                        ..default()
                    },
                ))
                .id(),
        );

        new_entities.push(
            commands
                .spawn((
                    RenderLayers::layer(render_layer),
                    PartOfPreviewRender::new(entity),
                    SpriteBundle {
                        transform: Transform::from_xyz(0., 0., 0.),
                        sprite: Sprite {
                            custom_size: Some(pet_template.pre_calculated.custom_size),
                            ..default()
                        },
                        texture: asset_server.load(pet_template.image_set.sprite_sheet.clone()),
                        ..default()
                    },
                    TextureAtlas {
                        layout: pet_template.pre_calculated.layout.clone(),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    let wearer = Wearer {
                        size: &pet_template.pre_calculated.custom_size,
                        anchor_points: &pet_template.anchor_points,
                    };

                    for accessory in &preview.accessory {
                        let accessory_template = match accessory_db.get(&accessory.template) {
                            Some(accessory) => accessory,
                            None => continue,
                        };
                        let size = match accessory_template.wear_size {
                            accessory_core::AccessorySize::StretchX => {
                                get_adjusted_size(wearer.size.x, accessory_template.texture_size)
                            }
                            accessory_core::AccessorySize::StretchY => {
                                get_adjusted_size(wearer.size.y, accessory_template.texture_size)
                            }
                            accessory_core::AccessorySize::Constant(size) => size,
                        };
                        let point = wearer
                            .anchor_points
                            .get(accessory_template.anchor_point, wearer.size)
                            + accessory_template.anchor_offset;

                        parent
                            .spawn((
                                RenderLayers::layer(render_layer),
                                SpriteBundle {
                                    sprite: Sprite {
                                        custom_size: Some(size),
                                        color: accessory.tint,
                                        ..default()
                                    },
                                    transform: Transform::from_xyz(
                                        point.x,
                                        point.y,
                                        0. + accessory_template.layer.z(),
                                    ),
                                    texture: asset_server.load(&accessory_template.texture),
                                    ..default()
                                },
                            ))
                            .with_children(|parent| {
                                for spewer in accessory_template
                                    .spewers
                                    .iter()
                                    .chain(&accessory.extra_spewers)
                                {
                                    parent.spawn((
                                        Transform::from_xyz(0., 0., 5.),
                                        GlobalTransform::default(),
                                        spewer
                                            .clone()
                                            .with_spawn_area(Rect::new(
                                                -size.x / 2.,
                                                -size.y / 2.,
                                                size.x / 2.,
                                                size.y / 2.,
                                            ))
                                            .with_render_layer(render_layer),
                                    ));
                                }
                            });
                    }
                })
                .id(),
        );

        commands.entity(entity).insert(PetDisplayHolder {
            own: new_entities,
            layer: render_layer,
        });
    }
}

fn cleanup_render_objects(
    mut commands: Commands,
    mut removed: RemovedComponents<PetDisplayHolder>,
    preview_render_bits: Query<(Entity, &PartOfPreviewRender)>,
) {
    if removed.len() <= 0 {
        return;
    }

    // This is probably stupid as fuck, What would be better is making a single child that we can find
    // Would be better than allocateing all this bullshit.
    // Hell maybe this whole thing could be a child of a UI element not sure if that would fuck with stuff though
    let mut mapping = HashMap::new();

    for (entity, part) in &preview_render_bits {
        mapping.entry(part.owner).or_insert(Vec::new()).push(entity);
    }

    for entity in removed.read() {
        if let Some(entires) = mapping.get(&entity) {
            for entity in entires {
                if let Some(commands) = commands.get_entity(*entity) {
                    commands.despawn_recursive();
                }
            }
        }
    }
}
