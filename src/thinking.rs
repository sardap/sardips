use bevy::prelude::*;

use crate::{
    dynamic_dialogue::{
        ActionEvent, Concept, EntityFactDatabase, FactQuery, GlobalFactDatabase, RuleSet,
    },
    simulation::SimulationState,
};

pub struct ThinkingPlugin;

impl Plugin for ThinkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_thought).run_if(in_state(SimulationState::Running)),
        );
    }
}

#[derive(Bundle, Default)]
pub struct ThinkerBundle {
    pub think_timer: ThinkTimer,
    pub thought: Thought,
}

#[derive(Debug, Component)]
pub struct ThinkTimer {
    timer: Timer,
}

impl Default for ThinkTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct Thought {
    pub text: Option<String>,
}

fn update_thought(
    mut action_events: EventWriter<ActionEvent>,
    time: Res<Time>,
    rule_set: Res<RuleSet>,
    global_fact_db: Res<GlobalFactDatabase>,
    mut thinkers: Query<(Entity, &mut ThinkTimer, &mut Thought, &EntityFactDatabase)>,
) {
    for (entity, mut thinker, mut thought, fact_db) in thinkers.iter_mut() {
        if thinker.timer.tick(time.delta()).just_finished() {
            let fact_query = FactQuery::new(Concept::ThinkIdle)
                .add_fact_db(&global_fact_db.0)
                .add_fact_db(&fact_db.0);
            let response = fact_query.run(&rule_set);
            if let Some(response) = response {
                thought.text = response.now.get_text().get(0).cloned();
                debug!("{:?} thinks: {:?}", entity, thought.text);
                action_events.send(ActionEvent::new(response.now).with_entity(entity));
            }
        }
    }
}
