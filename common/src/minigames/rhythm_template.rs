use bevy::prelude::*;
use shared_deps::bevy_kira_audio::AudioSource;
use std::{collections::HashMap, hash::Hash, time::Duration};

use serde::{Deserialize, Serialize};

pub struct RhythmTemplatePlugin;

impl Plugin for RhythmTemplatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveRhythmTemplate::default());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RhythmTemplateIntro {
    pub intro_text_key: String,
    pub image: String,
    pub end: f64,
    #[serde(skip)]
    pub image_handle: Option<Handle<Image>>,
}

impl RhythmTemplateIntro {
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.end)
    }

    pub fn start_load(&mut self, asset_server: &AssetServer) {
        self.image_handle = Some(asset_server.load(self.image.clone()));
    }

    pub fn loaded(&self, asset_server: &AssetServer) -> bool {
        if let Some(handle) = &self.image_handle {
            asset_server.is_loaded_with_dependencies(handle)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RhythmTemplateBackgroundEntry {
    pub end: f64,
    pub path: String,
    #[serde(skip)]
    pub image_handle: Option<Handle<Image>>,
}

impl RhythmTemplateBackgroundEntry {
    pub fn new<T: ToString>(image: T, start: f64) -> Self {
        Self {
            path: image.to_string(),
            end: start,
            image_handle: None,
        }
    }

    pub fn start_load(&mut self, asset_server: &AssetServer) {
        self.image_handle = Some(asset_server.load(self.path.clone()));
    }

    pub fn loaded(&self, asset_server: &AssetServer) -> bool {
        if let Some(handle) = &self.image_handle {
            asset_server.is_loaded_with_dependencies(handle)
        } else {
            false
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum Button {
    #[default]
    Click,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Tap {
    pub button: Button,
    pub sound_path: String,
    pub start: f64,
    #[serde(skip)]
    pub page: usize,
    #[serde(skip)]
    pub line: usize,
}

impl Tap {
    pub fn new<T: ToString>(button: Button, sound_path: T, start: f64) -> Self {
        Self {
            button,
            sound_path: sound_path.to_string(),
            start,
            ..default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RhythmTemplateInput {
    Tap(Tap),
}

impl RhythmTemplateInput {
    pub fn start(&self) -> f64 {
        match self {
            RhythmTemplateInput::Tap(tap) => tap.start,
        }
    }

    pub fn set_page(&mut self, page: usize) {
        match self {
            RhythmTemplateInput::Tap(tap) => tap.page = page,
        }
    }

    pub fn page(&self) -> usize {
        match self {
            RhythmTemplateInput::Tap(tap) => tap.page,
        }
    }

    pub fn set_line(&mut self, line: usize) {
        match self {
            RhythmTemplateInput::Tap(tap) => tap.line = line,
        }
    }

    pub fn line(&self) -> usize {
        match self {
            RhythmTemplateInput::Tap(tap) => tap.line,
        }
    }

    pub fn sound_path(&self) -> &str {
        match self {
            RhythmTemplateInput::Tap(tap) => &tap.sound_path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LineText {
    pub text_key: String,
    pub start: f64,
}

impl LineText {
    pub fn new<T: ToString>(text_key: T, duration_seconds: f64) -> Self {
        Self {
            text_key: text_key.to_string(),
            start: duration_seconds,
        }
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.start)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SoundBank {
    pub sounds: HashMap<String, Handle<AudioSource>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RhythmTemplateLine {
    pub end: f64,
    pub text: String,
    #[serde(skip)]
    pub start: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RhythmTemplate {
    pub song_path: String,
    pub intro: RhythmTemplateIntro,
    pub pages: Vec<Vec<RhythmTemplateLine>>,
    pub inputs: Vec<RhythmTemplateInput>,
    pub backgrounds: Vec<RhythmTemplateBackgroundEntry>,
    #[serde(skip)]
    pub sound_bank: SoundBank,
    #[serde(skip)]
    pub song_handle: Option<Handle<AudioSource>>,
}

impl RhythmTemplate {
    pub fn start_load(&mut self, asset_server: &AssetServer) {
        self.intro.start_load(asset_server);

        for background in &mut self.backgrounds {
            background.start_load(asset_server);
        }

        self.song_handle = Some(asset_server.load(self.song_path.clone()));

        for input in &self.inputs {
            match input {
                RhythmTemplateInput::Tap(tap) => {
                    let sound: Handle<AudioSource> = asset_server.load(tap.sound_path.clone());
                    self.sound_bank.sounds.insert(tap.sound_path.clone(), sound);
                }
            }
        }

        for i in 0..self.pages.len() {
            let line_count = self.pages[i].len();
            for j in 0..line_count {
                let prev_end = if j == 0 {
                    if i == 0 {
                        self.intro.end
                    } else {
                        self.pages[i - 1].last().unwrap().end
                    }
                } else {
                    self.pages[i][j - 1].end
                };

                self.pages[i][j].start = prev_end;
            }
        }

        for i in 0..self.inputs.len() {
            let input_start = self.inputs[i].start();
            if let Some((page, index)) = self.get_page_line_for_time(input_start) {
                self.inputs[i].set_page(page);
                self.inputs[i].set_line(index);
            }
        }
    }

    fn get_page_line_for_time(&self, time: f64) -> Option<(usize, usize)> {
        for i in 0..self.pages.len() {
            for j in 0..self.pages[i].len() {
                let line = &self.pages[i][j];
                if line.start <= time && time <= line.end {
                    return Some((i, j));
                }
            }
        }

        None
    }

    pub fn page_end(&self, page: usize) -> f64 {
        self.pages[page].last().unwrap().end
    }

    pub fn page_start(&self, page: usize) -> f64 {
        self.pages[page].first().unwrap().start
    }

    pub fn index_for_time(&self, time: f64) -> Option<(usize, usize)> {
        for (i, page) in self.pages.iter().enumerate() {
            for (j, line) in page.iter().enumerate() {
                if line.start <= time && time <= line.end {
                    return Some((i, j));
                }
            }
        }
        None
    }

    pub fn get_input_sound(&self, input: &RhythmTemplateInput) -> Handle<AudioSource> {
        self.sound_bank.sounds[input.sound_path()].clone()
    }

    pub fn loaded(&self, asset_server: &AssetServer) -> bool {
        if !self.intro.loaded(asset_server) {
            return false;
        }

        for background in &self.backgrounds {
            if !background.loaded(asset_server) {
                return false;
            }
        }

        if !asset_server.is_loaded_with_dependencies(self.song_handle.as_ref().unwrap()) {
            return false;
        }

        for sound in self.sound_bank.sounds.values() {
            if !asset_server.is_loaded_with_dependencies(sound) {
                return false;
            }
        }
        true
    }
}

lazy_static! {
    static ref TESTING_TEMPLATE: RhythmTemplate = {
        const LENGTH: f64 = 74.;
        const STEP_COUNT: f64 = (LENGTH / 60.) * 105.;
        const STEP_SIZE: f64 = 60. / STEP_COUNT;
        let mut inputs = vec![];
        for i in 0..STEP_COUNT as usize {
            inputs.push(RhythmTemplateInput::Tap(Tap::new(
                Button::Click,
                "sounds/mini_games/rhythm/hey.wav",
                6. + i as f64 * STEP_SIZE,
            )));
        }

        RhythmTemplate {
            song_path: "sounds/mini_games/rhythm/bird.ogg".to_string(),
            intro: RhythmTemplateIntro {
                intro_text_key: "bird.intro".to_string(),
                image: "textures/mini_games/rhythm/test/intro_background.png".to_string(),
                end: 1.5,
                ..default()
            },
            pages: vec![
                vec![
                    RhythmTemplateLine {
                        text: "bird.line_1".to_string(),
                        end: 6.25,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_2".to_string(),
                        end: 11.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_3".to_string(),
                        end: 13.0,
                        ..default()
                    },
                ],
                vec![
                    RhythmTemplateLine {
                        text: "bird.line_4".to_string(),
                        end: 18.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_5".to_string(),
                        end: 20.0,
                        ..default()
                    },
                ],
                vec![
                    RhythmTemplateLine {
                        text: "bird.line_6".to_string(),
                        end: 22.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_7".to_string(),
                        end: 26.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_8".to_string(),
                        end: 29.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_9".to_string(),
                        end: 30.0,
                        ..default()
                    },
                ],
                vec![
                    RhythmTemplateLine {
                        text: "bird.line_10".to_string(),
                        end: 33.5,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_11".to_string(),
                        end: 36.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_12".to_string(),
                        end: 37.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_13".to_string(),
                        end: 38.0,
                        ..default()
                    },
                ],
                vec![
                    RhythmTemplateLine {
                        text: "bird.line_14".to_string(),
                        end: 42.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_15".to_string(),
                        end: 44.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_16".to_string(),
                        end: 46.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_17".to_string(),
                        end: 47.0,
                        ..default()
                    },
                ],
                vec![
                    RhythmTemplateLine {
                        text: "bird.line_18".to_string(),
                        end: 56.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_19".to_string(),
                        end: 59.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_20".to_string(),
                        end: 65.0,
                        ..default()
                    },
                ],
                vec![
                    RhythmTemplateLine {
                        text: "bird.line_21".to_string(),
                        end: 67.0,
                        ..default()
                    },
                    RhythmTemplateLine {
                        text: "bird.line_22".to_string(),
                        end: 74.0,
                        ..default()
                    },
                ],
            ],
            inputs,
            backgrounds: vec![RhythmTemplateBackgroundEntry::new(
                "textures/mini_games/rhythm/test/background_1.png",
                3.58,
            )],
            ..default()
        }
    };
}

#[derive(Resource)]
pub struct ActiveRhythmTemplate {
    pub selected_template: Option<RhythmTemplate>,
}

impl Default for ActiveRhythmTemplate {
    fn default() -> Self {
        Self {
            selected_template: Some(TESTING_TEMPLATE.clone()),
        }
    }
}

impl ActiveRhythmTemplate {
    pub fn active(&self) -> &RhythmTemplate {
        self.selected_template.as_ref().unwrap()
    }
}
