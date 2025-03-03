use bevy::prelude::*;
use sardips_core::{
    accessory_core::{self, AccessoryTemplateDatabase},
    name::SpeciesName,
    pet_core::PetTemplateDatabase,
    sprite_utils::get_adjusted_size,
    view::HasView,
};

use crate::{accessory::Wearer, simulation::Simulated};

use super::Accessory;

pub struct AccessoryViewPlugin;

impl Plugin for AccessoryViewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Accessory>().add_systems(
            Update,
            spawn_accessory_view.run_if(resource_exists::<PetTemplateDatabase>),
        );
    }
}

#[derive(Component)]
pub struct AccessoryView;

fn spawn_accessory_view(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    pet_db: Res<PetTemplateDatabase>,
    accessory_db: Res<AccessoryTemplateDatabase>,
    views: Query<&GlobalTransform>,
    parents: Query<(&HasView, &SpeciesName, Option<&Simulated>)>,
    to_spawn: Query<(Entity, &Accessory, &Parent), Without<HasView>>,
) {
    for (entity, accessory, parent) in &to_spawn {
        info!("Spawning accessory view");
        let (parent_has_view, name, simulated) = match parents.get(parent.get()) {
            Ok(parent) => parent,
            Err(_) => continue,
        };

        let pet_template = match pet_db.get_by_name(&name.0) {
            Some(template) => template,
            None => continue,
        };

        let accessory_template = match accessory_db.get(&accessory.template) {
            Some(accessory) => accessory,
            None => continue,
        };

        let wearer = Wearer {
            size: &pet_template.pre_calculated.custom_size,
            anchor_points: &pet_template.anchor_points,
        };

        let parent_transform = match views.get(parent_has_view.view_entity) {
            Ok(view) => view,
            Err(_) => continue,
        };

        // TODO figure out anchor points
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

        let new_ent = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(size),
                    color: accessory.tint,
                    ..default()
                },
                transform: Transform::from_xyz(
                    point.x,
                    point.y,
                    parent_transform.translation().z + accessory_template.layer.z(),
                ),
                texture: asset_server.load(&accessory_template.texture),
                ..default()
            })
            .with_children(|parent| {
                for spewer in accessory_template
                    .spewers
                    .iter()
                    .chain(&accessory.extra_spewers)
                {
                    parent.spawn((
                        Transform::from_xyz(0., 0., 5.),
                        GlobalTransform::default(),
                        spewer.clone().with_spawn_area(Rect::new(
                            -size.x / 2.,
                            -size.y / 2.,
                            size.x / 2.,
                            size.y / 2.,
                        )),
                    ));
                }
            })
            .id();

        commands
            .entity(parent_has_view.view_entity)
            .push_children(&[new_ent]);

        commands.entity(entity).insert(HasView {
            view_entity: new_ent,
        });

        if simulated.is_some() {
            commands.entity(new_ent).insert(Simulated);
        }
    }
}
