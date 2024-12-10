use bevy::prelude::*;

use sardips::{rgb_to_color, rgba_to_color};

pub(crate) const INACTIVE_PROGRESS_BAR_COLOR: Color = rgba_to_color!(59, 59, 59, 125);
pub(crate) const ACTIVE_PROGRESS_BAR_COLOR: Color = rgb_to_color!(100, 100, 100);

pub(crate) const INACTIVE_LYRIC_COLOR: Color = rgb_to_color!(200, 200, 200);
pub(crate) const ACTIVE_LYRIC_COLOR: Color = rgb_to_color!(255, 255, 255);

pub(crate) const PROGRESS_MARKER: Color = rgb_to_color!(167, 180, 169);

pub(crate) const PASSED_INPUT_MARKER: Color = rgb_to_color!(139, 0, 0);
pub(crate) const HIT_INPUT_MARKER: Color = rgb_to_color!(1, 50, 32);
pub(crate) const PENDING_INPUT_MARKER: Color = rgb_to_color!(200, 200, 200);
