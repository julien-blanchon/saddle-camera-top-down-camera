use bevy::{
    app::PostStartup,
    ecs::schedule::ScheduleLabel,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll},
    prelude::*,
    time::TimeUpdateStrategy,
};

use super::*;
use crate::{TopDownCamera, TopDownCameraPlugin, TopDownCameraSettings};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct ActivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct DeactivateSchedule;

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct SimulationSchedule;

fn init_input_resources(app: &mut App) {
    app.init_resource::<ButtonInput<KeyCode>>()
        .init_resource::<ButtonInput<MouseButton>>()
        .insert_resource(AccumulatedMouseMotion::default())
        .insert_resource(AccumulatedMouseScroll::default())
        .insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
}

fn spawn_input_camera_with_components(
    app: &mut App,
    is_active: bool,
    input: TopDownCameraInput,
    policy: Option<TopDownCameraInputPolicy>,
) -> Entity {
    let mut entity = app.world_mut().spawn((
        Camera2d,
        Camera {
            is_active,
            ..default()
        },
        TopDownCamera::new(Vec3::ZERO),
        TopDownCameraSettings::default(),
        input,
    ));

    if let Some(policy) = policy {
        entity.insert(policy);
    }

    entity.id()
}

fn spawn_input_camera(app: &mut App, is_active: bool) -> Entity {
    spawn_input_camera_with_components(
        app,
        is_active,
        TopDownCameraInput::default(),
        Some(TopDownCameraInputPolicy::default()),
    )
}

#[test]
fn default_input_enables_expected_features() {
    let input = TopDownCameraInput::default();
    assert!(input.keyboard_pan_enabled);
    assert!(input.mouse_drag_enabled);
    assert!(input.scroll_zoom_enabled);
    assert!(input.zoom_to_cursor);
    assert!(!input.edge_scroll_enabled);
    assert!(input.keyboard_rotate_enabled);
    assert!(input.keyboard_zoom_enabled);
}

#[test]
fn default_policy_uses_active_camera_filter_and_bindings() {
    let policy = TopDownCameraInputPolicy::default();
    assert_eq!(
        policy.target_filter,
        TopDownCameraInputTargetFilter::ActiveCamera
    );
    assert_eq!(
        policy.bindings.keyboard_pan_x,
        TopDownCameraKeyAxisBinding::new(
            [KeyCode::KeyA, KeyCode::ArrowLeft],
            [KeyCode::KeyD, KeyCode::ArrowRight],
        )
    );
    assert_eq!(
        policy.bindings.mouse_drag_buttons,
        vec![MouseButton::Middle]
    );
}

#[test]
fn binding_table_reads_custom_keys() {
    let bindings = TopDownCameraInputBindingTable {
        keyboard_pan_x: TopDownCameraKeyAxisBinding::new([KeyCode::KeyJ], [KeyCode::KeyL]),
        keyboard_pan_y: TopDownCameraKeyAxisBinding::new([KeyCode::KeyK], [KeyCode::KeyI]),
        keyboard_rotate: TopDownCameraKeyAxisBinding::new([KeyCode::KeyU], [KeyCode::KeyO]),
        keyboard_zoom: TopDownCameraKeyAxisBinding::new([KeyCode::KeyN], [KeyCode::KeyM]),
        mouse_drag_buttons: vec![MouseButton::Right],
    };

    let mut keys = ButtonInput::default();
    keys.press(KeyCode::KeyL);
    keys.press(KeyCode::KeyI);
    keys.press(KeyCode::KeyU);
    keys.press(KeyCode::KeyM);

    let mut mouse_buttons = ButtonInput::default();
    mouse_buttons.press(MouseButton::Right);

    assert_eq!(bindings.keyboard_pan(&keys), Vec2::new(1.0, 1.0));
    assert_eq!(bindings.keyboard_rotate(&keys), -1.0);
    assert_eq!(bindings.keyboard_zoom(&keys), 1.0);
    assert!(bindings.mouse_drag_active(&mouse_buttons));
}

#[test]
fn input_requires_policy_component() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins((
        TopDownCameraPlugin::default(),
        TopDownCameraInputPlugin::default(),
    ));
    init_input_resources(&mut app);

    let controlled = spawn_input_camera(&mut app, true);
    let uncontrolled =
        spawn_input_camera_with_components(&mut app, true, TopDownCameraInput::default(), None);

    app.finish();
    app.world_mut().run_schedule(PostStartup);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    app.update();

    let controlled = app
        .world()
        .get::<TopDownCamera>(controlled)
        .expect("controlled camera exists");
    let uncontrolled = app
        .world()
        .get::<TopDownCamera>(uncontrolled)
        .expect("uncontrolled camera exists");

    assert!(controlled.target_anchor.x > 0.0);
    assert_eq!(uncontrolled.target_anchor, Vec3::ZERO);
}

#[test]
fn custom_bindings_drive_runtime_keyboard_pan() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins((
        TopDownCameraPlugin::default(),
        TopDownCameraInputPlugin::default(),
    ));
    init_input_resources(&mut app);

    let camera = spawn_input_camera_with_components(
        &mut app,
        true,
        TopDownCameraInput::default(),
        Some(TopDownCameraInputPolicy {
            bindings: TopDownCameraInputBindingTable {
                keyboard_pan_x: TopDownCameraKeyAxisBinding::new([KeyCode::KeyJ], [KeyCode::KeyL]),
                keyboard_pan_y: TopDownCameraKeyAxisBinding::new([KeyCode::KeyK], [KeyCode::KeyI]),
                ..TopDownCameraInputBindingTable::default()
            },
            ..default()
        }),
    );

    app.finish();
    app.world_mut().run_schedule(PostStartup);
    app.update();

    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    app.update();

    let camera_after_default_key = app
        .world()
        .get::<TopDownCamera>(camera)
        .expect("camera exists");
    assert_eq!(camera_after_default_key.target_anchor, Vec3::ZERO);

    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release(KeyCode::KeyD);
        keys.press(KeyCode::KeyL);
    }
    app.update();

    let camera = app
        .world()
        .get::<TopDownCamera>(camera)
        .expect("camera exists");
    assert!(camera.target_anchor.x > 0.0);
}

#[test]
fn inactive_cameras_do_not_consume_default_keyboard_input() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins((
        TopDownCameraPlugin::default(),
        TopDownCameraInputPlugin::default(),
    ));
    init_input_resources(&mut app);

    let active_camera = spawn_input_camera(&mut app, true);
    let inactive_camera = spawn_input_camera(&mut app, false);

    app.finish();
    app.world_mut().run_schedule(PostStartup);
    app.update();
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    app.update();

    let active = app
        .world()
        .get::<TopDownCamera>(active_camera)
        .expect("active camera exists");
    let inactive = app
        .world()
        .get::<TopDownCamera>(inactive_camera)
        .expect("inactive camera exists");

    assert!(active.target_anchor.x > 0.0);
    assert_eq!(inactive.target_anchor, Vec3::ZERO);
}

#[test]
fn input_plugin_runs_on_configured_schedule() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_schedule(ActivateSchedule)
        .init_schedule(DeactivateSchedule)
        .init_schedule(SimulationSchedule)
        .add_plugins(TopDownCameraPlugin::new(
            ActivateSchedule,
            DeactivateSchedule,
            SimulationSchedule,
        ))
        .add_plugins(TopDownCameraInputPlugin::new(SimulationSchedule));
    init_input_resources(&mut app);

    let camera = spawn_input_camera(&mut app, true);

    app.finish();
    app.world_mut().run_schedule(ActivateSchedule);
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f64(1.0 / 60.0));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyD);
    app.world_mut().run_schedule(SimulationSchedule);

    let camera = app
        .world()
        .get::<TopDownCamera>(camera)
        .expect("camera exists");
    assert!(camera.target_anchor.x > 0.0);
}
