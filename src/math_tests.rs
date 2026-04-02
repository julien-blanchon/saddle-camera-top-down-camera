use super::*;

#[test]
fn dead_zone_correction_is_zero_inside_zone() {
    assert_eq!(
        dead_zone_correction(Vec2::new(12.0, -8.0), Vec2::new(20.0, 20.0)),
        Vec2::ZERO
    );
}

#[test]
fn dead_zone_correction_returns_excess_distance() {
    assert_eq!(
        dead_zone_correction(Vec2::new(32.0, -28.0), Vec2::new(20.0, 10.0)),
        Vec2::new(12.0, -18.0)
    );
}

#[test]
fn solve_anchor_goal_clamps_planar_bounds() {
    let goal = solve_anchor_goal(
        Vec3::ZERO,
        Vec3::new(90.0, 20.0, 0.0),
        Vec2::ZERO,
        Vec2::splat(20.0),
        Some(TopDownCameraBounds {
            min: Vec2::new(-8.0, -6.0),
            max: Vec2::new(8.0, 6.0),
        }),
        TopDownCameraMode::flat_2d(10.0),
        0.0,
    );

    assert_eq!(goal, Vec3::new(8.0, 6.0, 0.0));
}

#[test]
fn clamp_zoom_handles_reversed_ranges() {
    assert_eq!(clamp_zoom(20.0, 10.0, 5.0), 10.0);
    assert_eq!(clamp_zoom(2.0, 10.0, 5.0), 5.0);
}

#[test]
fn smooth_scalar_converges_toward_target() {
    let mut value = 0.0;
    for _ in 0..60 {
        value = smooth_scalar(value, 10.0, 8.0, 1.0 / 60.0);
    }

    assert!(value > 9.9);
}

#[test]
fn tilted_3d_translation_uses_yaw_pitch_and_distance() {
    let translation = tilted_3d_camera_translation(Vec3::ZERO, 0.0, 60.0_f32.to_radians(), 10.0);
    assert!((translation.y - 8.6602545).abs() < 0.001);
    assert!((translation.z - 5.0).abs() < 0.001);
}
