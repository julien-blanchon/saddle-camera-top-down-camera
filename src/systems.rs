use bevy::{camera::Projection, math::StableInterpolate, prelude::*};

use crate::{
    TopDownCamera, TopDownCameraMode, TopDownCameraRuntime, TopDownCameraSettings,
    TopDownCameraTarget,
    components::TopDownCameraTargetState,
    math::{
        clamp_zoom, planar_frame, smooth_scalar, smooth_vec2_axes, solve_anchor_goal,
        tilted_3d_camera_translation,
    },
};

const LOOK_AHEAD_VELOCITY_DECAY: f32 = 14.0;

pub(crate) fn initialize_added_cameras(
    mut commands: Commands,
    cameras: Query<(Entity, &TopDownCamera), Added<TopDownCamera>>,
) {
    for (entity, camera) in &cameras {
        commands
            .entity(entity)
            .insert(TopDownCameraRuntime::from_camera(camera));
    }
}

pub(crate) fn initialize_added_targets(
    mut commands: Commands,
    targets: Query<(Entity, &Transform, &TopDownCameraTarget), Added<TopDownCameraTarget>>,
) {
    for (entity, transform, target) in &targets {
        commands.entity(entity).insert(TopDownCameraTargetState {
            previous_anchor: transform.translation + target.anchor_offset,
            velocity: Vec3::ZERO,
        });
    }
}

pub(crate) fn capture_target_motion(
    time: Res<Time>,
    mut targets: Query<(
        &Transform,
        &TopDownCameraTarget,
        &mut TopDownCameraTargetState,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (transform, target, mut state) in &mut targets {
        let anchor = transform.translation + target.anchor_offset;
        let raw_velocity = (anchor - state.previous_anchor) / dt;
        // Smooth the measured velocity so optional look-ahead remains usable even when
        // the followed target is updated in a different schedule such as FixedUpdate.
        state
            .velocity
            .smooth_nudge(&raw_velocity, LOOK_AHEAD_VELOCITY_DECAY, dt);
        state.previous_anchor = anchor;
    }
}

pub(crate) fn resolve_follow_targets(
    mut cameras: Query<(
        &mut TopDownCamera,
        &TopDownCameraSettings,
        &mut TopDownCameraRuntime,
    )>,
    explicit_targets: Query<(
        &Transform,
        Option<&TopDownCameraTarget>,
        Option<&TopDownCameraTargetState>,
    )>,
    auto_targets: Query<(
        Entity,
        &Transform,
        &TopDownCameraTarget,
        Option<&TopDownCameraTargetState>,
    )>,
) {
    for (mut camera, settings, mut runtime) in &mut cameras {
        let yaw = runtime.yaw;
        let resolved = resolve_target_candidate(
            camera.tracked_target,
            settings,
            yaw,
            &explicit_targets,
            &auto_targets,
        );

        runtime.active_target = resolved.map(|(entity, _)| entity);
        runtime.tracked_point = resolved
            .map(|(_, point)| point)
            .unwrap_or(camera.target_anchor);

        if !camera.follow_enabled {
            runtime.goal_anchor = camera.target_anchor;
            continue;
        }

        if let Some((_, tracked_point)) = resolved {
            let goal_anchor = solve_anchor_goal(
                runtime.follow_anchor,
                tracked_point,
                settings.bias,
                settings.dead_zone,
                settings.soft_zone,
                settings.bounds,
                settings.bounds_soft_margin,
                settings.mode,
                yaw,
            );
            camera.target_anchor = goal_anchor;
            runtime.goal_anchor = goal_anchor;
        } else {
            runtime.goal_anchor = camera.target_anchor;
        }
    }
}

fn resolve_target_candidate(
    explicit: Option<Entity>,
    settings: &TopDownCameraSettings,
    yaw: f32,
    explicit_targets: &Query<(
        &Transform,
        Option<&TopDownCameraTarget>,
        Option<&TopDownCameraTargetState>,
    )>,
    auto_targets: &Query<(
        Entity,
        &Transform,
        &TopDownCameraTarget,
        Option<&TopDownCameraTargetState>,
    )>,
) -> Option<(Entity, Vec3)> {
    if let Some(entity) = explicit {
        let Ok((transform, target, state)) = explicit_targets.get(entity) else {
            return None;
        };
        return Some((
            entity,
            tracked_point(
                transform,
                target.copied(),
                state.copied(),
                settings.mode,
                yaw,
            ),
        ));
    }

    let mut best: Option<(Entity, i32, Vec3)> = None;
    for (entity, transform, target, state) in auto_targets.iter() {
        if !target.enabled {
            continue;
        }

        let replace = best
            .map(|(best_entity, best_priority, _)| {
                target.priority > best_priority
                    || (target.priority == best_priority && entity.index() < best_entity.index())
            })
            .unwrap_or(true);

        if replace {
            best = Some((
                entity,
                target.priority,
                tracked_point(transform, Some(*target), state.copied(), settings.mode, yaw),
            ));
        }
    }

    best.map(|(entity, _, point)| (entity, point))
}

fn tracked_point(
    transform: &Transform,
    target: Option<TopDownCameraTarget>,
    state: Option<TopDownCameraTargetState>,
    mode: TopDownCameraMode,
    yaw: f32,
) -> Vec3 {
    let target = target.unwrap_or_default();
    let anchor = transform.translation + target.anchor_offset;
    let Some(state) = state else {
        return anchor;
    };

    let frame = planar_frame(mode, yaw);
    let velocity = frame.project_vector(state.velocity);
    let look_ahead = Vec2::new(
        (velocity.x * target.look_ahead_time.x)
            .clamp(-target.max_look_ahead.x, target.max_look_ahead.x),
        (velocity.y * target.look_ahead_time.y)
            .clamp(-target.max_look_ahead.y, target.max_look_ahead.y),
    );
    anchor + frame.planar_offset(look_ahead)
}

pub(crate) fn clamp_programmatic_goal(
    mut cameras: Query<(
        &mut TopDownCamera,
        &TopDownCameraSettings,
        &mut TopDownCameraRuntime,
    )>,
) {
    for (mut camera, settings, mut runtime) in &mut cameras {
        camera.zoom = clamp_zoom(camera.zoom, settings.zoom_min, settings.zoom_max);

        if !camera.follow_enabled {
            camera.target_anchor = solve_anchor_goal(
                camera.target_anchor,
                camera.target_anchor,
                Vec2::ZERO,
                Vec2::ZERO,
                Vec2::ZERO,
                settings.bounds,
                settings.bounds_soft_margin,
                settings.mode,
                runtime.yaw,
            );
        }

        runtime.goal_anchor = camera.target_anchor;
    }
}

pub(crate) fn advance_runtime(
    time: Res<Time>,
    mut cameras: Query<(
        &mut TopDownCamera,
        &TopDownCameraSettings,
        &mut TopDownCameraRuntime,
    )>,
) {
    let dt = time.delta_secs();

    for (mut camera, settings, mut runtime) in &mut cameras {
        if camera.snap {
            runtime.follow_anchor = camera.target_anchor;
            runtime.goal_anchor = camera.target_anchor;
            runtime.yaw = camera.target_yaw;
            runtime.zoom = camera.zoom;
            camera.snap = false;
            continue;
        }

        let frame = planar_frame(settings.mode, runtime.yaw);
        let current_planar = frame.project_point(Vec3::ZERO, runtime.follow_anchor);
        let goal_planar = frame.project_point(Vec3::ZERO, camera.target_anchor);
        let next_planar = smooth_vec2_axes(
            current_planar,
            goal_planar,
            settings.damping.planar_x,
            settings.damping.planar_y,
            dt,
        );
        let next_offset = frame.planar_offset(next_planar);
        runtime.follow_anchor = match settings.mode {
            TopDownCameraMode::Flat2d { .. } => {
                Vec3::new(next_offset.x, next_offset.y, runtime.follow_anchor.z)
            }
            TopDownCameraMode::Tilted3d { .. } => Vec3::new(
                next_offset.x,
                smooth_scalar(
                    runtime.follow_anchor.y,
                    camera.target_anchor.y,
                    settings.damping.height,
                    dt,
                ),
                next_offset.z,
            ),
        };

        runtime.goal_anchor = camera.target_anchor;
        runtime.yaw = smooth_scalar(runtime.yaw, camera.target_yaw, settings.damping.yaw, dt);
        runtime.zoom = smooth_scalar(runtime.zoom, camera.zoom, settings.damping.zoom, dt);
    }
}

pub(crate) fn sync_transform(
    mut cameras: Query<(
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        Option<&Projection>,
        &mut Transform,
    )>,
) {
    for (settings, runtime, projection, mut transform) in &mut cameras {
        match settings.mode {
            TopDownCameraMode::Flat2d { depth } => {
                transform.translation =
                    Vec3::new(runtime.render_anchor.x, runtime.render_anchor.y, depth);
                transform.rotation = Quat::IDENTITY;
            }
            TopDownCameraMode::Tilted3d {
                pitch,
                orthographic_distance,
            } => {
                let distance = match projection {
                    Some(Projection::Orthographic(_)) => orthographic_distance,
                    _ => runtime.render_zoom,
                };
                transform.translation = tilted_3d_camera_translation(
                    runtime.render_anchor,
                    runtime.render_yaw,
                    pitch,
                    distance,
                );
                transform.look_at(runtime.render_anchor, Vec3::Y);
            }
        }
    }
}

pub(crate) fn sync_projection(mut cameras: Query<(&TopDownCameraRuntime, &mut Projection)>) {
    for (runtime, mut projection) in &mut cameras {
        if let Projection::Orthographic(orthographic) = &mut *projection {
            orthographic.scale = runtime.render_zoom.max(0.001);
        }
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod tests;
