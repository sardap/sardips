use bevy::prelude::*;
use fact_db::parse;
use fact_db::{self, RuleSet};
use shared_deps::bevy_common_assets::ron::RonAssetPlugin;

pub struct DynamicDialoguePlugin;

impl Plugin for DynamicDialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<parse::RawRuleSet>::new(&["dialogue.ron"]))
            .add_systems(Startup, setup)
            .add_systems(Update, parse_rule_sets);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let template_set = parse::RawRuleSetHandle(asset_server.load("dialogue/main.dialogue.ron"));
    commands.insert_resource(template_set);
}

fn parse_rule_sets(
    mut commands: Commands,
    handle: Res<parse::RawRuleSetHandle>,
    mut assets: ResMut<Assets<parse::RawRuleSet>>,
) {
    if let Some(raw_rule_sets) = assets.remove(handle.0.id()) {
        let rule_set: RuleSet = raw_rule_sets.into();
        commands.insert_resource(rule_set);
    }
}
