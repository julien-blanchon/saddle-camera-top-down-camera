use bevy::{math::StableInterpolate, prelude::*};

use crate::{TopDownCameraBounds, TopDownCameraMode};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PlanarFrame {
    pub axis_x: Vec3,
    pub axis_y: Vec3,
}

impl PlanarFrame {
    pub(crate) fn project_point(self, origin: Vec3, point: Vec3) -> Vec2 {
        let delta = point - origin;
        Vec2::new(delta.dot(self.axis_x), delta.dot(self.axis_y))
    }

    pub(crate) fn project_vector(self, vector: Vec3) -> Vec2 {
        Vec2::new(vector.dot(self.axis_x), vector.dot(self.axis_y))
    }

    pub(crate) fn planar_offset(self, offset: Vec2) -> Vec3 {
        self.axis_x * offset.x + self.axis_y * offset.y
    }
}

pub(crate) fn planar_frame(mode: TopDownCameraMode, yaw: f32) -> PlanarFrame {
    match mode {
        TopDownCameraMode::Flat2d { .. } => PlanarFrame {
            axis_x: Vec3::X,
            axis_y: Vec3::Y,
        },
        TopDownCameraMode::Tilted3d { .. } => {
            let rotation = Quat::from_rotation_y(yaw);
            PlanarFrame {
                axis_x: rotation * Vec3::X,
                axis_y: rotation * Vec3::NEG_Z,
            }
        }
    }
}

pub(crate) fn dead_zone_correction(relative: Vec2, dead_zone_half: Vec2) -> Vec2 {
    Vec2::new(
        if relative.x > dead_zone_half.x {
            relative.x - dead_zone_half.x
        } else if relative.x < -dead_zone_half.x {
            relative.x + dead_zone_half.x
        } else {
            0.0
        },
        if relative.y > dead_zone_half.y {
            relative.y - dead_zone_half.y
        } else if relative.y < -dead_zone_half.y {
            relative.y + dead_zone_half.y
        } else {
            0.0
        },
    )
}

pub(crate) fn soft_zone_correction(
    relative: Vec2,
    dead_zone_half: Vec2,
    soft_zone_half: Vec2,
) -> Vec2 {
    Vec2::new(
        soft_zone_correction_axis(relative.x, dead_zone_half.x, soft_zone_half.x),
        soft_zone_correction_axis(relative.y, dead_zone_half.y, soft_zone_half.y),
    )
}

fn soft_zone_correction_axis(value: f32, dead_half: f32, soft_half: f32) -> f32 {
    let sign = value.signum();
    let distance = value.abs();
    let dead_half = dead_half.max(0.0);
    let soft_half = soft_half.max(dead_half);

    if distance <= dead_half {
        return 0.0;
    }

    let full_correction = distance - dead_half;
    if soft_half <= dead_half + f32::EPSILON {
        return sign * full_correction;
    }

    let blend = ((distance - dead_half) / (soft_half - dead_half)).clamp(0.0, 1.0);
    sign * full_correction * blend
}

pub(crate) fn clamp_zoom(value: f32, min: f32, max: f32) -> f32 {
    value.clamp(min.min(max), max.max(min))
}

pub(crate) fn clamp_to_bounds(
    value: Vec2,
    bounds: Option<TopDownCameraBounds>,
    soft_margin: f32,
) -> Vec2 {
    let Some(bounds) = bounds else {
        return value;
    };

    if soft_margin <= 0.0 {
        return Vec2::new(
            value.x.clamp(bounds.min.x, bounds.max.x),
            value.y.clamp(bounds.min.y, bounds.max.y),
        );
    }

    Vec2::new(
        soft_clamp_axis(value.x, bounds.min.x, bounds.max.x, soft_margin),
        soft_clamp_axis(value.y, bounds.min.y, bounds.max.y, soft_margin),
    )
}

fn soft_clamp_axis(value: f32, min: f32, max: f32, margin: f32) -> f32 {
    if value < min {
        let overshoot = min - value;
        let pull = margin * (1.0 - (-overshoot / margin).exp());
        min - (margin - pull)
    } else if value > max {
        let overshoot = value - max;
        let pull = margin * (1.0 - (-overshoot / margin).exp());
        max + (margin - pull)
    } else {
        value
    }
}

pub(crate) fn solve_anchor_goal(
    current_anchor: Vec3,
    tracked_point: Vec3,
    bias: Vec2,
    dead_zone_size: Vec2,
    soft_zone_size: Vec2,
    bounds: Option<TopDownCameraBounds>,
    bounds_soft_margin: f32,
    mode: TopDownCameraMode,
    yaw: f32,
) -> Vec3 {
    let frame = planar_frame(mode, yaw);
    let current_planar = frame.project_point(Vec3::ZERO, current_anchor);
    let tracked_planar = frame.project_point(Vec3::ZERO, tracked_point);
    let relative = tracked_planar - current_planar - bias;
    let correction = if soft_zone_size.cmple(dead_zone_size).all() {
        dead_zone_correction(relative, dead_zone_size * 0.5)
    } else {
        soft_zone_correction(relative, dead_zone_size * 0.5, soft_zone_size * 0.5)
    };
    let unclamped_goal = current_anchor + frame.planar_offset(correction);

    match mode {
        TopDownCameraMode::Flat2d { .. } => {
            let clamped = clamp_to_bounds(unclamped_goal.xy(), bounds, bounds_soft_margin);
            Vec3::new(clamped.x, clamped.y, current_anchor.z)
        }
        TopDownCameraMode::Tilted3d { .. } => {
            let clamped = clamp_to_bounds(unclamped_goal.xz(), bounds, bounds_soft_margin);
            Vec3::new(clamped.x, tracked_point.y, clamped.y)
        }
    }
}

pub(crate) fn smooth_scalar(current: f32, target: f32, decay_rate: f32, dt: f32) -> f32 {
    let mut value = current;
    value.smooth_nudge(&target, decay_rate.max(0.0), dt.max(0.0));
    value
}

pub(crate) fn smooth_vec2_axes(
    current: Vec2,
    target: Vec2,
    decay_x: f32,
    decay_y: f32,
    dt: f32,
) -> Vec2 {
    Vec2::new(
        smooth_scalar(current.x, target.x, decay_x, dt),
        smooth_scalar(current.y, target.y, decay_y, dt),
    )
}

pub(crate) fn tilted_3d_camera_translation(
    anchor: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
) -> Vec3 {
    let horizontal = distance * pitch.cos();
    let vertical = distance * pitch.sin();
    let back = Quat::from_rotation_y(yaw) * Vec3::Z;
    anchor + back * horizontal + Vec3::Y * vertical
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod tests;
