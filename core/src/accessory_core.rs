use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::{money_core::Money, particles::Spewer};

pub struct AccessoryCorePlugin;

impl Plugin for AccessoryCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AccessoryTemplateDatabase::new())
            .register_type::<AccessoryDiscoveredEntries>();
    }
}

#[derive(Deserialize, TypePath, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum AnchorPoint {
    Head,
}

#[derive(Deserialize, TypePath, Default)]
pub struct AnchorPointSet {
    pub points: HashMap<AnchorPoint, Vec2>,
}

impl AnchorPointSet {
    pub fn get(&self, point: AnchorPoint, wearer_size: &Vec2) -> Vec2 {
        info!("anchor point for {:?} wearer size {:?}", point, wearer_size);
        match self.points.get(&point) {
            Some(point) => *point,
            None => match point {
                AnchorPoint::Head => Vec2::new(0., wearer_size.y / 2.),
            },
        }
    }
}

#[derive(Deserialize, TypePath, Clone)]
pub enum AccessorySize {
    StretchX,
    StretchY,
    Constant(Vec2),
}

#[derive(Deserialize, TypePath, Clone, Copy)]
pub enum AccessoryLayer {
    Behind,
    Front,
}

impl From<AccessoryLayer> for f32 {
    fn from(layer: AccessoryLayer) -> f32 {
        layer.z()
    }
}

impl AccessoryLayer {
    pub fn z(&self) -> f32 {
        match self {
            AccessoryLayer::Behind => -0.1,
            AccessoryLayer::Front => 0.1,
        }
    }
}

#[derive(Deserialize, TypePath, Clone)]
pub struct AccessoryTemplate {
    pub name: String,
    pub anchor_point: AnchorPoint,
    pub anchor_offset: Vec2,
    pub texture: String,
    pub spewers: Vec<Spewer>,
    pub texture_size: Vec2,
    pub wear_size: AccessorySize,
    pub cost: Money,
    pub layer: AccessoryLayer,
}

impl AccessoryTemplate {
    pub fn with_spewers(mut self, spewers: Vec<Spewer>) -> Self {
        self.spewers = spewers;
        self
    }

    pub fn with_cost(mut self, cost: Money) -> Self {
        self.cost = cost;
        self
    }

    pub fn texture_area(&self) -> Rect {
        Rect::new(
            -(self.texture_size.x / 2.),
            -(self.texture_size.y / 2.),
            self.texture_size.x / 2.,
            self.texture_size.y / 2.,
        )
    }
}


#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct AccessoryDiscoveredEntries {
    pub entries: std::collections::HashSet<String>,
}

#[derive(Resource, Default)]
pub struct AccessoryTemplateDatabase {
    templates: HashMap<String, AccessoryTemplate>,
}

impl AccessoryTemplateDatabase {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        let mut add_template = |entry: AccessoryTemplate| {
            templates.insert(entry.name.clone(), entry);
        };

        let cowboy_hat = AccessoryTemplate {
            name: "cowboy_hat".to_string(),
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(-5., -3.),
            texture: "textures/accessories/cowboyhat.png".to_string(),
            spewers: vec![
                crate::particles::HAPPY.clone()
            ],
            texture_size: Vec2::new(173., 89.),
            wear_size: AccessorySize::StretchX,
            cost: 0,
            layer: AccessoryLayer::Front,
        };
        add_template(cowboy_hat);

        let ushanka_hat = AccessoryTemplate {
            name: "ushanka_hat".to_string(),
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(-5., -20.),
            texture: "textures/accessories/ushanka.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(128., 120.),
            wear_size: AccessorySize::StretchX,
            cost: 0,
            layer: AccessoryLayer::Front,
        };
        add_template(ushanka_hat);

        let fez_hat = AccessoryTemplate {
            name: "fez_hat".to_string(),
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(-5., 0.),
            texture: "textures/accessories/fez.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(250., 101.),
            wear_size: AccessorySize::StretchX,
            cost: 0,
            layer: AccessoryLayer::Front,
        };
        add_template(fez_hat);

        let wiz_hat = AccessoryTemplate {
            name: "wiz_hat".to_string(),
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(-5., 15.),
            texture: "textures/accessories/wizhat.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(100., 123.),
            wear_size: AccessorySize::StretchX,
            cost: 0,
            layer: AccessoryLayer::Front,
        };
        add_template(wiz_hat);

        let bennie_hat = AccessoryTemplate {
            name: "bennie_hat".to_string(),
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(0., -5.),
            texture: "textures/accessories/bennie.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(102., 66.),
            wear_size: AccessorySize::StretchX,
            cost: 0,
            layer: AccessoryLayer::Front,
        };
        add_template(bennie_hat);

        let pink_helmet = AccessoryTemplate {
            name: "pink_helmet".to_string(),
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(0., -5.),
            texture: "textures/accessories/pink_helment.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(173., 89.),
            wear_size: AccessorySize::StretchX,
            cost: 0,
            layer: AccessoryLayer::Front,
        };
        add_template(pink_helmet);

        Self { templates }
    }

    pub fn get(&self, name: &str) -> Option<&AccessoryTemplate> {
        self.templates.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &AccessoryTemplate)> {
        self.templates.iter()
    }
}
