use bevy::prelude::*;

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EntityView>()
            .register_type::<HasView>()
            .add_systems(Update, copy_transform);
    }
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct EntityView {
    pub entity: Entity,
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
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
