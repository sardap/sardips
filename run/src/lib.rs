use bevy::prelude::*;
use sardips::GamePlugin;

pub struct SardipsPlugin;

impl Plugin for SardipsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GamePlugin);

        app.add_plugins((
            sardips_endless_shooter::EndlessShooterPlugin,
            sardips_four_in_row::FourInRowPlugin,
            sardips_tic_tac_toe::TicTacToePlugin,
            sardips_higher_lower::HigherLowerPlugin,
            sardips_rhythm::RhythmPlugin,
        ));
    }
}
