use bevy::prelude::*;
use shared_deps::bevy_kira_audio::prelude::*;

use crate::assets::AudioAssets;

pub struct SoundsPlugin;

impl Plugin for SoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlaySoundEffect>();
        app.add_systems(
            Update,
            play_pending_sounds.run_if(resource_exists::<AudioAssets>),
        );
    }
}

pub enum SoundEffect {
    Error,
    Poop,
    Scoop,
    Eating,
    Place,
    Victory,
    Defeat,
    Draw,
    Lower,
    Higher,
    Correct,
    PlasticDrop,
}

#[derive(Event)]
pub struct PlaySoundEffect {
    pub sound: SoundEffect,
    pub volume: Option<f32>,
}

impl PlaySoundEffect {
    pub fn new(sound: SoundEffect) -> Self {
        Self {
            sound,
            volume: None,
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume);
        self
    }
}

fn play_pending_sounds(
    mut to_play: EventReader<PlaySoundEffect>,
    audio: Res<Audio>,
    audio_assets: Res<AudioAssets>,
) {
    for sound_effect in to_play.read() {
        let volume = sound_effect.volume.unwrap_or(1.0);

        let asset = match sound_effect.sound {
            SoundEffect::Error => &audio_assets.error,
            SoundEffect::Poop => &audio_assets.poop,
            SoundEffect::Scoop => &audio_assets.poop_scoop,
            SoundEffect::Eating => &audio_assets.eating,
            SoundEffect::Place => &audio_assets.place,
            SoundEffect::Victory => &audio_assets.victory,
            SoundEffect::Defeat => &audio_assets.defeat,
            SoundEffect::Draw => &audio_assets.draw,
            SoundEffect::Lower => &audio_assets.lower,
            SoundEffect::Higher => &audio_assets.higher,
            SoundEffect::Correct => &audio_assets.correct,
            SoundEffect::PlasticDrop => &audio_assets.plastic_drop,
        }
        .clone();

        audio
            .play(asset)
            .with_volume(Volume::Amplitude(volume.into()));
    }
}
