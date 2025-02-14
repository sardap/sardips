use bevy::prelude::*;

pub struct AnimePlugin;

impl Plugin for AnimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (check_anime_timers_valid, tick_frames));
    }
}

#[derive(Bundle)]
pub struct AnimeBundle {
    pub timer: AnimeTimer,
    pub indices: AnimeIndices,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimeTimer(pub Timer);

#[derive(Component)]
pub struct AnimeIndices {
    pub first: usize,
    pub last: usize,
}

impl AnimeIndices {
    pub fn new(first: usize, last: usize) -> Self {
        assert!(
            first <= last,
            "First index must be less than or equal to last index"
        );
        Self { first, last }
    }
}

fn check_anime_timers_valid(query: Query<&AnimeTimer, Added<AnimeTimer>>) {
    for anime in query.iter() {
        assert!(
            anime.mode() == TimerMode::Repeating,
            "Anime frame timer must be repeating"
        );
    }
}

fn tick_frames(
    time: Res<Time>,
    mut query: Query<(&mut AnimeTimer, &mut TextureAtlas, &AnimeIndices)>,
) {
    for (mut anime, mut atlas, indices) in query.iter_mut() {
        if anime.tick(time.delta()).just_finished() {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}
