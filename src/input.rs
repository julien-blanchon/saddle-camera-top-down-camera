use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
    window::PrimaryWindow,
};

use crate::{
    TopDownCamera, TopDownCameraMode, TopDownCameraRuntime, TopDownCameraSettings,
    TopDownCameraSystems, math::planar_frame,
};

/// Configuration component for built-in top-down camera input handling.
///
/// Attach this to any entity that also has [`TopDownCamera`] and
/// [`TopDownCameraSettings`] to enable mouse, keyboard, and touch input.
///
/// Every feature can be individually toggled and configured.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct TopDownCameraInput {
    /// Enable keyboard panning (WASD / arrow keys by default).
    pub keyboard_pan_enabled: bool,
    /// Keyboard pan speed in world units per second.
    pub keyboard_pan_speed: f32,

    /// Enable mouse drag panning.
    pub mouse_drag_enabled: bool,
    /// Which mouse button activates drag panning.
    pub mouse_drag_button: MouseButton,

    /// Enable scroll wheel zoom.
    pub scroll_zoom_enabled: bool,
    /// Zoom amount per scroll line.
    pub scroll_zoom_sensitivity: f32,

    /// Enable zoom toward the cursor position instead of screen center.
    pub zoom_to_cursor: bool,

    /// Enable edge scrolling (camera moves when cursor is near screen edges).
    pub edge_scroll_enabled: bool,
    /// Width of the edge scroll zone in pixels from each screen edge.
    pub edge_scroll_margin: f32,
    /// Edge scroll speed in world units per second at the screen edge.
    pub edge_scroll_speed: f32,

    /// Enable Q/E keyboard rotation (only for `Tilted3d` mode).
    pub keyboard_rotate_enabled: bool,
    /// Rotation speed in radians per second.
    pub keyboard_rotate_speed: f32,

    /// Enable keyboard zoom (+/- keys).
    pub keyboard_zoom_enabled: bool,
    /// Zoom speed for keyboard zoom in units per second.
    pub keyboard_zoom_speed: f32,
}

impl Default for TopDownCameraInput {
    fn default() -> Self {
        Self {
            keyboard_pan_enabled: true,
            keyboard_pan_speed: 10.0,
            mouse_drag_enabled: true,
            mouse_drag_button: MouseButton::Middle,
            scroll_zoom_enabled: true,
            scroll_zoom_sensitivity: 0.15,
            zoom_to_cursor: true,
            edge_scroll_enabled: false,
            edge_scroll_margin: 30.0,
            edge_scroll_speed: 8.0,
            keyboard_rotate_enabled: true,
            keyboard_rotate_speed: 1.8,
            keyboard_zoom_enabled: true,
            keyboard_zoom_speed: 2.0,
        }
    }
}

impl TopDownCameraInput {
    /// Preset for strategy/RTS games: edge scroll, zoom-to-cursor, keyboard pan.
    pub fn strategy() -> Self {
        Self {
            edge_scroll_enabled: true,
            edge_scroll_speed: 12.0,
            keyboard_pan_speed: 14.0,
            scroll_zoom_sensitivity: 0.2,
            ..Self::default()
        }
    }

    /// Preset for ARPG games: no edge scroll, no keyboard pan, just follow target.
    pub fn arpg() -> Self {
        Self {
            keyboard_pan_enabled: false,
            mouse_drag_enabled: false,
            edge_scroll_enabled: false,
            keyboard_zoom_speed: 3.0,
            ..Self::default()
        }
    }
}

/// Optional plugin that adds built-in input handling for top-down cameras.
///
/// Attach a [`TopDownCameraInput`] component to any camera entity that also has
/// [`TopDownCamera`] and [`TopDownCameraSettings`] to activate input processing.
///
/// This plugin does NOT require `bevy_enhanced_input` and uses standard Bevy
/// input resources directly.
pub struct TopDownCameraInputPlugin;

impl Plugin for TopDownCameraInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TopDownCameraInput>()
            .add_systems(
                Update,
                (keyboard_pan_system, mouse_drag_pan_system)
                    .before(TopDownCameraSystems::ResolveTarget),
            )
            .add_systems(
                Update,
                (scroll_zoom_system, edge_scroll_system)
                    .before(TopDownCameraSystems::ResolveTarget),
            )
            .add_systems(
                Update,
                (keyboard_rotate_system, keyboard_zoom_system)
                    .before(TopDownCameraSystems::ResolveTarget),
            );
    }
}

fn keyboard_pan_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &mut TopDownCamera,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (input, settings, runtime, mut camera) in &mut cameras {
        if !input.keyboard_pan_enabled {
            continue;
        }

        let mut direction = Vec2::ZERO;
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }

        if direction == Vec2::ZERO {
            continue;
        }

        direction = direction.normalize();

        // Scale pan speed by zoom so it feels consistent at different zoom levels.
        let zoom_scale = match settings.mode {
            TopDownCameraMode::Flat2d { .. } => runtime.zoom.max(0.1),
            TopDownCameraMode::Tilted3d { .. } => 1.0,
        };

        let frame = planar_frame(settings.mode, runtime.yaw);
        let offset = frame.planar_offset(direction * input.keyboard_pan_speed * zoom_scale * dt);
        camera.target_anchor += offset;
    }
}

fn mouse_drag_pan_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    accumulated_motion: Res<AccumulatedMouseMotion>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &Camera,
        &mut TopDownCamera,
    )>,
) {
    let accumulated = accumulated_motion.delta;
    if accumulated == Vec2::ZERO {
        return;
    }

    for (input, settings, runtime, camera_component, mut camera) in &mut cameras {
        if !input.mouse_drag_enabled {
            continue;
        }
        if !mouse_buttons.pressed(input.mouse_drag_button) {
            continue;
        }

        let frame = planar_frame(settings.mode, runtime.yaw);

        // Convert pixel motion to world units.
        let world_delta = match settings.mode {
            TopDownCameraMode::Flat2d { .. } => {
                let Some(viewport_size) = camera_component.logical_viewport_size() else {
                    continue;
                };
                let world_units_per_pixel = runtime.zoom * 2.0 / viewport_size.y.max(1.0);
                Vec2::new(
                    -accumulated.x * world_units_per_pixel,
                    accumulated.y * world_units_per_pixel,
                )
            }
            TopDownCameraMode::Tilted3d { .. } => {
                let Some(viewport_size) = camera_component.logical_viewport_size() else {
                    continue;
                };
                let distance = runtime.zoom.max(1.0);
                let fov_scale = distance / viewport_size.y.max(1.0);
                Vec2::new(-accumulated.x * fov_scale, accumulated.y * fov_scale)
            }
        };

        let offset = frame.planar_offset(world_delta);
        camera.target_anchor += offset;
    }
}

fn scroll_zoom_system(
    accumulated_scroll: Res<AccumulatedMouseScroll>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &Camera,
        &mut TopDownCamera,
    )>,
) {
    let scroll_delta = match accumulated_scroll.unit {
        MouseScrollUnit::Line => accumulated_scroll.delta.y,
        MouseScrollUnit::Pixel => accumulated_scroll.delta.y / 120.0,
    };

    if scroll_delta.abs() < f32::EPSILON {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    for (input, settings, runtime, camera_component, mut camera) in &mut cameras {
        if !input.scroll_zoom_enabled {
            continue;
        }

        let old_zoom = camera.zoom;
        camera.zoom -= scroll_delta * input.scroll_zoom_sensitivity * camera.zoom;
        camera.zoom = camera.zoom.clamp(settings.zoom_min, settings.zoom_max);

        // Zoom-to-cursor: adjust anchor so cursor world position stays fixed.
        if input.zoom_to_cursor && (camera.zoom - old_zoom).abs() > f32::EPSILON {
            let Some(cursor_pos) = window.cursor_position() else {
                continue;
            };

            match settings.mode {
                TopDownCameraMode::Flat2d { .. } => {
                    let Some(viewport_size) = camera_component.logical_viewport_size() else {
                        continue;
                    };

                    // Normalized cursor position from center (-1..1)
                    let normalized = Vec2::new(
                        (cursor_pos.x / viewport_size.x - 0.5) * 2.0,
                        -(cursor_pos.y / viewport_size.y - 0.5) * 2.0,
                    );

                    let aspect = viewport_size.x / viewport_size.y.max(1.0);

                    // World position under cursor before and after zoom
                    let world_before = Vec2::new(
                        runtime.follow_anchor.x + normalized.x * old_zoom * aspect,
                        runtime.follow_anchor.y + normalized.y * old_zoom,
                    );
                    let world_after = Vec2::new(
                        runtime.follow_anchor.x + normalized.x * camera.zoom * aspect,
                        runtime.follow_anchor.y + normalized.y * camera.zoom,
                    );

                    let correction = world_before - world_after;
                    camera.target_anchor.x += correction.x;
                    camera.target_anchor.y += correction.y;
                }
                TopDownCameraMode::Tilted3d { .. } => {
                    let Some(viewport_size) = camera_component.logical_viewport_size() else {
                        continue;
                    };

                    let normalized = Vec2::new(
                        (cursor_pos.x / viewport_size.x - 0.5) * 2.0,
                        -(cursor_pos.y / viewport_size.y - 0.5) * 2.0,
                    );

                    let frame = planar_frame(settings.mode, runtime.yaw);
                    let scale_before = old_zoom * 0.5;
                    let scale_after = camera.zoom * 0.5;
                    let offset_before = frame.planar_offset(normalized * scale_before);
                    let offset_after = frame.planar_offset(normalized * scale_after);
                    let correction = offset_before - offset_after;
                    camera.target_anchor += correction;
                }
            }
        }
    }
}

fn edge_scroll_system(
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &mut TopDownCamera,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let width = window.width();
    let height = window.height();

    for (input, settings, runtime, mut camera) in &mut cameras {
        if !input.edge_scroll_enabled {
            continue;
        }

        let margin = input.edge_scroll_margin;
        if margin <= 0.0 {
            continue;
        }

        let mut direction = Vec2::ZERO;

        // Right edge
        if cursor_pos.x > width - margin {
            direction.x += ((cursor_pos.x - (width - margin)) / margin).clamp(0.0, 1.0);
        }
        // Left edge
        if cursor_pos.x < margin {
            direction.x -= ((margin - cursor_pos.x) / margin).clamp(0.0, 1.0);
        }
        // Top edge (screen Y=0 is top)
        if cursor_pos.y < margin {
            direction.y += ((margin - cursor_pos.y) / margin).clamp(0.0, 1.0);
        }
        // Bottom edge
        if cursor_pos.y > height - margin {
            direction.y -= ((cursor_pos.y - (height - margin)) / margin).clamp(0.0, 1.0);
        }

        if direction == Vec2::ZERO {
            continue;
        }

        let zoom_scale = match settings.mode {
            TopDownCameraMode::Flat2d { .. } => runtime.zoom.max(0.1),
            TopDownCameraMode::Tilted3d { .. } => 1.0,
        };

        let frame = planar_frame(settings.mode, runtime.yaw);
        let offset = frame.planar_offset(direction * input.edge_scroll_speed * zoom_scale * dt);
        camera.target_anchor += offset;
    }
}

fn keyboard_rotate_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<(&TopDownCameraInput, &mut TopDownCamera)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (input, mut camera) in &mut cameras {
        if !input.keyboard_rotate_enabled {
            continue;
        }

        let mut rotation = 0.0;
        if keys.pressed(KeyCode::KeyQ) {
            rotation -= 1.0;
        }
        if keys.pressed(KeyCode::KeyE) {
            rotation += 1.0;
        }

        if rotation != 0.0 {
            camera.target_yaw += rotation * input.keyboard_rotate_speed * dt;
        }
    }
}

fn keyboard_zoom_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraSettings,
        &mut TopDownCamera,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    for (input, settings, mut camera) in &mut cameras {
        if !input.keyboard_zoom_enabled {
            continue;
        }

        let mut zoom_dir = 0.0;
        if keys.pressed(KeyCode::Equal) || keys.pressed(KeyCode::NumpadAdd) {
            zoom_dir -= 1.0;
        }
        if keys.pressed(KeyCode::Minus) || keys.pressed(KeyCode::NumpadSubtract) {
            zoom_dir += 1.0;
        }

        if zoom_dir != 0.0 {
            camera.zoom += zoom_dir * input.keyboard_zoom_speed * dt;
            camera.zoom = camera.zoom.clamp(settings.zoom_min, settings.zoom_max);
        }
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
