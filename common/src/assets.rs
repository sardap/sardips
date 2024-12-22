use bevy::{
    asset::Handle, ecs::system::Resource, render::texture::Image, sprite::TextureAtlasLayout,
    text::Font,
};
use bevy_asset_loader::asset_collection::AssetCollection;
use shared_deps::bevy_kira_audio::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct BackgroundTexturesAssets {}

impl BackgroundTexturesAssets {
    pub const MENU_BACKGROUND: &'static str = "textures/main_menu/background.png";
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/main_font.ttf")]
    pub main_font: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "sounds/eating.ogg")]
    pub eating: Handle<AudioSource>,
    #[asset(path = "sounds/poop.ogg")]
    pub poop: Handle<AudioSource>,
    #[asset(path = "sounds/poop_scoop.ogg")]
    pub poop_scoop: Handle<AudioSource>,
    #[asset(path = "sounds/error.ogg")]
    pub error: Handle<AudioSource>,
    #[asset(path = "sounds/place.ogg")]
    pub place: Handle<AudioSource>,
    #[asset(path = "sounds/victory.ogg")]
    pub victory: Handle<AudioSource>,
    #[asset(path = "sounds/defeat.ogg")]
    pub defeat: Handle<AudioSource>,
    #[asset(path = "sounds/draw.ogg")]
    pub draw: Handle<AudioSource>,
    #[asset(path = "sounds/higher.ogg")]
    pub higher: Handle<AudioSource>,
    #[asset(path = "sounds/lower.ogg")]
    pub lower: Handle<AudioSource>,
    #[asset(path = "sounds/correct.ogg")]
    pub correct: Handle<AudioSource>,
    #[asset(path = "sounds/plastic_drop.ogg")]
    pub plastic_drop: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct ViewScreenImageAssets {
    #[asset(texture_atlas_layout(tile_size_x = 70, tile_size_y = 70, columns = 5, rows = 1,))]
    pub view_buttons_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/view_screen/view_buttons.png")]
    pub view_buttons: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 30, tile_size_y = 30, columns = 5, rows = 1,))]
    pub moods_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/view_screen/moods.png")]
    pub moods: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 30, tile_size_y = 30, columns = 10, rows = 1,))]
    pub mood_icons_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/view_screen/mood_icons.png")]
    pub mood_icons: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 60, tile_size_y = 60, columns = 1, rows = 1,))]
    pub top_icons_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/view_screen/top_icons.png")]
    pub top_icons: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct GameImageAssets {
    #[asset(path = "textures/game/poop.png")]
    pub poop: Handle<Image>,

    #[asset(path = "textures/game/stink_lines.png")]
    pub stink_lines: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 64, tile_size_y = 30, columns = 1, rows = 2,))]
    pub stink_line_layout: Handle<TextureAtlasLayout>,

    #[asset(path = "textures/game/poop_scooper.png")]
    pub poop_scooper: Handle<Image>,

    #[asset(path = "textures/game/egg.png")]
    pub egg: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct TicTacToeAssets {
    #[asset(texture_atlas_layout(tile_size_x = 120, tile_size_y = 120, columns = 3, rows = 1,))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/mini_games/tic_tac_toe/sprites.png")]
    pub sprites: Handle<Image>,
}

impl TicTacToeAssets {
    pub const BACKGROUND: &'static str = "textures/mini_games/tic_tac_toe/background.png";
    pub const WIN_BACKGROUND: &'static str = "textures/mini_games/tic_tac_toe/win_background.png";
    pub const LOSE_BACKGROUND: &'static str = "textures/mini_games/tic_tac_toe/lose_background.png";
    pub const DRAW_BACKGROUND: &'static str = "textures/mini_games/tic_tac_toe/draw_background.png";
}

#[derive(AssetCollection, Resource)]
pub struct SprintAssets {}

impl SprintAssets {
    pub const SKY: &'static str = "textures/mini_games/sprint/sky.png";
    pub const HILLS: &'static str = "textures/mini_games/sprint/hills.png";
    pub const GROUND: &'static str = "textures/mini_games/sprint/ground.png";
    pub const GRASS_BLADES: &'static str = "textures/mini_games/sprint/grass_blades.png";
}

#[derive(AssetCollection, Resource)]
pub struct HigherLowerAssets {
    #[asset(texture_atlas_layout(tile_size_x = 100, tile_size_y = 80, columns = 5, rows = 1,))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/mini_games/higher_lower/sprites.png")]
    pub sprites: Handle<Image>,
    #[asset(path = "textures/mini_games/higher_lower/sign.png")]
    pub sign: Handle<Image>,
}

impl HigherLowerAssets {
    pub const BACKGROUND: &'static str = "textures/mini_games/higher_lower/background.png";
    pub const WIN_BACKGROUND: &'static str = "textures/mini_games/higher_lower/win_background.png";
    pub const LOSE_BACKGROUND: &'static str =
        "textures/mini_games/higher_lower/lose_background.png";
}

#[derive(AssetCollection, Resource)]
pub struct FourInRowAssets {
    #[asset(texture_atlas_layout(tile_size_x = 60, tile_size_y = 60, columns = 3, rows = 1,))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/mini_games/four_in_row/discs.png")]
    pub discs: Handle<Image>,
    #[asset(path = "textures/mini_games/four_in_row/board_tile.png")]
    pub board_tile: Handle<Image>,
    #[asset(path = "textures/mini_games/four_in_row/background.png")]
    pub background: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct EndlessShooterAssets {
    #[asset(texture_atlas_layout(tile_size_x = 60, tile_size_y = 60, columns = 3, rows = 1,))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/mini_games/endless_shooter/discs.png")]
    pub discs: Handle<Image>,
    #[asset(path = "textures/mini_games/endless_shooter/background.png")]
    pub background: Handle<Image>,
    #[asset(path = "textures/mini_games/endless_shooter/bubble.png")]
    pub bubble: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct TranslateAssets {
    #[asset(path = "textures/mini_games/translate/background.jpg")]
    pub background: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct DipdexImageAssets {
    #[asset(path = "textures/dipdex/unknown.png")]
    pub unknown: Handle<Image>,
    #[asset(path = "textures/dipdex/noise_3.png")]
    pub screen_noise: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 100, tile_size_y = 100, columns = 8, rows = 8,))]
    pub screen_noise_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/dipdex/known_background.png")]
    pub known_background: Handle<Image>,
}
