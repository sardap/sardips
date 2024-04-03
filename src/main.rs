use bevy::prelude::*;
use sardips::GamePlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(GamePlugin);
    app.run();
}
