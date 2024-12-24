use std::time::Duration;

use bevy::prelude::*;
use shared_deps::serde::{Deserialize, Serialize};

use crate::fact_str_hash;

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
pub struct RawRuleSet {
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
pub struct RawRuleSetHandle(pub Handle<RawRuleSet>);
