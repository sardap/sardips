use std::time::Duration;

use bevy::{
    app::{MainScheduleOrder, RunFixedMainLoop},
    ecs::schedule::ScheduleLabel,
    prelude::*,
};
use sardips_core::{from_days, from_hours, from_mins};
use shared_deps::chrono::{DateTime, Utc};
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Simulated>();

        app.insert_state(SimulationState::default())
            .insert_state(SimulationViewState::default())
            .insert_resource(SimTimeScale(1.0))
            .add_systems(
                OnEnter(SimulationViewState::Visible),
                sim_entities_visibility,
            )
            .add_systems(
                OnEnter(SimulationViewState::Invisible),
                sim_entities_visibility,
            );

        app.insert_resource(Time::<SimTime>::default())
            .init_schedule(RunSimulationUpdate)
            .init_schedule(SimulationUpdate)
            .add_systems(RunSimulationUpdate, run_simulation_schedule);

        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_after(RunFixedMainLoop, RunSimulationUpdate);
    }
}

pub const HUNGER_MOOD_UPDATE: Duration = from_mins(2);
pub const FUN_MOOD_UPDATE: Duration = from_mins(2);
pub const MONEY_MOOD_UPDATE: Duration = from_hours(2);
pub const CLEANLINESS_MOOD_UPDATE: Duration = from_mins(2);
pub const EGG_HATCH_ATTEMPT_INTERVAL: Duration = from_mins(30);
pub const MAX_EGG_LIFE: Duration = from_days(2);

// Tick down rates one point per seconds
// One point per minute
pub const HUNGER_TICK_DOWN: f32 = 2. / 60.;
// One point per 120 seconds
pub const FUN_TICK_DOWN: f32 = 1. / 120.;

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum SimulationViewState {
    #[default]
    Invisible,
    Visible,
}

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum SimulationState {
    #[default]
    Paused,
    Running,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Simulated;

fn sim_entities_visibility(
    sim_state: Res<State<SimulationViewState>>,
    mut visibility: Query<&mut Visibility>,
    mut query: Query<(Entity, Option<&Children>), With<Simulated>>,
) {
    let mut update_vis = |entity| {
        if let Ok(mut visibility) = visibility.get_mut(entity) {
            *visibility = match **sim_state {
                SimulationViewState::Invisible => Visibility::Hidden,
                SimulationViewState::Visible => Visibility::Visible,
            };
        }
    };

    for (entity, children) in &mut query.iter_mut() {
        update_vis(entity);
        if let Some(children) = children {
            for entity in children.iter() {
                update_vis(*entity);
            }
        }
    }
}

#[derive(Debug)]
pub struct SimTime {
    overstep: Duration,
    timestep: Duration,
    last_run: DateTime<Utc>,
}

impl Default for SimTime {
    fn default() -> Self {
        Self {
            overstep: Duration::default(),
            timestep: Duration::from_secs(1),
            last_run: Utc::now(),
        }
    }
}

pub trait SimTimeTrait {
    fn expend(&mut self) -> bool;

    fn accumulate(&mut self, now: DateTime<Utc>, scale: f32);

    fn last_run(&self) -> DateTime<Utc>;

    fn set_last_run(&mut self, last_run: DateTime<Utc>);
}

impl SimTimeTrait for Time<SimTime> {
    fn expend(&mut self) -> bool {
        let timestep = self.context().timestep;
        if let Some(new_value) = self.context().overstep.checked_sub(timestep) {
            self.context_mut().overstep = new_value;
            self.advance_by(timestep);
            true
        } else {
            false
        }
    }

    fn accumulate(&mut self, now: DateTime<Utc>, scale: f32) {
        let last_run = self.context().last_run;
        let delta = now - last_run;
        let delta: Duration = Duration::from_nanos(delta.num_nanoseconds().unwrap() as u64);
        self.context_mut().overstep += delta.mul_f32(scale);
        self.context_mut().last_run = now;
    }

    fn last_run(&self) -> DateTime<Utc> {
        self.context().last_run
    }

    fn set_last_run(&mut self, last_run: DateTime<Utc>) {
        self.context_mut().last_run = last_run;
    }
}

#[derive(Resource)]
pub struct SimTimeScale(pub f32);

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SimulationUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct RunSimulationUpdate;

pub fn run_simulation_schedule(world: &mut World) {
    if world.get_resource::<Time<SimTime>>().is_none() {
        return;
    }

    let time_scale = world.resource::<SimTimeScale>().0;

    // Continue to accumulate time even if sim is not running
    world
        .resource_mut::<Time<SimTime>>()
        .accumulate(Utc::now(), time_scale);

    if matches!(
        **world.resource::<State<SimulationState>>(),
        SimulationState::Paused
    ) {
        return;
    }

    // Run the schedule until we run out of accumulated time
    let _ = world.try_schedule_scope(SimulationUpdate, |world, schedule| {
        while world.resource_mut::<Time<SimTime>>().expend() {
            *world.resource_mut::<Time>() = world.resource::<Time<SimTime>>().as_generic();
            schedule.run(world);
        }
    });

    *world.resource_mut::<Time>() = world.resource::<Time<Virtual>>().as_generic();
}
