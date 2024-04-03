use bevy::math::Vec2;
use bevy_turborand::{DelegatedRng, RngComponent};

const GAME_ZONE_WIDTH: i32 = 400;
const GAME_ZONE_Y: i32 = 500;

pub fn random_point_in_game_zone(rng: &mut RngComponent) -> Vec2 {
    let x = rng.i32(-(GAME_ZONE_WIDTH / 2)..(GAME_ZONE_WIDTH / 2));
    let y = rng.i32(-(GAME_ZONE_Y / 2)..(GAME_ZONE_Y / 2));
    Vec2::new(x as f32, y as f32)
}
