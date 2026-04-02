use bevy::{camera::Projection, prelude::*};
use bevy_enhanced_input::prelude::EnhancedInputSystems;
use saddle_camera_top_down_camera::{TopDownCamera, TopDownCameraRuntime};
use saddle_bevy_e2e::{
    E2EPlugin, E2ESet,
    action::Action,
    actions::{assertions, inspect},
    init_scenario,
    scenario::Scenario,
};

use crate::{LabCameraEntity, LabPrimaryTargetEntity, LabSecondaryTargetEntity};

pub struct TopDownCameraLabE2EPlugin;

impl Plugin for TopDownCameraLabE2EPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(E2EPlugin);
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
        "top_down_camera_target_switch" => Some(build_target_switch()),
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
        "top_down_camera_target_switch",
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
        .then(assertions::component_satisfies::<TopDownCameraRuntime>(
            "runtime has a tracked point and valid zoom",
            |runtime| runtime.zoom > 0.0 && runtime.tracked_point.is_finite(),
        ))
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
