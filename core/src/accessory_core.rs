use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use crate::{money_core::Money, particles::Spewer};

pub struct AccessoryCorePlugin;

impl Plugin for AccessoryCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AccessoryTemplateDatabase::new());
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

#[derive(Resource, Default)]
pub struct AccessoryTemplateDatabase {
    templates: HashMap<String, AccessoryTemplate>,
}

impl AccessoryTemplateDatabase {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        let cowboy_hat = AccessoryTemplate {
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(-5., -3.),
            texture: "textures/accessories/cowboyhat.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(173., 89.),
            wear_size: AccessorySize::StretchX,
            cost: 100,
            layer: AccessoryLayer::Front,
        };
        templates.insert("cowboy_hat".to_string(), cowboy_hat.clone());

        let ushanka_hat = AccessoryTemplate {
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(-5., -20.),
            texture: "textures/accessories/ushanka.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(128., 120.),
            wear_size: AccessorySize::StretchX,
            cost: 100,
            layer: AccessoryLayer::Front,
        };
        templates.insert("ushanka_hat".to_string(), ushanka_hat.clone());

        let fez_hat = AccessoryTemplate {
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(-5., 0.),
            texture: "textures/accessories/fez.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(250., 101.),
            wear_size: AccessorySize::StretchX,
            cost: 100,
            layer: AccessoryLayer::Front,
        };
        templates.insert("fez_hat".to_string(), fez_hat.clone());

        let wiz_hat = AccessoryTemplate {
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(-5., 15.),
            texture: "textures/accessories/wizhat.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(100., 123.),
            wear_size: AccessorySize::StretchX,
            cost: 100,
            layer: AccessoryLayer::Front,
        };
        templates.insert("wiz_hat".to_string(), wiz_hat.clone());

        let bennie_hat = AccessoryTemplate {
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(0., -5.),
            texture: "textures/accessories/bennie.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(102., 66.),
            wear_size: AccessorySize::StretchX,
            cost: 100,
            layer: AccessoryLayer::Front,
        };
        templates.insert("bennie_hat".to_string(), bennie_hat.clone());

        let pink_helmet = AccessoryTemplate {
            anchor_point: AnchorPoint::Head,
            anchor_offset: Vec2::new(0., -5.),
            texture: "textures/accessories/pink_helment.png".to_string(),
            spewers: vec![],
            texture_size: Vec2::new(173., 89.),
            wear_size: AccessorySize::StretchX,
            cost: 100,
            layer: AccessoryLayer::Front,
        };
        templates.insert("pink_helmet".to_string(), pink_helmet.clone());

        Self { templates }
    }

    pub fn get(&self, name: &str) -> Option<&AccessoryTemplate> {
        self.templates.get(name)
    }
}
