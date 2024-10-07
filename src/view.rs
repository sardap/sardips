use bevy::prelude::*;

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EntityView>().register_type::<HasView>();
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
