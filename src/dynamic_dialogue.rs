use core::fmt;
use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use rand::prelude::SliceRandom;
use serde::de::{self, Visitor};
use serde::Deserializer;
use serde::{Deserialize, Serialize};

use crate::facts::{fact_str_hash, EntityFactDatabase, GlobalFactDatabase};

// https://www.youtube.com/watch?v=tAbBID3N64A&t=20s
// https://www.gdcvault.com/play/1015317/AI-driven-Dynamic-Dialog-through

pub struct DynamicDialoguePlugin;

impl Plugin for DynamicDialoguePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<parse::RawRuleSet>::new(&["dialogue.ron"]))
            .insert_resource(GlobalFactDatabase::default())
            .add_event::<ActionEvent>()
            .add_event::<FactInsert>()
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

#[allow(dead_code)]
#[derive(Event)]
enum FactInsert {
    Global(String, f32, Option<Duration>),
    Entity(Entity, String, f32, Option<Duration>),
}

#[derive(Component)]
struct PendingFactDelete {
    key: String,
    expire: Timer,
}

impl PendingFactDelete {
    fn new<T: ToString>(key: T, duration: Duration) -> Self {
        Self {
            key: key.to_string(),
            expire: Timer::new(duration, TimerMode::Once),
        }
    }
}

#[allow(dead_code)]
#[derive(Component)]
struct PendingFactDeleteEntity(Entity);

#[allow(dead_code)]
fn read_fact_inserts(
    mut commands: Commands,
    mut fact_db: ResMut<GlobalFactDatabase>,
    mut fact_db_entities: Query<&mut EntityFactDatabase>,
    mut fact_inserts: EventReader<FactInsert>,
) {
    for insert in fact_inserts.read() {
        match insert {
            FactInsert::Global(key, value, expire) => {
                fact_db.0.add(key.clone(), *value);
                if let Some(expire) = expire {
                    commands.spawn(PendingFactDelete::new(key.clone(), *expire));
                }
            }
            FactInsert::Entity(entity, key, value, expire) => {
                if let Ok(mut fact_db) = fact_db_entities.get_mut(*entity) {
                    fact_db.0.add(key.clone(), *value);
                    if let Some(expire) = expire {
                        commands.spawn((
                            PendingFactDelete::new(key.clone(), *expire),
                            PendingFactDeleteEntity(*entity),
                        ));
                    }
                } else {
                    error!("Entity {:?} does not have a fact database", entity);
                }
            }
        }
    }
}

#[allow(dead_code)]
fn tick_pending_fact_deletes(time: Res<Time>, mut query: Query<&mut PendingFactDelete>) {
    for mut delete in query.iter_mut() {
        delete.expire.tick(time.delta());
    }
}

#[allow(dead_code)]
fn expire_global_facts(
    mut commands: Commands,
    mut fact_db: ResMut<GlobalFactDatabase>,
    query: Query<(Entity, &PendingFactDelete), Without<PendingFactDeleteEntity>>,
) {
    for (entity, delete) in &query {
        if delete.expire.finished() {
            fact_db.0.facts.remove(&delete.key);
            commands.entity(entity).despawn();
        }
    }
}

#[allow(dead_code)]
fn expire_entity_facts(
    mut commands: Commands,
    mut fact_dbs: Query<&mut EntityFactDatabase>,
    deletes: Query<(Entity, &PendingFactDelete, &PendingFactDeleteEntity)>,
) {
    for (entity, delete, target) in &deletes {
        if delete.expire.finished() {
            if let Ok(mut fact_db) = fact_dbs.get_mut(target.0) {
                fact_db.0.facts.remove(&delete.key);
            }

            commands.entity(entity).despawn();
        }
    }
}

#[derive(Event)]
pub struct ActionEvent {
    pub entity: Option<Entity>,
    pub action_set: ActionSet,
}

impl ActionEvent {
    pub fn new(action_set: ActionSet) -> Self {
        Self {
            entity: None,
            action_set,
        }
    }

    pub fn with_entity(mut self, entity: Entity) -> Self {
        self.entity = Some(entity);
        self
    }
}

#[allow(dead_code)]
fn apply_pending_action(
    mut action_events: EventReader<ActionEvent>,
    mut inserts: EventWriter<FactInsert>,
) {
    for event in action_events.read() {
        for action in &event.action_set.actions {
            match action {
                Action::InsertGlobalFact(key, value, expire) => {
                    inserts.send(FactInsert::Global(key.clone(), *value, *expire));
                }
                Action::InsertEntityFact(key, value, expire) => {
                    if let Some(entity) = event.entity {
                        inserts.send(FactInsert::Entity(entity, key.clone(), *value, *expire));
                    } else {
                        error!("No entity provided for entity fact insert");
                    }
                }
                Action::RandomText(_) => {}
            }
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub struct FactDb {
    facts: HashMap<String, f32>,
}

impl FactDb {
    pub fn add<T: ToString>(&mut self, key: T, value: f32) {
        self.facts.insert(key.to_string(), value);
    }

    pub fn add_str<T: ToString, J: ToString>(&mut self, key: T, value: J) {
        self.facts
            .insert(key.to_string(), fact_str_hash(value.to_string()));
    }

    pub fn remove<T: ToString>(&mut self, key: T) {
        self.facts.remove(&key.to_string());
    }

    pub fn get(&self, key: &str) -> f32 {
        if let Some(value) = self.facts.get(key) {
            *value
        } else {
            0.
        }
    }

    // SLOW POINT
    pub fn remove_with_prefix(&mut self, prefix: &str) {
        let keys: Vec<_> = self
            .facts
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        for key in keys {
            self.facts.remove(&key);
        }
    }
}

impl fmt::Display for FactDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (key, value)) in self.facts.iter().enumerate() {
            write!(f, "{}: {}", key, value)?;
            if i < self.facts.len() - 1 {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Concept {
    ThinkIdle,
    ThinkJustAte,
    ThinkStartingEating,
    Evolve,
}

// Add cooldown
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    RandomText(Vec<String>),
    InsertGlobalFact(String, f32, Option<Duration>),
    InsertEntityFact(String, f32, Option<Duration>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSet {
    actions: Vec<Action>,
}

impl ActionSet {
    pub fn new(actions: Vec<Action>) -> Self {
        Self { actions }
    }

    pub fn get_text(&self) -> Vec<String> {
        self.actions
            .iter()
            .filter_map(|a| match a {
                Action::RandomText(text) => Some(text.clone()),
                _ => None,
            })
            .flatten()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub now: ActionSet,
    pub after: ActionSet,
}

#[derive(Debug, Clone)]
pub struct Criterion {
    key: String,
    fa: f32,
    fb: f32,
}

impl Criterion {
    pub fn evaluate(&self, value: f32) -> bool {
        value >= self.fa && value <= self.fb
    }
}

impl fmt::Display for Criterion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} in [{}, {}]", self.key, self.fa, self.fb)
    }
}

struct CriterionVisitor;

impl<'de> Visitor<'de> for CriterionVisitor {
    type Value = Criterion;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("")
    }

    fn visit_str<E>(self, v: &str) -> Result<Criterion, E>
    where
        E: de::Error,
    {
        Ok(parse::parse_criterion(v))
    }
}

impl<'de> Deserialize<'de> for Criterion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CriterionVisitor)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Criteria {
    concept: Concept,
    criterion: Vec<Criterion>,
}

impl Criteria {
    pub fn new(concept: Concept, criterion: &[Criterion]) -> Self {
        Self {
            concept,
            criterion: criterion.to_vec(),
        }
    }

    fn evaluate(&self, fact_dbs: &FactDataBaseSet) -> bool {
        for criterion in &self.criterion {
            if !criterion.evaluate(fact_dbs.get(&criterion.key)) {
                return false;
            }
        }

        true
    }
}

impl fmt::Display for Criteria {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "concept {:?} criterion", self.concept).unwrap();

        for criteria in &self.criterion {
            write!(f, " {}", criteria)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    id: String,
    criteria: Criteria,
    response: Response,
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rule {} criteria {}", self.id, self.criteria)
    }
}

#[derive(Debug, Resource, Clone)]
pub struct RuleSet {
    rules: Vec<Rule>,
}

#[derive(Debug, Default, Clone)]
struct FactDataBaseSet<'a> {
    fact_dbs: Vec<&'a FactDb>,
}

impl<'a> FactDataBaseSet<'a> {
    fn get(&self, key: &str) -> f32 {
        for fact_db in &self.fact_dbs {
            if let Some(value) = fact_db.facts.get(key) {
                return *value;
            }
        }

        0.
    }

    fn add(&mut self, fact_db: &'a FactDb) {
        self.fact_dbs.push(fact_db);
    }

    fn len(&self) -> usize {
        self.fact_dbs.len()
    }
}

impl fmt::Display for FactDataBaseSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FactDataBaseSet: {}", self.fact_dbs.len()).unwrap();

        for fact_db in &self.fact_dbs {
            write!(f, " {}", *fact_db).unwrap();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct FactQuery<'a> {
    pub concept: Concept,
    fact_dbs: FactDataBaseSet<'a>,
    query_fact_db: FactDb,
}

impl<'a> FactQuery<'a> {
    pub fn new(concept: Concept) -> Self {
        Self {
            concept,
            query_fact_db: FactDb::default(),
            fact_dbs: FactDataBaseSet::default(),
        }
    }

    pub fn add_fact<T: ToString>(mut self, key: T, value: f32) -> Self {
        self.query_fact_db.add(key, value);
        self
    }

    pub fn add_fact_db(mut self, fact_db: &'a FactDb) -> Self {
        self.fact_dbs.add(fact_db);
        self
    }

    pub fn run(&self, rule_set: &RuleSet) -> Option<Response> {
        debug!(
            "Running fact query with {} dbs and concept {:?}",
            self.fact_dbs.len(),
            self.concept
        );

        // This is maybe bad
        let mut fact_dbs = self.fact_dbs.clone();
        fact_dbs.add(&self.query_fact_db);

        debug!("Query fact_dbs loaded: {}", fact_dbs);

        let mut matches = Vec::new();
        let mut level = None;

        for rule in &rule_set.rules {
            // Found all possible matches for the current level
            if let Some(level) = level {
                debug!("Checking level {} {}", level, rule.criteria.criterion.len());
                if rule.criteria.criterion.len() < level {
                    break;
                }
            }

            if rule.criteria.concept != self.concept {
                debug!("rule {} does not match concept", rule.id);
                continue;
            }

            // find all matching rules with the same criteria count
            debug!("Checking rule {}", rule);
            if rule.criteria.evaluate(&fact_dbs) {
                debug!("Rule {} matches", rule.id);
                if matches.is_empty() {
                    level = Some(rule.criteria.criterion.len());
                }
                matches.push(&rule.response);
            }
        }

        if matches.is_empty() {
            debug!("No matches found");
            return None;
        }

        Some((*matches.choose(&mut rand::thread_rng()).unwrap()).clone())
    }

    pub fn single_criteria(&self, criteria: &Criteria) -> bool {
        criteria.concept == self.concept && criteria.evaluate(&self.fact_dbs)
    }
}

mod parse {
    use std::time::Duration;

    use bevy::prelude::*;
    use serde::{Deserialize, Serialize};

    use crate::facts::fact_str_hash;

    use super::Concept;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub(super) struct Criteria {
        pub(super) concept: Concept,
        pub(super) facts: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub(super) struct RawResponse {
        pub(super) id: String,
        pub(super) now: Vec<String>,
        #[serde(default)]
        pub(super) after: Vec<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub(super) struct ApplyFact {}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub(super) struct RawRule {
        pub(super) id: String,
        pub(super) criteria: Criteria,
        pub(super) response: String,
        #[serde(default)]
        pub(super) apply_facts: Vec<ApplyFact>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub(super) enum Entry {
        Response(RawResponse),
        Rule(RawRule),
    }

    #[derive(Debug, Asset, Serialize, Deserialize, TypePath, Clone)]
    pub(super) struct RawRuleSet {
        pub(super) entries: Vec<Entry>,
    }

    impl RawRuleSet {
        fn get_response(&self, id: &str) -> &RawResponse {
            match self
                .entries
                .iter()
                .filter_map(|e| match e {
                    Entry::Response(r) => Some(r),
                    Entry::Rule(_) => None,
                })
                .find(|r| r.id == id)
            {
                Some(response) => response,
                None => panic!("Response {} not found", id),
            }
        }

        fn get_rules(&self) -> Vec<&RawRule> {
            self.entries
                .iter()
                .filter_map(|e| match e {
                    Entry::Rule(r) => Some(r),
                    Entry::Response(_) => None,
                })
                .collect()
        }
    }

    fn raw_action_to_action_set(actions: &[String]) -> super::ActionSet {
        let mut result = Vec::new();

        for action in actions {
            let splits = action.split(" ").collect::<Vec<_>>();

            match splits[0] {
                "RandomText" => {
                    let strings: Vec<_> = splits[1].split(",").map(|s| s.to_owned()).collect();
                    result.push(super::Action::RandomText(strings));
                }
                "AddGlobalFact" => {
                    let key = splits[1].to_string();
                    let expire = if splits.len() > 2 {
                        Some(Duration::from_secs_f32(splits[2].parse::<f32>().unwrap()))
                    } else {
                        None
                    };
                    result.push(super::Action::InsertGlobalFact(key, 1.0, expire));
                }
                "AddEntityFact" => {
                    let key = splits[1].to_string();
                    let expire = if splits.len() > 2 {
                        Some(Duration::from_secs_f32(splits[2].parse::<f32>().unwrap()))
                    } else {
                        None
                    };
                    result.push(super::Action::InsertEntityFact(key, 1.0, expire));
                }
                _ => panic!("Invalid action: {}", action),
            }
        }

        super::ActionSet::new(result)
    }

    impl From<RawRuleSet> for super::RuleSet {
        fn from(val: RawRuleSet) -> super::RuleSet {
            let mut result = super::RuleSet { rules: Vec::new() };

            let rules = val.get_rules();

            for rule in rules {
                let possible_criteria = parse_criteria(&rule.criteria);

                for criteria in possible_criteria.into_iter() {
                    let raw_response = val.get_response(&rule.response);

                    let response = super::Response {
                        now: raw_action_to_action_set(&raw_response.now),
                        after: raw_action_to_action_set(&raw_response.after),
                    };

                    result.rules.push(super::Rule {
                        id: rule.id.clone(),
                        criteria,
                        response,
                    });
                }
            }

            // Sort rules by criteria count
            result
                .rules
                .sort_by(|a, b| b.criteria.criterion.len().cmp(&a.criteria.criterion.len()));

            result
        }
    }

    pub fn parse_criterion<T: ToString>(criterion: T) -> super::Criterion {
        let criterion = criterion.to_string();
        let splits = criterion.split(" ").collect::<Vec<_>>();

        match splits.len() {
            1 => {
                let key = splits[0].to_string();
                let fa = 1.0;
                let fb = 1.0;

                super::Criterion { key, fa, fb }
            }
            2 => {
                let key = splits[0].to_string();
                let operator = splits[1];

                let (fa, fb) = match operator {
                    "!" => (0.0, 0.0),
                    _ => panic!("Invalid operator: {}", operator),
                };

                super::Criterion { key, fa, fb }
            }
            3 => {
                let key = splits[0].to_string();
                let operator = splits[1];

                if let Ok(value) = splits[2].parse::<f32>() {
                    let (fa, fb) = match operator {
                        "<" => (f32::MIN, value),
                        ">" => (value, f32::MAX),
                        "=" => (value, value),
                        _ => panic!("Invalid operator: {}", operator),
                    };

                    super::Criterion { key, fa, fb }
                } else {
                    // handle string
                    if operator != "=" {
                        panic!("Invalid operator: {} for string", operator);
                    }

                    let hash = fact_str_hash(splits[2]);
                    super::Criterion {
                        key,
                        fa: hash,
                        fb: hash,
                    }
                }
            }
            _ => panic!("Invalid fact: {}", criterion),
        }
    }

    fn generate_combinations_with_core_vec<T: Clone + std::fmt::Debug>(
        alts: Vec<Vec<T>>,
        core: Vec<T>,
    ) -> Vec<Vec<T>> {
        let mut combinations = vec![];

        // Recursive function to generate combinations
        fn recurse<T: Clone + std::fmt::Debug>(
            alts: &[Vec<T>],
            core: &Vec<T>,
            current: Vec<T>,
            combinations: &mut Vec<Vec<T>>,
        ) {
            if alts.is_empty() {
                // Once there are no more alternative groups, append the core items to the current combination
                let mut final_combination = current.clone();
                final_combination.extend(core.clone());
                combinations.push(final_combination);
            } else {
                // Iterate over the first group of alternatives
                let first_group = &alts[0];
                let rest_groups = &alts[1..];

                for item in first_group {
                    let mut new_current = current.clone();
                    new_current.push(item.clone());
                    recurse(rest_groups, core, new_current, combinations);
                }
            }
        }

        // Start the recursive generation
        recurse(&alts, &core, vec![], &mut combinations);

        combinations
    }

    fn parse_criteria(criteria: &Criteria) -> Vec<super::Criteria> {
        let mut alts = Vec::new();
        let mut core = Vec::new();

        for fact in &criteria.facts {
            // It's an Or
            if fact.contains("||") {
                let mut alt_set = Vec::new();
                let splits = fact.split(" || ").collect::<Vec<_>>();
                for split in splits {
                    alt_set.push(parse_criterion(split));
                }
                alts.push(alt_set);
            } else {
                core.push(parse_criterion(fact));
            }
        }

        if alts.is_empty() {
            return vec![super::Criteria {
                concept: criteria.concept,
                criterion: core,
            }];
        }

        let mut permutations = generate_combinations_with_core_vec(alts, core);

        let mut result = Vec::new();
        for perm in &mut permutations {
            result.push(super::Criteria {
                concept: criteria.concept,
                criterion: perm.clone(),
            });
        }

        result
    }

    #[derive(Debug, Resource)]
    pub(super) struct RawRuleSetHandle(pub Handle<RawRuleSet>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    lazy_static! {
        static ref EXAMPLE_RAW_RULE_SET: parse::RawRuleSet = parse::RawRuleSet {
            entries: vec![
                parse::Entry::Response(parse::RawResponse {
                    id: "Greet".to_string(),
                    now: vec!["RandomText hello".to_string()],
                    after: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "Greet".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["TimeOfDay = 12.0".to_string()],
                    },
                    response: "Greet".to_string(),
                    apply_facts: vec![],
                }),
                parse::Entry::Response(parse::RawResponse {
                    id: "GreetRaining".to_string(),
                    now: vec!["RandomText raining_dialogue".to_string()],
                    after: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "GreetRaining".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["TimeOfDay = 12.0".to_string(), "IsRaining".to_string()],
                    },
                    response: "GreetRaining".to_string(),
                    apply_facts: vec![],
                }),
                parse::Entry::Response(parse::RawResponse {
                    id: "LunchTime".to_string(),
                    now: vec!["RandomText lunch_dialogue".to_string()],
                    after: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "LunchTime".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["TimeOfDay > 13.0".to_string(), "Hunger > 0.5".to_string()],
                    },
                    response: "LunchTime".to_string(),
                    apply_facts: vec![],
                }),
                parse::Entry::Response(parse::RawResponse {
                    id: "QueryFacts".to_string(),
                    now: vec!["RandomText query_dialogue".to_string()],
                    after: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "Query Facts".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec![
                            "TimeOfDay > 13.0".to_string(),
                            "Hunger > 0.5".to_string(),
                            "IsQueryFact".to_string(),
                        ],
                    },
                    response: "QueryFacts".to_string(),
                    apply_facts: vec![],
                }),
                parse::Entry::Response(parse::RawResponse {
                    id: "InsertGlobalFact".to_string(),
                    now: vec![
                        "RandomText global".to_string(),
                        "AddGlobalFact NewGlobalFact".to_string()
                    ],
                    after: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "InsertGlobalFact".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["DoGlobalFact".to_string(), "NewGlobalFact !".to_string()],
                    },
                    response: "InsertGlobalFact".to_string(),
                    apply_facts: vec![],
                }),
                parse::Entry::Response(parse::RawResponse {
                    id: "InsertEntityFact".to_string(),
                    now: vec![
                        "RandomText entity".to_string(),
                        "AddEntityFact NewEntityFact".to_string()
                    ],
                    after: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "InsertEntityFact".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["DoEntityFact".to_string(), "NewEntityFact !".to_string()],
                    },
                    response: "InsertEntityFact".to_string(),
                    apply_facts: vec![],
                })
            ],
        };
        static ref EXAMPLE_RULE_SET: RuleSet = EXAMPLE_RAW_RULE_SET.clone().into();
    }

    #[test]
    fn test_parse_file() {
        let mut file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        file_path.push("assets/dialogue/main.dialogue.ron");

        let data = std::fs::read_to_string(file_path).unwrap();

        let raw_rule_set: parse::RawRuleSet = ron::from_str(&data).unwrap();

        let rule_set: RuleSet = raw_rule_set.clone().into();

        assert!(!rule_set.rules.is_empty());
    }

    #[test]
    fn test_more_specific_response() {
        let fact_db = FactDb {
            facts: hashmap! {
                "TimeOfDay".to_string() => 12.0,
                "IsRaining".to_string() => 1.0,
            },
        };

        let f_query = FactQuery::new(Concept::ThinkIdle).add_fact_db(&fact_db);

        let response = f_query.run(&EXAMPLE_RULE_SET);

        assert!(response.is_some(), "Eval failed to find a response");
        assert!(response.unwrap().now.get_text()[0].contains("raining"),);
    }

    #[test]
    fn test_response() {
        let fact_db = FactDb {
            facts: hashmap! {
                "TimeOfDay".to_string() => 12.0,
            },
        };

        let fact_query = FactQuery::new(Concept::ThinkIdle).add_fact_db(&fact_db);

        let response = fact_query.run(&EXAMPLE_RULE_SET);

        assert!(response.is_some(), "Eval failed to find a response");
    }

    #[test]
    fn test_response_entity_fact() {
        let global_fact_db = FactDb {
            facts: hashmap! {
                "TimeOfDay".to_string() => 14.0,
            },
        };

        let entity_fact_db = FactDb {
            facts: hashmap! {
                "Hunger".to_string() => 0.6,
            },
        };

        let fact_query = FactQuery::new(Concept::ThinkIdle)
            .add_fact_db(&global_fact_db)
            .add_fact_db(&entity_fact_db);

        let response = fact_query.run(&EXAMPLE_RULE_SET);

        assert!(
            response.unwrap().now.get_text()[0].contains("lunch"),
            "Was unable to find the entity fact"
        );
    }

    #[test]
    fn test_response_query_fact() {
        let global_fact_db = FactDb {
            facts: hashmap! {
                "TimeOfDay".to_string() => 14.0,
            },
        };

        let entity_fact_db = FactDb {
            facts: hashmap! {
                "Hunger".to_string() => 0.6,
            },
        };

        let fact_query = FactQuery::new(Concept::ThinkIdle)
            .add_fact_db(&global_fact_db)
            .add_fact_db(&entity_fact_db)
            .add_fact("IsQueryFact", 1.0);

        let response = fact_query.run(&EXAMPLE_RULE_SET);

        assert!(
            response.unwrap().now.get_text()[0].contains("query"),
            "Was unable to find the Query fact"
        );
    }

    #[test]
    fn test_or_rule() {
        let raw_rule_set: parse::RawRuleSet = parse::RawRuleSet {
            entries: vec![
                parse::Entry::Response(parse::RawResponse {
                    id: "OrRule".to_string(),
                    now: vec!["RandomText or".to_string()],
                    after: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "OrRule".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["A || B".to_string(), "C || D".to_string(), "E".to_string()],
                    },
                    response: "OrRule".to_string(),
                    apply_facts: vec![],
                }),
            ],
        };

        let rule_set: RuleSet = raw_rule_set.clone().into();

        // Single rule should expand into four variations
        assert_eq!(rule_set.rules.len(), 4);

        // A OR B, C OR D, E
        // means
        // A C, E
        // A D, E
        // B C, E
        // B D, E
        let expected_combos = vec![
            "ACE".to_string(),
            "ADE".to_string(),
            "BCE".to_string(),
            "BDE".to_string(),
        ];

        let combos: Vec<_> = rule_set
            .rules
            .iter()
            .map(|r| {
                r.criteria
                    .criterion
                    .iter()
                    .map(|c| c.key.clone())
                    .collect::<String>()
            })
            .collect();

        assert_eq!(combos, expected_combos);
    }

    #[test]
    fn test_response_insert_global_fact() {
        #[derive(Debug, Default)]
        struct Runs(usize);

        fn test_query(
            mut writer: EventWriter<ActionEvent>,
            rule_set: Res<RuleSet>,
            global_fact_db: Res<GlobalFactDatabase>,
            mut local: Local<Runs>,
        ) {
            let query = FactQuery::new(Concept::ThinkIdle)
                .add_fact_db(&global_fact_db.0)
                .add_fact("DoGlobalFact", 1.0);

            let response = query.run(&rule_set);

            match local.0 {
                0 => {
                    assert!(response.is_some(), "Eval failed to find a response");

                    let response = response.unwrap();

                    assert!(response.now.get_text()[0].contains("global"));

                    writer.send(ActionEvent::new(response.now));
                }
                1 => {}
                2 => {
                    println!("Response: {:?}", response);

                    assert!(
                        response.is_none(),
                        "Eval found a response after global fact insert"
                    );
                }
                _ => panic!("Too many runs"),
            }

            local.0 += 1;
        }

        let mut app = App::new();

        let rule_set: RuleSet = EXAMPLE_RAW_RULE_SET.clone().into();

        app.add_event::<ActionEvent>();
        app.add_event::<FactInsert>();
        app.insert_resource(rule_set);
        app.insert_resource(GlobalFactDatabase::default());
        app.add_systems(
            Update,
            (test_query, read_fact_inserts, apply_pending_action).chain(),
        );

        app.update();
        app.update();
        app.update();
    }

    #[test]
    fn test_response_insert_entity_fact() {
        #[derive(Debug, Default)]
        struct Runs(usize);

        #[derive(Component)]
        struct TestTag;

        fn test_query(
            mut writer: EventWriter<ActionEvent>,
            rule_set: Res<RuleSet>,
            mut local: Local<Runs>,
            query: Query<(Entity, &EntityFactDatabase), With<TestTag>>,
        ) {
            let (entity, fact_db) = query.single();

            let query = FactQuery::new(Concept::ThinkIdle)
                .add_fact_db(&fact_db.0)
                .add_fact("DoEntityFact", 1.0);

            let response = query.run(&rule_set);

            match local.0 {
                0 => {
                    assert!(response.is_some(), "Eval failed to find a response");

                    let response = response.unwrap();

                    assert!(response.now.get_text()[0].contains("entity"));

                    writer.send(ActionEvent::new(response.now).with_entity(entity));
                }
                1 => {}
                2 => {
                    println!("Response: {:?}", response);

                    assert!(
                        response.is_none(),
                        "Eval found a response after entity fact insert"
                    );
                }
                _ => panic!("Too many runs"),
            }

            local.0 += 1;
        }

        let mut app = App::new();

        let rule_set: RuleSet = EXAMPLE_RAW_RULE_SET.clone().into();

        app.add_event::<ActionEvent>();
        app.add_event::<FactInsert>();
        app.insert_resource(rule_set);
        app.insert_resource(GlobalFactDatabase::default());
        app.add_systems(
            Update,
            (test_query, read_fact_inserts, apply_pending_action).chain(),
        );

        app.world_mut()
            .spawn((TestTag, EntityFactDatabase::default()));

        app.update();
        app.update();
        app.update();
    }

    #[test]
    fn test_rules_sorted_by_criterion_count() {
        let raw_rule_set = parse::RawRuleSet {
            entries: vec![
                parse::Entry::Response(parse::RawResponse {
                    id: "Response".to_string(),
                    now: vec!["RandomText response1".to_string()],
                    after: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "Double".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["A".to_string(), "B".to_string()],
                    },
                    response: "Response".to_string(),
                    apply_facts: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "Single".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["A || B".to_string()],
                    },
                    response: "Response".to_string(),
                    apply_facts: vec![],
                }),
                parse::Entry::Rule(parse::RawRule {
                    id: "Triple".to_string(),
                    criteria: parse::Criteria {
                        concept: Concept::ThinkIdle,
                        facts: vec!["A".to_string(), "B".to_string(), "C".to_string()],
                    },
                    response: "Response".to_string(),
                    apply_facts: vec![],
                }),
            ],
        };

        let rule_set: RuleSet = raw_rule_set.clone().into();

        assert_eq!(rule_set.rules[0].id, "Triple");
        assert_eq!(rule_set.rules[1].id, "Double");
        assert_eq!(rule_set.rules[2].id, "Single");
    }
}
