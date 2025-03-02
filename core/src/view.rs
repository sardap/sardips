use bevy::prelude::*;

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (copy_transform, add_has_view))
            .observe(destroy_view);
    }
}

#[derive(Debug, Component)]
pub struct EntityView {
    pub entity: Entity,
}

#[derive(Debug, Component)]
pub struct HasView {
    pub view_entity: Entity,
}

fn copy_transform(mut transforms: Query<&mut Transform>, views: Query<(Entity, &EntityView)>) {
    for (entity, view) in views.iter() {
        let to_copy = if let Ok(transform) = transforms.get(view.entity) {
            *transform
        } else {
            continue;
        };
        if let Ok(mut view_transform) = transforms.get_mut(entity) {
            *view_transform = to_copy
        }
    }
}

fn add_has_view(
    mut commands: Commands,
    new_views: Query<(Entity, &EntityView), Added<EntityView>>,
) {
    for (entity, view) in new_views.iter() {
        commands.entity(view.entity).insert(HasView {
            view_entity: entity,
        });
    }
}

fn destroy_view(
    removed: Trigger<OnRemove, HasView>,
    mut commands: Commands,
    views: Query<(Entity, &EntityView)>,
) {
    let entity = removed.entity();

    for (view_entity, view) in views.iter() {
        if view.entity == entity {
            commands.entity(view_entity).despawn_recursive();
        }
    }
}
