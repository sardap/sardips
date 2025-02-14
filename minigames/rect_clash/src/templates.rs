use std::ops::Range;

use bevy::prelude::*;
use sardips_core::rgb_to_color;

pub struct TemplatePlugin;

impl Plugin for TemplatePlugin {
    fn build(&self, _: &mut App) {
        // app.add_systems(
        //     Update,
        //     insert_battler_templates.run_if(resource_added::<PetTemplateDatabase>),
        // );
    }
}

pub enum BattlerBrainTypes {
    Independent,
}

pub struct BaseBattlerTemplate {
    #[allow(dead_code)]
    pub weight_range: Range<i32>,
    #[allow(dead_code)]
    pub color: Color,
    pub size: Vec3,
    pub col_discipline: f32,
    #[allow(dead_code)]
    pub brain: BattlerBrainTypes,
}

pub const LINE_MAN_TEMPLATE: BaseBattlerTemplate = BaseBattlerTemplate {
    weight_range: Range::<i32> { start: 70, end: 80 },
    color: rgb_to_color!(0, 0, 0),
    size: Vec3::new(0.6, 0.6, 0.6),
    col_discipline: 0.08,
    brain: BattlerBrainTypes::Independent,
};

// pub struct BattlerTemplate {
//     base: &'static BaseBattlerTemplate,
//     sprite: PreCalculated,
// }

// impl BattlerTemplate {
//     pub fn new(base: &'static BaseBattlerTemplate, sprite: PreCalculated) -> Self {
//         Self { base, sprite }
//     }
// }

// #[derive(Resource)]
// pub struct InstantiatedBattlerTemplate {
//     line_man: BattlerTemplate,
// }

// fn find_pet_based_on_base_template<T: DelegatedRng>(
//     rng: &mut T,
//     base: &BaseBattlerTemplate,
//     pet_db: &PetTemplateDatabase,
// ) -> PreCalculated {
//     let possible = pet_db
//         .iter()
//         .filter(|pet| base.weight_classes.iter().any(|w| Some(pet.weight) == *w))
//         .collect::<Vec<_>>();

//     if possible.is_empty() {
//         panic!("No pets found for base template");
//     }

//     possible[rng.usize(0..possible.len())]
//         .pre_calculated
//         .clone()
// }

// impl InstantiatedBattlerTemplate {
//     pub fn new<T: DelegatedRng>(rng: &mut T, pet_db: &PetTemplateDatabase) -> Self {
//         Self {
//             line_man: BattlerTemplate::new(
//                 &LINE_MAN_TEMPLATE,
//                 find_pet_based_on_base_template(rng, &LINE_MAN_TEMPLATE, pet_db),
//             ),
//         }
//     }
// }

// fn insert_battler_templates(
//     mut commands: Commands,
//     global_rng: ResMut<GlobalRng>,
//     pet_db: Res<PetTemplateDatabase>,
// ) {
//     let rng = global_rng.into_inner();

//     let instantiated = InstantiatedBattlerTemplate::new(rng, &pet_db);

//     commands.insert_resource(instantiated);
// }
