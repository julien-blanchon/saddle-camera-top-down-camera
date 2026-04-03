use bevy::prelude::*;

use crate::{
    TopDownCameraDebug, TopDownCameraMode, TopDownCameraRuntime, TopDownCameraSettings,
    math::{PlanarFrame, planar_frame},
};

pub(crate) fn draw_debug_gizmos(
    mut gizmos: Gizmos,
    cameras: Query<(
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &TopDownCameraDebug,
    )>,
) {
    for (settings, runtime, debug) in &cameras {
        let frame = planar_frame(settings.mode, runtime.yaw);

        if debug.draw_dead_zone {
            if settings.soft_zone.cmpgt(settings.dead_zone).any() {
                draw_rect(
                    &mut gizmos,
                    runtime.follow_anchor + frame.planar_offset(settings.bias),
                    frame,
                    settings.soft_zone.max(settings.dead_zone),
                    match settings.mode {
                        TopDownCameraMode::Flat2d { .. } => Color::srgba(0.35, 0.72, 0.98, 0.55),
                        TopDownCameraMode::Tilted3d { .. } => {
                            Color::srgba(0.28, 0.66, 0.96, 0.55)
                        }
                    },
                );
            }
            draw_rect(
                &mut gizmos,
                runtime.follow_anchor + frame.planar_offset(settings.bias),
                frame,
                settings.dead_zone,
                match settings.mode {
                    TopDownCameraMode::Flat2d { .. } => Color::srgb(0.22, 0.82, 0.96),
                    TopDownCameraMode::Tilted3d { .. } => Color::srgb(0.20, 0.74, 0.96),
                },
            );
        }

        if debug.draw_bounds {
            if let Some(bounds) = settings.bounds {
                let size = bounds.max - bounds.min;
                let center = frame.planar_offset((bounds.min + bounds.max) * 0.5);
                draw_rect(
                    &mut gizmos,
                    match settings.mode {
                        TopDownCameraMode::Flat2d { .. } => {
                            Vec3::new(center.x, center.y, runtime.follow_anchor.z)
                        }
                        TopDownCameraMode::Tilted3d { .. } => {
                            Vec3::new(center.x, runtime.follow_anchor.y, center.z)
                        }
                    },
                    frame,
                    size,
                    Color::srgb(0.94, 0.66, 0.28),
                );
            }
        }

        if debug.draw_targets {
            gizmos.line(
                runtime.follow_anchor,
                runtime.tracked_point,
                Color::srgb(0.98, 0.34, 0.40),
            );
            gizmos.sphere(runtime.follow_anchor, 0.15, Color::srgb(0.18, 0.90, 0.42));
            gizmos.sphere(runtime.goal_anchor, 0.12, Color::srgb(0.98, 0.74, 0.16));
            gizmos.sphere(runtime.tracked_point, 0.10, Color::srgb(0.96, 0.24, 0.52));
        }
    }
}

fn draw_rect(gizmos: &mut Gizmos, center: Vec3, frame: PlanarFrame, size: Vec2, color: Color) {
    let half = size * 0.5;
    let corners = [
        center + frame.planar_offset(Vec2::new(-half.x, -half.y)),
        center + frame.planar_offset(Vec2::new(half.x, -half.y)),
        center + frame.planar_offset(Vec2::new(half.x, half.y)),
        center + frame.planar_offset(Vec2::new(-half.x, half.y)),
    ];
    gizmos.line(corners[0], corners[1], color);
    gizmos.line(corners[1], corners[2], color);
    gizmos.line(corners[2], corners[3], color);
    gizmos.line(corners[3], corners[0], color);
}
