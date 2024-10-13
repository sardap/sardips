use bevy::prelude::*;

use crate::{
    dynamic_dialogue::{ActionEvent, Concept, FactDb, FactQuery, RuleSet},
    facts::{EntityFactDatabase, GlobalFactDatabase},
    simulation::SimulationState,
};

pub struct ThinkingPlugin;

impl Plugin for ThinkingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ThinkTimer>()
            .register_type::<Thought>()
            .add_event::<TryThinkEvent>()
            .add_systems(
                Update,
                (trigger_idle_thoughts, handle_thought).run_if(in_state(SimulationState::Running)),
            );
    }
}

#[derive(Bundle, Default)]
pub struct ThinkerBundle {
    pub think_timer: ThinkTimer,
    pub thought: Thought,
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ThinkTimer {
    timer: Timer,
}

impl Default for ThinkTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(60.0, TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Component, Default, Reflect)]
#[reflect(Component)]
pub struct Thought {
    pub text: Option<String>,
}

fn trigger_idle_thoughts(
    mut think_events: EventWriter<TryThinkEvent>,
    time: Res<Time>,
    mut thinkers: Query<(Entity, &mut ThinkTimer)>,
) {
    for (entity, mut thinker) in thinkers.iter_mut() {
        if thinker.timer.tick(time.delta()).just_finished() {
            think_events.send(TryThinkEvent::new(entity, Concept::ThinkIdle));
        }
    }
}

#[derive(Debug, Event)]
pub struct TryThinkEvent {
    pub entity: Entity,
    pub concept: Concept,
    pub facts: FactDb,
}

impl TryThinkEvent {
    pub fn new(entity: Entity, concept: Concept) -> Self {
        Self {
            entity,
            concept,
            facts: FactDb::default(),
        }
    }

    pub fn with_facts(mut self, facts: FactDb) -> Self {
        self.facts = facts;
        self
    }
}

pub fn handle_thought(
    mut thinking_events: EventReader<TryThinkEvent>,
    mut action_events: EventWriter<ActionEvent>,
    rule_set: Res<RuleSet>,
    global_fact_db: Res<GlobalFactDatabase>,
    mut thinkers: Query<(Entity, &mut ThinkTimer, &mut Thought, &EntityFactDatabase)>,
) {
    for event in thinking_events.read() {
        if let Ok((entity, mut thinker, mut thought, fact_db)) = thinkers.get_mut(event.entity) {
            let fact_query = FactQuery::new(Concept::ThinkIdle)
                .add_fact_db(&global_fact_db.0)
                .add_fact_db(&fact_db.0)
                .add_fact_db(&event.facts);
            let response = fact_query.run(&rule_set);
            if let Some(response) = response {
                thought.text = response.now.get_text().first().cloned();
                info!("{:?} thinks: {:?}", entity, thought.text);
                action_events.send(ActionEvent::new(response.now).with_entity(entity));
            }
            thinker.timer.reset();
        } else {
            error!("Failed to get thinker entity {:?}", event.entity);
        }
    }
}
