use bevy::{
    camera::{Projection, ScalingMode},
    prelude::*,
    time::TimeUpdateStrategy,
};

use crate::{
    TopDownCamera, TopDownCameraBounds, TopDownCameraMode, TopDownCameraPlugin,
    TopDownCameraRuntime, TopDownCameraSettings, TopDownCameraTarget,
};

fn start(app: &mut App) {
    app.insert_resource(TimeUpdateStrategy::ManualDuration(
        std::time::Duration::from_secs_f64(1.0 / 60.0),
    ));
    app.finish();
    app.world_mut().run_schedule(bevy::app::PostStartup);
    app.update();
}

fn spawn_2d_camera(app: &mut App, settings: TopDownCameraSettings) -> Entity {
    app.world_mut()
        .spawn((Camera2d, TopDownCamera::new(Vec3::ZERO), settings))
        .id()
}

fn spawn_3d_camera(app: &mut App, settings: TopDownCameraSettings) -> Entity {
    app.world_mut()
        .spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection::default()),
            TopDownCamera::looking_at_3d(Vec3::ZERO, 0.0, 12.0),
            settings,
        ))
        .id()
}

fn spawn_3d_orthographic_camera(app: &mut App, settings: TopDownCameraSettings) -> Entity {
    app.world_mut()
        .spawn((
            Camera3d::default(),
            Projection::Orthographic(OrthographicProjection {
                scale: 1.25,
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 18.0,
                },
                ..OrthographicProjection::default_3d()
            }),
            TopDownCamera::looking_at_3d(Vec3::ZERO, 0.0, 1.25),
            settings,
        ))
        .id()
}

fn spawn_target(app: &mut App, translation: Vec3, priority: i32) -> Entity {
    app.world_mut()
        .spawn((
            Transform::from_translation(translation),
            TopDownCameraTarget {
                priority,
                ..default()
            },
        ))
        .id()
}

#[test]
fn automatic_target_selection_prefers_highest_priority_enabled_target() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(&mut app, TopDownCameraSettings::default());
    spawn_target(&mut app, Vec3::new(-80.0, 0.0, 0.0), 1);
    let disabled = spawn_target(&mut app, Vec3::new(60.0, 0.0, 0.0), 50);
    app.world_mut()
        .entity_mut(disabled)
        .insert(TopDownCameraTarget {
            priority: 50,
            enabled: false,
            ..default()
        });
    let preferred = spawn_target(&mut app, Vec3::new(30.0, 0.0, 0.0), 10);
    start(&mut app);

    let runtime = app
        .world()
        .get::<TopDownCameraRuntime>(camera)
        .expect("runtime exists");
    assert_eq!(runtime.active_target, Some(preferred));
}

#[test]
fn camera_keeps_existing_goal_when_no_target_is_available() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(
        &mut app,
        TopDownCameraSettings {
            dead_zone: Vec2::new(80.0, 80.0),
            ..default()
        },
    );
    start(&mut app);

    let runtime = app
        .world()
        .get::<TopDownCameraRuntime>(camera)
        .expect("runtime exists");
    assert_eq!(runtime.active_target, None);
    assert_eq!(runtime.goal_anchor, Vec3::ZERO);
    assert_eq!(runtime.tracked_point, Vec3::ZERO);
}

#[test]
fn camera_stays_put_while_target_is_inside_dead_zone() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(
        &mut app,
        TopDownCameraSettings {
            dead_zone: Vec2::new(100.0, 100.0),
            ..default()
        },
    );
    spawn_target(&mut app, Vec3::new(24.0, 0.0, 0.0), 1);
    start(&mut app);

    let runtime = app
        .world()
        .get::<TopDownCameraRuntime>(camera)
        .expect("runtime exists");
    assert_eq!(runtime.goal_anchor, Vec3::ZERO);
}

#[test]
fn camera_moves_only_the_excess_distance_after_leaving_dead_zone() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(
        &mut app,
        TopDownCameraSettings {
            dead_zone: Vec2::new(100.0, 100.0),
            ..default()
        },
    );
    spawn_target(&mut app, Vec3::new(78.0, 0.0, 0.0), 1);
    start(&mut app);

    let camera = app
        .world()
        .get::<TopDownCamera>(camera)
        .expect("camera exists");
    assert!((camera.target_anchor.x - 28.0).abs() < 0.001);
}

#[test]
fn soft_zone_reduces_recentering_inside_outer_frame() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(
        &mut app,
        TopDownCameraSettings {
            dead_zone: Vec2::new(100.0, 100.0),
            soft_zone: Vec2::new(180.0, 180.0),
            ..default()
        },
    );
    spawn_target(&mut app, Vec3::new(78.0, 0.0, 0.0), 1);
    start(&mut app);

    let camera = app
        .world()
        .get::<TopDownCamera>(camera)
        .expect("camera exists");
    assert!(camera.target_anchor.x > 0.0);
    assert!(camera.target_anchor.x < 28.0);
}

#[test]
fn follow_can_be_suspended_without_losing_manual_camera_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(
        &mut app,
        TopDownCameraSettings {
            dead_zone: Vec2::new(100.0, 100.0),
            ..default()
        },
    );
    spawn_target(&mut app, Vec3::new(220.0, 0.0, 0.0), 10);
    start(&mut app);

    {
        let mut controller = app
            .world_mut()
            .get_mut::<TopDownCamera>(camera)
            .expect("camera exists");
        controller.follow_enabled = false;
        controller.target_anchor = Vec3::new(12.0, -4.0, 0.0);
        controller.snap = true;
    }
    app.update();
    app.update();

    let controller = app
        .world()
        .get::<TopDownCamera>(camera)
        .expect("camera exists");
    let runtime = app
        .world()
        .get::<TopDownCameraRuntime>(camera)
        .expect("runtime exists");
    assert_eq!(controller.target_anchor, Vec3::new(12.0, -4.0, 0.0));
    assert_eq!(runtime.goal_anchor, Vec3::new(12.0, -4.0, 0.0));
    assert_eq!(runtime.follow_anchor, Vec3::new(12.0, -4.0, 0.0));
}

#[test]
fn bounds_clamp_follow_goal() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(
        &mut app,
        TopDownCameraSettings {
            dead_zone: Vec2::new(20.0, 20.0),
            bounds: Some(TopDownCameraBounds {
                min: Vec2::new(-10.0, -8.0),
                max: Vec2::new(10.0, 8.0),
            }),
            ..default()
        },
    );
    spawn_target(&mut app, Vec3::new(90.0, 0.0, 0.0), 1);
    start(&mut app);

    let camera = app
        .world()
        .get::<TopDownCamera>(camera)
        .expect("camera exists");
    assert_eq!(camera.target_anchor.x, 10.0);
}

#[test]
fn orthographic_3d_uses_zoom_for_projection_scale_but_keeps_fixed_arm_distance() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_3d_orthographic_camera(
        &mut app,
        TopDownCameraSettings {
            mode: TopDownCameraMode::tilted_3d(60.0_f32.to_radians(), 18.0),
            zoom_min: 0.5,
            zoom_max: 3.0,
            ..default()
        },
    );
    start(&mut app);

    {
        let mut controller = app
            .world_mut()
            .get_mut::<TopDownCamera>(camera)
            .expect("camera exists");
        controller.snap_to(Vec3::new(0.0, 1.0, 0.0), 0.0, 2.0);
    }
    app.update();

    let transform = app
        .world()
        .get::<Transform>(camera)
        .expect("transform exists");
    let projection = app
        .world()
        .get::<Projection>(camera)
        .expect("projection exists");
    let Projection::Orthographic(orthographic) = projection else {
        panic!("expected orthographic projection");
    };

    assert!((orthographic.scale - 2.0).abs() < 0.001);
    assert!((transform.translation.y - 16.588457).abs() < 0.01);
    assert!((transform.translation.z - 9.0).abs() < 0.01);
}

#[test]
fn tracked_target_can_be_swapped_at_runtime() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(&mut app, TopDownCameraSettings::default());
    let target_a = spawn_target(&mut app, Vec3::new(-20.0, 0.0, 0.0), 0);
    let target_b = spawn_target(&mut app, Vec3::new(40.0, 0.0, 0.0), 0);
    {
        let mut controller = app
            .world_mut()
            .get_mut::<TopDownCamera>(camera)
            .expect("camera exists");
        controller.tracked_target = Some(target_a);
    }
    start(&mut app);

    {
        let runtime = app
            .world()
            .get::<TopDownCameraRuntime>(camera)
            .expect("runtime exists");
        assert_eq!(runtime.active_target, Some(target_a));
    }

    {
        let mut controller = app
            .world_mut()
            .get_mut::<TopDownCamera>(camera)
            .expect("camera exists");
        controller.tracked_target = Some(target_b);
    }
    app.update();

    let runtime = app
        .world()
        .get::<TopDownCameraRuntime>(camera)
        .expect("runtime exists");
    assert_eq!(runtime.active_target, Some(target_b));
}

#[test]
fn orthographic_zoom_updates_projection_scale() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_2d_camera(&mut app, TopDownCameraSettings::default());
    start(&mut app);

    {
        let mut controller = app
            .world_mut()
            .get_mut::<TopDownCamera>(camera)
            .expect("camera exists");
        controller.snap_to(Vec3::ZERO, 0.0, 2.5);
    }
    app.update();

    let projection = app
        .world()
        .get::<Projection>(camera)
        .expect("projection exists");
    let Projection::Orthographic(orthographic) = projection else {
        panic!("expected orthographic projection");
    };
    assert!((orthographic.scale - 2.5).abs() < 0.001);
}

#[test]
fn perspective_3d_uses_zoom_as_camera_distance() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(TopDownCameraPlugin::always_on(Update));

    let camera = spawn_3d_camera(
        &mut app,
        TopDownCameraSettings {
            mode: TopDownCameraMode::tilted_3d(60.0_f32.to_radians(), 18.0),
            zoom_min: 6.0,
            zoom_max: 40.0,
            ..default()
        },
    );
    start(&mut app);

    {
        let mut controller = app
            .world_mut()
            .get_mut::<TopDownCamera>(camera)
            .expect("camera exists");
        controller.snap_to(Vec3::new(0.0, 1.0, 0.0), 0.0, 20.0);
    }
    app.update();

    let transform = app
        .world()
        .get::<Transform>(camera)
        .expect("transform exists");
    assert!((transform.translation.y - 18.320509).abs() < 0.01);
    assert!((transform.translation.z - 10.0).abs() < 0.01);
}
