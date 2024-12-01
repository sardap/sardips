use sardips::bevy::prelude::*;
use sardips::GamePlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(GamePlugin);
    app.add_plugins((
        sardips_endless_shooter::EndlessShooterPlugin,
        sardips_four_in_row::FourInRowPlugin,
        sardips_tic_tac_toe::TicTacToePlugin,
        sardips_higher_lower::HigherLowerPlugin,
    ));
    app.run();
}
