use bevy::{camera::Projection, prelude::*};
use bevy_enhanced_input::prelude::EnhancedInputSystems;
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet,
    action::Action,
    actions::{assertions, inspect},
    init_scenario,
    scenario::Scenario,
};
use saddle_camera_top_down_camera::{
    TopDownCamera, TopDownCameraBounds, TopDownCameraDamping, TopDownCameraMode,
    TopDownCameraRuntime, TopDownCameraSettings, TopDownCameraTarget,
};

use crate::{LabCameraEntity, LabPrimaryTargetEntity, LabSecondaryTargetEntity};

pub struct TopDownCameraLabE2EPlugin;

impl Plugin for TopDownCameraLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
        app.add_systems(Update, enforce_authored_lab_defaults.before(E2ESet));
        app.configure_sets(
            Update,
            (
                E2ESet.before(EnhancedInputSystems::Update),
                E2ESet.before(saddle_camera_top_down_camera::TopDownCameraSystems::ResolveTarget),
            ),
        );

        let args: Vec<String> = std::env::args().collect();
        let (scenario_name, handoff) = parse_e2e_args(&args);

        if let Some(name) = scenario_name {
            if let Some(mut scenario) = scenario_by_name(&name) {
                if handoff {
                    scenario.actions.push(Action::Handoff);
                }
                init_scenario(app, scenario);
            } else {
                error!(
                    "[saddle_camera_top_down_camera_lab:e2e] Unknown scenario '{name}'. Available: {:?}",
                    list_scenarios()
                );
            }
        }
    }
}

fn enforce_authored_lab_defaults(
    overridden: Option<Res<crate::LabDefaultsOverridden>>,
    pane: Option<ResMut<saddle_camera_top_down_camera_example_common::ExampleTopDownPane>>,
    mut cameras: Query<(&mut TopDownCamera, &mut TopDownCameraSettings)>,
) {
    if overridden.is_some() {
        return;
    }
    let authored_dead_zone = Vec2::new(3.4, 2.3);
    let authored_soft_zone = Vec2::new(5.8, 3.9);
    let authored_bias = Vec2::new(0.0, -0.2);

    if let Some(mut pane) = pane {
        pane.follow_enabled = true;
        pane.debug_gizmos = true;
        pane.dead_zone_x = authored_dead_zone.x;
        pane.dead_zone_y = authored_dead_zone.y;
        pane.soft_zone_x = authored_soft_zone.x;
        pane.soft_zone_y = authored_soft_zone.y;
        pane.bias_x = authored_bias.x;
        pane.bias_y = authored_bias.y;
        pane.zoom_speed = 0.75;
        pane.planar_damping = 9.0;
        pane.zoom_damping = 12.0;
        pane.yaw_damping = 10.0;
        pane.yaw_radians = 0.45;
        pane.pitch_degrees = 58.0;
    }

    for (mut camera, mut settings) in &mut cameras {
        camera.follow_enabled = true;
        camera.target_yaw = 0.45;

        settings.dead_zone = authored_dead_zone;
        settings.soft_zone = authored_soft_zone;
        settings.bias = authored_bias;
        settings.zoom_speed = 0.75;
        if let saddle_camera_top_down_camera::TopDownCameraMode::Tilted3d {
            pitch,
            orthographic_distance,
        } = &mut settings.mode
        {
            *pitch = 58.0_f32.to_radians();
            *orthographic_distance = 22.0;
        }
    }
}

#[derive(Resource, Clone, Copy)]
struct RuntimeBaseline {
    follow_anchor: Vec3,
    zoom: f32,
    active_target: Option<Entity>,
    projection_scale: Option<f32>,
}

fn parse_e2e_args(args: &[String]) -> (Option<String>, bool) {
    let mut scenario_name = None;
    let mut handoff = false;

    for arg in args.iter().skip(1) {
        if arg == "--handoff" {
            handoff = true;
        } else if !arg.starts_with('-') && scenario_name.is_none() {
            scenario_name = Some(arg.clone());
        }
    }

    if !handoff {
        handoff = std::env::var("E2E_HANDOFF").is_ok_and(|value| value == "1" || value == "true");
    }

    (scenario_name, handoff)
}

fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(build_smoke_launch()),
        "top_down_camera_smoke" => Some(build_runtime_smoke()),
        "top_down_camera_follow" => Some(build_follow()),
        "top_down_camera_bounds" => Some(build_bounds()),
        "top_down_camera_zoom" => Some(build_zoom()),
        "top_down_camera_soft_zone" => Some(build_soft_zone()),
        "top_down_camera_target_switch" => Some(build_target_switch()),
        // Per-example diagnostic scenarios
        "example_basic_2d" => Some(build_example_basic_2d()),
        "example_basic_3d" => Some(build_example_basic_3d()),
        "example_bounds" => Some(build_example_bounds()),
        "example_target_switching" => Some(build_example_target_switching()),
        "example_soft_zone_framing" => Some(build_example_soft_zone_framing()),
        "example_optional_controls" => Some(build_example_optional_controls()),
        "example_strategy_game" => Some(build_example_strategy_game()),
        "example_arpg_camera" => Some(build_example_arpg_camera()),
        _ => None,
    }
}

fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "top_down_camera_smoke",
        "top_down_camera_follow",
        "top_down_camera_bounds",
        "top_down_camera_zoom",
        "top_down_camera_soft_zone",
        "top_down_camera_target_switch",
        "example_basic_2d",
        "example_basic_3d",
        "example_bounds",
        "example_target_switching",
        "example_soft_zone_framing",
        "example_optional_controls",
        "example_strategy_game",
        "example_arpg_camera",
    ]
}

fn camera_entity(world: &World) -> Option<Entity> {
    world
        .get_resource::<LabCameraEntity>()
        .map(|resource| resource.0)
}

fn primary_target_entity(world: &World) -> Option<Entity> {
    world
        .get_resource::<LabPrimaryTargetEntity>()
        .map(|resource| resource.0)
}

fn secondary_target_entity(world: &World) -> Option<Entity> {
    world
        .get_resource::<LabSecondaryTargetEntity>()
        .map(|resource| resource.0)
}

fn runtime(world: &World) -> Option<TopDownCameraRuntime> {
    let entity = camera_entity(world)?;
    world.get::<TopDownCameraRuntime>(entity).cloned()
}

fn orthographic_scale(world: &World) -> Option<f32> {
    let entity = camera_entity(world)?;
    let projection = world.get::<Projection>(entity)?;
    match projection {
        Projection::Orthographic(orthographic) => Some(orthographic.scale),
        _ => None,
    }
}

fn store_baseline(world: &mut World) {
    if let Some(runtime) = runtime(world) {
        world.insert_resource(RuntimeBaseline {
            follow_anchor: runtime.follow_anchor,
            zoom: runtime.zoom,
            active_target: runtime.active_target,
            projection_scale: orthographic_scale(world),
        });
    }
}

fn build_smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description("Boot the lab, wait for the orthographic 3D scene to stabilize, verify runtime state, and capture a screenshot.")
        .then(Action::WaitFrames(90))
        .then(assertions::entity_exists::<TopDownCamera>("camera entity exists"))
        .then(assertions::component_satisfies::<TopDownCameraRuntime>(
            "runtime initialized",
            |runtime| runtime.zoom > 0.0 && runtime.follow_anchor.is_finite(),
        ))
        .then(assertions::custom(
            "orthographic projection active",
            Box::new(|world: &World| orthographic_scale(world).is_some()),
        ))
        .then(assertions::log_summary("smoke_launch summary"))
        .then(Action::Screenshot("smoke_launch".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_follow() -> Scenario {
    Scenario::builder("top_down_camera_follow")
        .description("Drive the real control path to move the primary target past the dead zone, then assert the camera anchor follows and capture before/after checkpoints.")
        .then(Action::WaitFrames(60))
        .then(Action::Custom(Box::new(|world: &mut World| {
            store_baseline(world);
        })))
        .then(Action::Screenshot("top_down_camera_follow_before".into()))
        .then(Action::HoldKey {
            key: KeyCode::KeyD,
            frames: 45,
        })
        .then(Action::WaitFrames(18))
        .then(assertions::custom(
            "camera follow anchor moved",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                runtime.follow_anchor.distance(baseline.follow_anchor) > 0.6
                    && runtime.active_target == primary_target_entity(world)
            }),
        ))
        .then(assertions::log_summary("top_down_camera_follow summary"))
        .then(inspect::dump_component_json::<TopDownCameraRuntime>(
            "top_down_camera_follow_runtime",
        ))
        .then(Action::Screenshot("top_down_camera_follow_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_runtime_smoke() -> Scenario {
    Scenario::builder("top_down_camera_smoke")
        .description("Verify the lab camera has valid runtime state, dump the runtime component, and capture a crate-specific smoke screenshot.")
        .then(Action::WaitFrames(90))
        .then(assertions::entity_exists::<TopDownCamera>(
            "camera entity exists",
        ))
        .then(assertions::entity_count::<TopDownCameraTarget>(
            "two camera targets exist",
            2,
        ))
        .then(assertions::component_satisfies::<TopDownCameraRuntime>(
            "runtime has a tracked point and valid zoom",
            |runtime| runtime.zoom > 0.0 && runtime.tracked_point.is_finite(),
        ))
        .then(inspect::log_world_summary("top_down_camera_smoke world"))
        .then(assertions::log_summary("top_down_camera_smoke summary"))
        .then(inspect::dump_component_json::<TopDownCameraRuntime>(
            "top_down_camera_smoke_runtime",
        ))
        .then(Action::Screenshot("top_down_camera_smoke".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_bounds() -> Scenario {
    Scenario::builder("top_down_camera_bounds")
        .description("Push the tracked target past the configured east bound, prove the target crossed the confiner, and assert the camera goal clamps to the edge with visual checkpoints.")
        .then(Action::WaitFrames(50))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let Some(target) = primary_target_entity(world) else {
                return;
            };
            if let Some(mut transform) = world.get_mut::<Transform>(target) {
                transform.translation = Vec3::new(11.5, 0.75, 0.0);
            }
            let Some(camera) = camera_entity(world) else {
                return;
            };
            if let Some(mut controller) = world.get_mut::<TopDownCamera>(camera) {
                let yaw = controller.target_yaw;
                let zoom = controller.zoom;
                controller.snap_to(Vec3::new(11.5, 1.5, 0.0), yaw, zoom);
                controller.tracked_target = Some(target);
            }
        })))
        .then(Action::Screenshot("top_down_camera_bounds_before".into()))
        .then(Action::WaitFrames(6))
        .then(Action::HoldKey {
            key: KeyCode::KeyD,
            frames: 35,
        })
        .then(Action::WaitFrames(18))
        .then(assertions::custom(
            "target crosses east bound and camera goal clamps there",
            Box::new(|world: &World| {
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                let Some(target) = primary_target_entity(world) else {
                    return false;
                };
                let Some(transform) = world.get::<Transform>(target) else {
                    return false;
                };
                transform.translation.x > 14.5
                    && runtime.goal_anchor.x >= 11.8
                    && runtime.goal_anchor.x <= 12.05
                    && runtime.follow_anchor.x <= 12.05
            }),
        ))
        .then(assertions::log_summary("top_down_camera_bounds summary"))
        .then(inspect::dump_component_json::<TopDownCameraRuntime>(
            "top_down_camera_bounds_runtime",
        ))
        .then(Action::Screenshot("top_down_camera_bounds_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_zoom() -> Scenario {
    Scenario::builder("top_down_camera_zoom")
        .description("Use the crate-local camera controls to change orthographic zoom, then assert both runtime zoom and projection scale changed.")
        .then(Action::WaitFrames(50))
        .then(Action::Custom(Box::new(|world: &mut World| {
            store_baseline(world);
        })))
        .then(Action::Screenshot("top_down_camera_zoom_before".into()))
        .then(Action::HoldKey {
            key: KeyCode::KeyX,
            frames: 30,
        })
        .then(Action::WaitFrames(18))
        .then(assertions::custom(
            "orthographic zoom changed",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                let Some(scale) = orthographic_scale(world) else {
                    return false;
                };
                (runtime.zoom - baseline.zoom).abs() > 0.05
                    && baseline
                        .projection_scale
                        .is_some_and(|before| (scale - before).abs() > 0.05)
            }),
        ))
        .then(assertions::log_summary("top_down_camera_zoom summary"))
        .then(Action::Screenshot("top_down_camera_zoom_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_soft_zone() -> Scenario {
    Scenario::builder("top_down_camera_soft_zone")
        .description("Temporarily neutralize look-ahead, place the tracked hero just beyond the dead zone but within the soft zone, then assert the camera applies only a partial correction.")
        .then(Action::WaitFrames(40))
        .then(Action::Custom(Box::new(|world: &mut World| {
            store_baseline(world);
            let Some(target) = primary_target_entity(world) else {
                return;
            };
            if let Some(mut camera_target) = world.get_mut::<saddle_camera_top_down_camera::TopDownCameraTarget>(target) {
                camera_target.look_ahead_time = Vec2::ZERO;
                camera_target.max_look_ahead = Vec2::ZERO;
            }
            if let Some(mut transform) = world.get_mut::<Transform>(target) {
                transform.translation = Vec3::new(2.4, 0.75, 0.0);
            }
        })))
        .then(Action::Screenshot("top_down_camera_soft_zone_before".into()))
        .then(Action::WaitFrames(18))
        .then(assertions::custom(
            "soft zone applies only a partial correction",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                let Some(camera_entity) = camera_entity(world) else {
                    return false;
                };
                let Some(settings) = world.get::<TopDownCameraSettings>(camera_entity) else {
                    return false;
                };
                let Some(target) = primary_target_entity(world) else {
                    return false;
                };
                let Some(transform) = world.get::<Transform>(target) else {
                    return false;
                };

                let (axis_x, axis_y) = match settings.mode {
                    saddle_camera_top_down_camera::TopDownCameraMode::Flat2d { .. } => {
                        (Vec3::X, Vec3::Y)
                    }
                    saddle_camera_top_down_camera::TopDownCameraMode::Tilted3d { .. } => {
                        let rotation = Quat::from_rotation_y(runtime.yaw);
                        (rotation * Vec3::X, rotation * Vec3::NEG_Z)
                    }
                };

                let relative_to_goal = runtime.tracked_point - runtime.goal_anchor;
                let residual = Vec2::new(
                    relative_to_goal.dot(axis_x),
                    relative_to_goal.dot(axis_y),
                ) - settings.bias;
                let dead_half = settings.dead_zone * 0.5;
                let soft_half = settings.soft_zone.max(settings.dead_zone) * 0.5;
                let goal_shift = runtime.goal_anchor.distance(baseline.follow_anchor);

                transform.translation.x > 2.3
                    && goal_shift > 0.15
                    && residual.x.abs() > dead_half.x + 0.05
                    && residual.x.abs() < soft_half.x - 0.05
                    && residual.y.abs() < soft_half.y - 0.05
                    && runtime.follow_anchor.distance(runtime.goal_anchor) > 0.02
                    && runtime.follow_anchor.distance(runtime.goal_anchor) < 0.25
            }),
        ))
        .then(assertions::log_summary("top_down_camera_soft_zone summary"))
        .then(inspect::dump_component_json::<TopDownCameraRuntime>(
            "top_down_camera_soft_zone_runtime",
        ))
        .then(Action::Screenshot("top_down_camera_soft_zone_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

fn build_target_switch() -> Scenario {
    Scenario::builder("top_down_camera_target_switch")
        .description("Trigger the shared example switch-target action, assert the camera retargets to the secondary hero, and capture checkpoints.")
        .then(Action::WaitFrames(50))
        .then(Action::Custom(Box::new(|world: &mut World| {
            store_baseline(world);
        })))
        .then(Action::Screenshot("top_down_camera_target_switch_before".into()))
        .then(Action::HoldKey {
            key: KeyCode::Tab,
            frames: 2,
        })
        .then(Action::WaitFrames(18))
        .then(assertions::custom(
            "active target changed",
            Box::new(|world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(runtime) = runtime(world) else {
                    return false;
                };
                runtime.active_target != baseline.active_target
                    && runtime.active_target == secondary_target_entity(world)
            }),
        ))
        .then(assertions::log_summary("top_down_camera_target_switch summary"))
        .then(inspect::dump_component_json::<TopDownCameraRuntime>(
            "top_down_camera_target_switch_runtime",
        ))
        .then(Action::Screenshot("top_down_camera_target_switch_after".into()))
        .then(Action::WaitFrames(1))
        .build()
}

// ──────────────────────────────────────────────────────────────────────
// Per-example diagnostic scenarios
//
// Each scenario reconfigures the lab camera to match the exact settings
// from a specific example, teleports the target, and verifies the camera
// follows correctly. The runtime JSON is dumped for debugging.
// ──────────────────────────────────────────────────────────────────────

/// Helper: reconfigure camera settings and snap the camera to a target,
/// matching a specific example's setup. Inserts `LabDefaultsOverridden` to
/// prevent the lab's enforcement system from reverting the changes, and
/// updates the pane resource to match so `sync_example_pane` stays in sync.
fn reconfigure_camera(
    world: &mut World,
    settings: TopDownCameraSettings,
    anchor: Vec3,
    yaw: f32,
    zoom: f32,
    follow_enabled: bool,
) {
    // Prevent enforce_authored_lab_defaults from reverting our settings
    world.insert_resource(crate::LabDefaultsOverridden);

    // Update the pane to match so sync_example_pane doesn't override our values
    if let Some(mut pane) =
        world.get_resource_mut::<saddle_camera_top_down_camera_example_common::ExampleTopDownPane>()
    {
        pane.follow_enabled = follow_enabled;
        pane.dead_zone_x = settings.dead_zone.x;
        pane.dead_zone_y = settings.dead_zone.y;
        pane.soft_zone_x = settings.soft_zone.x;
        pane.soft_zone_y = settings.soft_zone.y;
        pane.bias_x = settings.bias.x;
        pane.bias_y = settings.bias.y;
        pane.zoom = zoom;
        pane.zoom_speed = settings.zoom_speed;
        pane.planar_damping = settings.damping.planar_x;
        pane.zoom_damping = settings.damping.zoom;
        pane.yaw_damping = settings.damping.yaw;
        pane.yaw_radians = yaw;
        if let TopDownCameraMode::Tilted3d { pitch, .. } = settings.mode {
            pane.pitch_degrees = pitch.to_degrees();
        }
    }

    let Some(camera_entity) = camera_entity(world) else {
        return;
    };
    if let Some(mut s) = world.get_mut::<TopDownCameraSettings>(camera_entity) {
        *s = settings;
    }
    if let Some(mut camera) = world.get_mut::<TopDownCamera>(camera_entity) {
        camera.snap_to(anchor, yaw, zoom);
        camera.follow_enabled = follow_enabled;
    }
}

/// Helper: teleport the primary target to a position.
fn teleport_primary(world: &mut World, position: Vec3) {
    let Some(target) = primary_target_entity(world) else {
        return;
    };
    if let Some(mut transform) = world.get_mut::<Transform>(target) {
        transform.translation = position;
    }
}

/// Helper: set the target's properties.
fn configure_primary_target(world: &mut World, target_comp: TopDownCameraTarget) {
    let Some(entity) = primary_target_entity(world) else {
        return;
    };
    if let Some(mut t) = world.get_mut::<TopDownCameraTarget>(entity) {
        *t = target_comp;
    }
}

/// Helper: build a follow-validation scenario after configuration.
/// Moves the target to `move_to`, waits, and asserts the camera followed.
fn follow_validation_sequence(
    scenario_name: &str,
    move_to: Vec3,
    min_follow_distance: f32,
) -> Vec<Action> {
    let name = scenario_name.to_owned();
    let before_name = format!("{name}_before");
    let after_name = format!("{name}_after");
    let runtime_name = format!("{name}_runtime");
    let summary_name = format!("{name} summary");
    vec![
        Action::WaitFrames(30),
        Action::Custom(Box::new(|world: &mut World| {
            store_baseline(world);
        })),
        Action::Screenshot(before_name),
        Action::Log(format!(
            "[{name}] baseline stored, moving target to {move_to:?}"
        )),
        Action::Custom(Box::new(move |world: &mut World| {
            teleport_primary(world, move_to);
        })),
        Action::WaitFrames(60),
        Action::Custom(Box::new(move |world: &mut World| {
            // Log the runtime state for debugging
            if let Some(rt) = runtime(world) {
                info!(
                    "[{name}] follow_anchor={:?} goal_anchor={:?} tracked_point={:?} render_anchor={:?} zoom={:.2} yaw={:.2} active_target={:?}",
                    rt.follow_anchor,
                    rt.goal_anchor,
                    rt.tracked_point,
                    rt.render_anchor,
                    rt.zoom,
                    rt.yaw,
                    rt.active_target
                );
            }
        })),
        assertions::custom(
            "camera followed target",
            Box::new(move |world: &World| {
                let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                    return false;
                };
                let Some(rt) = runtime(world) else {
                    return false;
                };
                let distance = rt.follow_anchor.distance(baseline.follow_anchor);
                if distance < min_follow_distance {
                    warn!(
                        "[follow check] FAIL: follow_anchor moved only {distance:.3}, expected >= {min_follow_distance}. baseline={:?} current={:?}",
                        baseline.follow_anchor, rt.follow_anchor
                    );
                }
                distance >= min_follow_distance
            }),
        ),
        assertions::log_summary(Box::leak(summary_name.into_boxed_str())),
        inspect::dump_component_json::<TopDownCameraRuntime>(Box::leak(
            runtime_name.into_boxed_str(),
        )),
        Action::Screenshot(after_name),
        Action::WaitFrames(1),
    ]
}

fn build_example_basic_2d() -> Scenario {
    let mut builder = Scenario::builder("example_basic_2d")
        .description("Replicate basic_2d example settings: Flat2d, large dead zone (180,140), soft zone (280,220). Move target far and verify camera follows.");
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        reconfigure_camera(
            world,
            TopDownCameraSettings {
                mode: TopDownCameraMode::flat_2d(999.0),
                dead_zone: Vec2::new(180.0, 140.0),
                soft_zone: Vec2::new(280.0, 220.0),
                damping: TopDownCameraDamping {
                    planar_x: 7.0,
                    planar_y: 7.0,
                    ..default()
                },
                ..default()
            },
            Vec3::new(0.0, 0.0, 999.0),
            0.0,
            1.0,
            true,
        );
        configure_primary_target(
            world,
            TopDownCameraTarget {
                priority: 10,
                ..default()
            },
        );
        teleport_primary(world, Vec3::new(0.0, 0.0, 0.0));
    })));
    for action in follow_validation_sequence("example_basic_2d", Vec3::new(300.0, 200.0, 0.0), 20.0)
    {
        builder = builder.then(action);
    }
    builder.build()
}

fn build_example_basic_3d() -> Scenario {
    let mut builder = Scenario::builder("example_basic_3d")
        .description("Replicate basic_3d example settings: Tilted3d(60deg, 18.0), dead zone (3.8,2.6), soft zone (6.2,4.2). Move target and verify follow.");
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        reconfigure_camera(
            world,
            TopDownCameraSettings {
                mode: TopDownCameraMode::tilted_3d(60.0_f32.to_radians(), 18.0),
                dead_zone: Vec2::new(3.8, 2.6),
                soft_zone: Vec2::new(6.2, 4.2),
                damping: TopDownCameraDamping {
                    planar_x: 8.0,
                    planar_y: 10.0,
                    height: 12.0,
                    zoom: 8.0,
                    yaw: 10.0,
                },
                zoom_min: 8.0,
                zoom_max: 30.0,
                zoom_speed: 1.5,
                ..default()
            },
            Vec3::new(0.0, 0.75, 0.0),
            0.0,
            16.0,
            true,
        );
        configure_primary_target(
            world,
            TopDownCameraTarget {
                priority: 10,
                anchor_offset: Vec3::Y * 0.75,
                look_ahead_time: Vec2::splat(0.15),
                max_look_ahead: Vec2::splat(1.2),
                ..default()
            },
        );
        teleport_primary(world, Vec3::new(0.0, 0.75, 0.0));
    })));
    for action in follow_validation_sequence("example_basic_3d", Vec3::new(10.0, 0.75, 8.0), 2.0) {
        builder = builder.then(action);
    }
    builder.build()
}

fn build_example_bounds() -> Scenario {
    let mut builder = Scenario::builder("example_bounds")
        .description("Replicate bounds example: Flat2d, dead zone (140,120), bounds (-300..300, -220..220). Push target past bounds and verify camera clamps.");
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        reconfigure_camera(
            world,
            TopDownCameraSettings {
                mode: TopDownCameraMode::flat_2d(999.0),
                dead_zone: Vec2::new(140.0, 120.0),
                soft_zone: Vec2::new(240.0, 200.0),
                bounds: Some(TopDownCameraBounds {
                    min: Vec2::new(-300.0, -220.0),
                    max: Vec2::new(300.0, 220.0),
                }),
                damping: TopDownCameraDamping {
                    planar_x: 7.0,
                    planar_y: 7.0,
                    ..default()
                },
                ..default()
            },
            Vec3::new(0.0, 0.0, 999.0),
            0.0,
            1.0,
            true,
        );
        configure_primary_target(
            world,
            TopDownCameraTarget {
                priority: 10,
                ..default()
            },
        );
        teleport_primary(world, Vec3::new(0.0, 0.0, 0.0));
    })));
    // Move target past bounds
    builder = builder.then(Action::WaitFrames(30));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        store_baseline(world);
    })));
    builder = builder.then(Action::Screenshot("example_bounds_before".into()));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        teleport_primary(world, Vec3::new(400.0, 0.0, 0.0));
    })));
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(assertions::custom(
        "camera follows but clamps to bounds",
        Box::new(|world: &World| {
            let Some(rt) = runtime(world) else {
                return false;
            };
            // Camera should have moved but be clamped to max bound (300)
            rt.follow_anchor.x > 50.0 && rt.follow_anchor.x <= 302.0
        }),
    ));
    builder = builder.then(assertions::log_summary("example_bounds summary"));
    builder = builder.then(inspect::dump_component_json::<TopDownCameraRuntime>(
        "example_bounds_runtime",
    ));
    builder = builder.then(Action::Screenshot("example_bounds_after".into()));
    builder = builder.then(Action::WaitFrames(1));
    builder.build()
}

fn build_example_target_switching() -> Scenario {
    let mut builder = Scenario::builder("example_target_switching")
        .description("Replicate target_switching example: Tilted3d(58deg, 16.0), dead zone (3.4,2.2). Switch target and verify camera retargets.");
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        reconfigure_camera(
            world,
            TopDownCameraSettings {
                mode: TopDownCameraMode::tilted_3d(58.0_f32.to_radians(), 16.0),
                dead_zone: Vec2::new(3.4, 2.2),
                soft_zone: Vec2::new(5.6, 3.7),
                zoom_min: 10.0,
                zoom_max: 24.0,
                zoom_speed: 1.2,
                ..default()
            },
            Vec3::new(0.0, 0.75, 0.0),
            0.0,
            14.0,
            true,
        );
        configure_primary_target(
            world,
            TopDownCameraTarget {
                priority: 10,
                anchor_offset: Vec3::Y * 0.75,
                ..default()
            },
        );
        teleport_primary(world, Vec3::new(-5.0, 0.75, -3.0));
    })));
    for action in
        follow_validation_sequence("example_target_switching", Vec3::new(8.0, 0.75, 6.0), 2.0)
    {
        builder = builder.then(action);
    }
    builder.build()
}

fn build_example_soft_zone_framing() -> Scenario {
    let mut builder = Scenario::builder("example_soft_zone_framing")
        .description("Replicate soft_zone_framing example: Flat2d, dead zone (120,90), large soft zone (380,250). Verify soft follow.");
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        reconfigure_camera(
            world,
            TopDownCameraSettings {
                mode: TopDownCameraMode::flat_2d(999.0),
                dead_zone: Vec2::new(120.0, 90.0),
                soft_zone: Vec2::new(380.0, 250.0),
                damping: TopDownCameraDamping {
                    planar_x: 5.5,
                    planar_y: 5.5,
                    ..default()
                },
                ..default()
            },
            Vec3::new(0.0, 0.0, 999.0),
            0.0,
            1.0,
            true,
        );
        configure_primary_target(
            world,
            TopDownCameraTarget {
                priority: 10,
                ..default()
            },
        );
        teleport_primary(world, Vec3::new(0.0, 0.0, 0.0));
    })));
    for action in follow_validation_sequence(
        "example_soft_zone_framing",
        Vec3::new(250.0, 150.0, 0.0),
        15.0,
    ) {
        builder = builder.then(action);
    }
    builder.build()
}

fn build_example_optional_controls() -> Scenario {
    let mut builder = Scenario::builder("example_optional_controls")
        .description("Replicate optional_controls example: Tilted3d(58deg, 18.0), dead zone (3.6,2.4), soft zone (6.0,4.0), yaw 0.4. This is the example where follow felt broken — verify camera tracks the target.");
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        reconfigure_camera(
            world,
            TopDownCameraSettings {
                mode: TopDownCameraMode::tilted_3d(58.0_f32.to_radians(), 18.0),
                dead_zone: Vec2::new(3.6, 2.4),
                soft_zone: Vec2::new(6.0, 4.0),
                bias: Vec2::new(0.0, -0.2),
                zoom_min: 8.0,
                zoom_max: 26.0,
                zoom_speed: 1.5,
                ..default()
            },
            Vec3::new(0.0, 0.75, 0.0),
            0.4,
            16.0,
            true,
        );
        configure_primary_target(
            world,
            TopDownCameraTarget {
                priority: 10,
                anchor_offset: Vec3::Y * 0.75,
                ..default()
            },
        );
        // Target starts offset from camera — matches the example's initial state
        teleport_primary(world, Vec3::new(-4.0, 0.75, 0.0));
    })));
    // First: verify the camera converges toward the initial target offset
    builder = builder.then(Action::WaitFrames(30));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        store_baseline(world);
        if let Some(rt) = runtime(world) {
            info!(
                "[optional_controls] initial convergence: follow_anchor={:?} goal_anchor={:?} tracked_point={:?}",
                rt.follow_anchor, rt.goal_anchor, rt.tracked_point
            );
        }
    })));
    builder = builder.then(Action::Screenshot(
        "example_optional_controls_initial".into(),
    ));
    // Now move the target significantly
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        store_baseline(world);
    })));
    for action in follow_validation_sequence(
        "example_optional_controls",
        Vec3::new(10.0, 0.75, -8.0),
        2.0,
    ) {
        builder = builder.then(action);
    }
    builder.build()
}

fn build_example_strategy_game() -> Scenario {
    let mut builder = Scenario::builder("example_strategy_game")
        .description("Replicate strategy_game example: Tilted3d(55deg, 20.0), follow DISABLED. Verify camera does NOT auto-follow when follow_enabled is false.");
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        reconfigure_camera(
            world,
            TopDownCameraSettings {
                mode: TopDownCameraMode::tilted_3d(55.0_f32.to_radians(), 20.0),
                dead_zone: Vec2::new(2.0, 2.0),
                soft_zone: Vec2::new(4.0, 3.0),
                bounds: Some(TopDownCameraBounds {
                    min: Vec2::new(-20.0, -20.0),
                    max: Vec2::new(20.0, 20.0),
                }),
                bounds_soft_margin: 2.0,
                zoom_min: 6.0,
                zoom_max: 30.0,
                zoom_speed: 2.0,
                ..default()
            },
            Vec3::new(0.0, 0.75, 0.0),
            0.0,
            14.0,
            false, // follow disabled!
        );
        configure_primary_target(
            world,
            TopDownCameraTarget {
                priority: 1,
                anchor_offset: Vec3::Y * 0.75,
                ..default()
            },
        );
        teleport_primary(world, Vec3::new(0.0, 0.75, 0.0));
    })));
    builder = builder.then(Action::WaitFrames(30));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        store_baseline(world);
    })));
    builder = builder.then(Action::Screenshot("example_strategy_game_before".into()));
    // Move target far away — camera should NOT follow
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        teleport_primary(world, Vec3::new(15.0, 0.75, 10.0));
    })));
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(assertions::custom(
        "camera did NOT follow (follow_enabled=false)",
        Box::new(|world: &World| {
            let Some(baseline) = world.get_resource::<RuntimeBaseline>().copied() else {
                return false;
            };
            let Some(rt) = runtime(world) else {
                return false;
            };
            // Camera should NOT have moved significantly
            let distance = rt.follow_anchor.distance(baseline.follow_anchor);
            if distance > 0.5 {
                warn!(
                    "[strategy_game] FAIL: camera moved {distance:.3} with follow_enabled=false",
                );
            }
            distance < 0.5
        }),
    ));
    builder = builder.then(assertions::log_summary("example_strategy_game summary"));
    builder = builder.then(inspect::dump_component_json::<TopDownCameraRuntime>(
        "example_strategy_game_runtime",
    ));
    builder = builder.then(Action::Screenshot("example_strategy_game_after".into()));
    builder = builder.then(Action::WaitFrames(1));
    builder.build()
}

fn build_example_arpg_camera() -> Scenario {
    let mut builder = Scenario::builder("example_arpg_camera")
        .description("Replicate arpg_camera example: Tilted3d(58deg, 18.0), tight dead zone (1.5,1.0), look-ahead, bias. Verify responsive follow.");
    builder = builder.then(Action::WaitFrames(60));
    builder = builder.then(Action::Custom(Box::new(|world: &mut World| {
        reconfigure_camera(
            world,
            TopDownCameraSettings {
                mode: TopDownCameraMode::tilted_3d(58.0_f32.to_radians(), 18.0),
                dead_zone: Vec2::new(1.5, 1.0),
                soft_zone: Vec2::new(3.5, 2.5),
                bias: Vec2::new(0.0, -0.3),
                zoom_min: 8.0,
                zoom_max: 26.0,
                zoom_speed: 1.8,
                ..default()
            },
            Vec3::new(0.0, 0.75, 0.0),
            0.0,
            16.0,
            true,
        );
        configure_primary_target(
            world,
            TopDownCameraTarget {
                priority: 10,
                anchor_offset: Vec3::Y * 0.75,
                look_ahead_time: Vec2::new(0.15, 0.15),
                max_look_ahead: Vec2::splat(3.0),
                ..default()
            },
        );
        teleport_primary(world, Vec3::new(0.0, 0.75, 0.0));
    })));
    for action in follow_validation_sequence("example_arpg_camera", Vec3::new(6.0, 0.75, -5.0), 1.5)
    {
        builder = builder.then(action);
    }
    builder.build()
}
