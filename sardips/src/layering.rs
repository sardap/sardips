pub mod view_screen {
    pub const POOP: f32 = 0.0;
    pub const FOOD: f32 = 1.0;
    pub const PET: f32 = 2.0;
    pub const FOOD_EATING: f32 = 3.0;
    pub const TOOL: f32 = 4.0;
}


pub mod game_layers {
    pub const UNALLOCATED: std::ops::Range<u32> = 0..20;
    pub const PET_PREVIEW: std::ops::Range<u32> = 21..30;
}