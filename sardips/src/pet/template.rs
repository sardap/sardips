use bevy::prelude::*;
use fact_db::EntityFactDatabase;
use sardips_core::food_core::FoodPreference;
use sardips_core::mood_core::MoodCategoryHistory;
use sardips_core::name::{EntityName, SpeciesName};
use sardips_core::pet_core::{EvolvingPet, PetTemplate, PetTemplateDatabase};
use serde::Deserialize;
use shared_deps::bevy_common_assets::ron::RonAssetPlugin;

use crate::accessory::AccessoryBundle;
use crate::layering;
use sardips_core::{text_database::TextDatabase, velocity::Speed, GameState};

use super::PetBundle;

pub struct PetTemplatePlugin;

impl Plugin for PetTemplatePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPetEvent>()
            .add_plugins(RonAssetPlugin::<AssetPetTemplateSet>::new(&["pets.ron"]))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                spawn_pending_pets.run_if(not(in_state(GameState::Loading))),
            )
            .add_systems(
                Update,
                load_templates.run_if(not(resource_exists::<PetTemplateDatabase>)),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let template_set = PetTemplateSetHandle(asset_server.load("pets/complete.pets.ron"));
    commands.insert_resource(template_set);
}

fn load_templates(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    template_handle: Res<PetTemplateSetHandle>,
    mut template_assets: ResMut<Assets<AssetPetTemplateSet>>,
) {
    if let Some(set) = template_assets.remove(template_handle.0.id()) {
        let mut db = PetTemplateDatabase {
            templates: set.templates,
        };

        db.populate_pre_calculated(&asset_server, &mut layouts);

        commands.insert_resource(db);
    }
}

pub fn spawn_pet(
    template: &PetTemplate,
    commands: &mut Commands,
    location: Vec2,
    name: EntityName,
) -> Entity {
    let entity_id = commands
        .spawn(PetBundle {
            species_name: SpeciesName::new(&template.species_name),
            name,
            mood_category_history: MoodCategoryHistory::default(),
            fact_db: EntityFactDatabase::default(),
            kind: template.kind,
            speed: Speed(template.speed.value()),
            transform: Transform::from_xyz(location.x, location.y, layering::view_screen::PET),
            ..default()
        })
        .id();

    if let Some(breeds) = template.get_breeds() {
        commands.entity(entity_id).insert(breeds);
    }

    if let Some(hunger) = template.get_hunger() {
        commands.entity(entity_id).insert((
            hunger,
            FoodPreference {
                sensation_ratings: template.stomach.as_ref().unwrap().sensations.clone(),
            },
        ));
    }

    if let Some(fun) = template.get_fun() {
        commands.entity(entity_id).insert(fun);
    }

    if let Some(pooper) = template.get_pooper() {
        commands.entity(entity_id).insert(pooper);
    }

    if let Some(cleanliness) = template.get_cleanliness() {
        commands.entity(entity_id).insert(cleanliness);
    }

    if let Some(money_hungry) = template.get_money_hungry() {
        commands.entity(entity_id).insert(money_hungry);
    }

    commands.entity(entity_id).with_children(|parent| {
        parent.spawn(AccessoryBundle { ..default() });
    });

    entity_id
}

fn evolve_pet(template: &PetTemplate, commands: &mut Commands, evolving: EvolvingPet) {
    // Create new delete old

    commands.entity(evolving.entity).despawn_recursive();

    let new_entity = spawn_pet(template, commands, evolving.location, evolving.name);

    commands.entity(new_entity).insert(evolving.age);
    commands.entity(new_entity).insert(evolving.mood_history);
    commands.entity(new_entity).insert(evolving.fact_db);
}

#[derive(Deserialize, Asset, TypePath)]
pub struct AssetPetTemplateSet {
    pub templates: Vec<PetTemplate>,
}

#[derive(Debug, Resource)]
struct PetTemplateSetHandle(Handle<AssetPetTemplateSet>);

#[derive(Event)]
pub enum SpawnPetEvent {
    Blank((Vec2, String)),
    Evolve((String, EvolvingPet)),
}

impl SpawnPetEvent {
    fn species_name(&self) -> &str {
        match self {
            SpawnPetEvent::Blank((_, species_name)) => species_name,
            SpawnPetEvent::Evolve((species_name, _)) => species_name,
        }
    }
}

fn spawn_pending_pets(
    mut commands: Commands,
    mut events: EventReader<SpawnPetEvent>,
    pet_template_db: Res<PetTemplateDatabase>,
    text_db: Res<TextDatabase>,
) {
    for event in events.read() {
        info!("Spawning pet: {:?}", event.species_name());
        if let Some(template) = pet_template_db.get_by_name(event.species_name()) {
            match event {
                SpawnPetEvent::Blank((pos, _)) => {
                    spawn_pet(template, &mut commands, *pos, EntityName::random(&text_db));
                }
                SpawnPetEvent::Evolve((_, evolving)) => {
                    evolve_pet(template, &mut commands, evolving.clone());
                }
            }
        } else {
            error!(
                "Unable to find template for species {}",
                event.species_name()
            );
        }
    }
}
