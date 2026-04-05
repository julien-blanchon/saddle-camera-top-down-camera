mod components;
mod debug;
pub mod input;
mod math;
mod systems;

pub use components::{
    TopDownCamera, TopDownCameraBounds, TopDownCameraDamping, TopDownCameraDebug,
    TopDownCameraMode, TopDownCameraRuntime, TopDownCameraSettings, TopDownCameraTarget,
};
pub use input::{TopDownCameraInput, TopDownCameraInputPlugin};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    gizmos::{config::DefaultGizmoConfigGroup, gizmos::GizmoStorage},
    prelude::*,
    transform::TransformSystems,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TopDownCameraSystems {
    ResolveTarget,
    ComputeGoal,
    ApplySmoothing,
    SyncTransform,
    SyncProjection,
    DebugDraw,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

#[derive(Resource, Default)]
struct TopDownCameraRuntimeActive(bool);

pub struct TopDownCameraPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl TopDownCameraPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for TopDownCameraPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for TopDownCameraPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<TopDownCameraRuntimeActive>()
            .register_type::<TopDownCamera>()
            .register_type::<TopDownCameraBounds>()
            .register_type::<TopDownCameraDamping>()
            .register_type::<TopDownCameraDebug>()
            .register_type::<TopDownCameraMode>()
            .register_type::<TopDownCameraRuntime>()
            .register_type::<TopDownCameraSettings>()
            .register_type::<TopDownCameraTarget>()
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(self.deactivate_schedule, deactivate_runtime)
            .add_systems(
                self.activate_schedule,
                (
                    systems::initialize_added_targets,
                    systems::initialize_added_cameras,
                ),
            )
            .configure_sets(
                self.update_schedule,
                (
                    TopDownCameraSystems::ResolveTarget,
                    TopDownCameraSystems::ComputeGoal,
                    TopDownCameraSystems::ApplySmoothing,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::initialize_added_targets,
                    systems::initialize_added_cameras,
                    systems::capture_target_motion,
                    systems::resolve_follow_targets,
                )
                    .chain()
                    .in_set(TopDownCameraSystems::ResolveTarget)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::clamp_programmatic_goal
                    .in_set(TopDownCameraSystems::ComputeGoal)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                systems::advance_runtime
                    .in_set(TopDownCameraSystems::ApplySmoothing)
                    .run_if(runtime_is_active),
            )
            .configure_sets(
                PostUpdate,
                (
                    TopDownCameraSystems::SyncTransform,
                    TopDownCameraSystems::SyncProjection,
                    TopDownCameraSystems::DebugDraw,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                systems::sync_transform
                    .in_set(TopDownCameraSystems::SyncTransform)
                    .before(TransformSystems::Propagate)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                PostUpdate,
                systems::sync_projection
                    .in_set(TopDownCameraSystems::SyncProjection)
                    .before(TransformSystems::Propagate)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                PostUpdate,
                debug::draw_debug_gizmos
                    .in_set(TopDownCameraSystems::DebugDraw)
                    .run_if(resource_exists::<GizmoStorage<DefaultGizmoConfigGroup, ()>>)
                    .run_if(runtime_is_active),
            );
    }
}

fn activate_runtime(mut runtime: ResMut<TopDownCameraRuntimeActive>) {
    runtime.0 = true;
}

fn deactivate_runtime(mut runtime: ResMut<TopDownCameraRuntimeActive>) {
    runtime.0 = false;
}

fn runtime_is_active(runtime: Res<TopDownCameraRuntimeActive>) -> bool {
    runtime.0
}

#[cfg(test)]
#[path = "plugin_tests.rs"]
mod plugin_tests;
