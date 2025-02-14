use bevy::math::Vec2;

pub fn get_adjusted_size(target_size: f32, size: Vec2) -> Vec2 {
    let size_x: f32;
    let size_y: f32;

    if size.x > size.y {
        let x = target_size;
        let ratio = x / size.x;
        let y = size.y * ratio;
        size_x = x;
        size_y = y;
    } else {
        let y = target_size;
        let ratio = y / size.y;
        let x = size.x * ratio;
        size_x = x;
        size_y = y;
    }

    Vec2::new(size_x, size_y)
}
