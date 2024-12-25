pub mod rhythm_template;
pub mod translate_wordbank;

use bevy::prelude::*;
use rhythm_template::RhythmTemplatePlugin;
use translate_wordbank::TranslateWordBankPlugin;

pub struct MiniGamesCorePlugin;

impl Plugin for MiniGamesCorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state::<MiniGameState>(MiniGameState::default())
            .add_event::<MiniGameCompleted>()
            .add_plugins((RhythmTemplatePlugin, TranslateWordBankPlugin));
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum MiniGameResult {
    Incomplete,
    Win,
    Lose,
    Draw,
}

#[derive(Event)]
pub struct MiniGameCompleted {
    pub game_type: MiniGameType,
    pub result: MiniGameResult,
}

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum MiniGameState {
    #[default]
    None,
    Selecting,
    PlayingTicTakToe,
    PlayingHigherLower,
    PlayingFourInRow,
    PlayingEndlessShooter,
    PlayingRhythm,
    PlayingTranslate,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MiniGameType {
    TicTacToe,
    HigherLower,
    FourInRow,
    EndlessShooter,
    Rhythm,
    Translate,
}

#[derive(Component)]
pub struct Playing;

#[derive(Component)]
pub struct MiniGameBackButton;
