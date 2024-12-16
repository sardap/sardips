use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(sardips_run::SardipsPlugin);
    app.run();
}
