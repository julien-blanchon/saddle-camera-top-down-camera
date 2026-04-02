use bevy::{app::PostStartup, ecs::schedule::ScheduleLabel, prelude::*, time::TimeUpdateStrategy};

use crate::{
    TopDownCamera, TopDownCameraPlugin, TopDownCameraRuntime, TopDownCameraSettings,
    TopDownCameraSystems,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct ActivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct SimulationSchedule;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AfterRuntime;

#[derive(Resource, Default, Debug, PartialEq, Eq)]
struct OrderLog(Vec<&'static str>);

fn spawn_camera(app: &mut App) {
    app.world_mut().spawn((
        Camera2d,
        TopDownCamera::new(Vec3::ZERO),
        TopDownCameraSettings::default(),
    ));
}

fn start_runtime(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.finish();
    app.world_mut().run_schedule(PostStartup);
}

fn push_runtime_marker(mut log: ResMut<OrderLog>) {
    log.0.push("runtime");
}

fn push_after_marker(mut log: ResMut<OrderLog>) {
    log.0.push("after");
}

#[test]
fn plugin_builds_with_custom_schedule_labels_and_ordering_points() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .init_resource::<OrderLog>()
        .add_plugins(TopDownCameraPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ))
        .configure_sets(
            SimulationSchedule,
            TopDownCameraSystems::ApplySmoothing.before(AfterRuntime),
        )
        .add_systems(
            SimulationSchedule,
            (
                push_runtime_marker.in_set(TopDownCameraSystems::ApplySmoothing),
                push_after_marker.in_set(AfterRuntime),
            ),
        );

    spawn_camera(&mut app);
    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut().run_schedule(SimulationSchedule);

    assert_eq!(
        app.world().resource::<OrderLog>().0,
        vec!["runtime", "after"]
    );
}

#[test]
fn always_on_constructor_initializes_runtime() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    spawn_camera(&mut app);
    start_runtime(&mut app);
    app.update();

    let mut query = app.world_mut().query::<&TopDownCameraRuntime>();
    assert!(query.single(app.world()).is_ok());
}

#[test]
fn deactivate_schedule_stops_runtime_updates() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .add_plugins(TopDownCameraPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ));

    spawn_camera(&mut app);
    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut().run_schedule(SimulationSchedule);

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopDownCamera>>()
        .single(app.world())
        .expect("camera exists");

    {
        let mut camera = app
            .world_mut()
            .get_mut::<TopDownCamera>(entity)
            .expect("camera exists");
        camera.snap_to(Vec3::new(8.0, 0.0, 0.0), 0.5, 2.0);
    }

    app.world_mut().run_schedule(DeactivateSchedule);
    let before = app
        .world()
        .get::<TopDownCameraRuntime>(entity)
        .cloned()
        .expect("runtime exists");
    app.world_mut().run_schedule(SimulationSchedule);
    let after = app
        .world()
        .get::<TopDownCameraRuntime>(entity)
        .cloned()
        .expect("runtime exists");

    assert_eq!(before.follow_anchor, after.follow_anchor);
    assert_eq!(before.yaw, after.yaw);
    assert_eq!(before.zoom, after.zoom);
}
