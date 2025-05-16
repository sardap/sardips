use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item>().register_type::<Inventory>();
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum Item {
    Accessory(crate::accessory::Accessory),
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect, Default)]
#[reflect(Component)]
pub struct Inventory {
    pub items: Vec<Item>,
}

impl Inventory {
    pub fn add_item(&mut self, to_add: Item) {
        self.items.push(to_add)
    }

    pub fn get_accessories(&self) -> impl Iterator<Item = &crate::accessory::Accessory> {
        self.items.iter().map(|item| match item {
            Item::Accessory(accessory) => accessory,
        })
    }
}
