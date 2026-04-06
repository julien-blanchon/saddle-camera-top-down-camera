use bevy::{
    camera::RenderTarget,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
    window::WindowRef,
};

use crate::{
    TopDownCamera, TopDownCameraMode, TopDownCameraRuntime, TopDownCameraSettings,
    TopDownCameraSystems, math::planar_frame,
};

/// High-level ordering hook for the built-in input controller.
#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TopDownCameraInputSystems {
    ApplyControls,
}

/// Bindable signed keyboard axis used by the built-in camera controller.
#[derive(Reflect, Clone, Debug, Default, PartialEq, Eq)]
pub struct TopDownCameraKeyAxisBinding {
    pub negative: Vec<KeyCode>,
    pub positive: Vec<KeyCode>,
}

impl TopDownCameraKeyAxisBinding {
    pub fn new(
        negative: impl IntoIterator<Item = KeyCode>,
        positive: impl IntoIterator<Item = KeyCode>,
    ) -> Self {
        Self {
            negative: negative.into_iter().collect(),
            positive: positive.into_iter().collect(),
        }
    }

    pub fn value(&self, keys: &ButtonInput<KeyCode>) -> f32 {
        let negative = self.negative.iter().any(|key| keys.pressed(*key));
        let positive = self.positive.iter().any(|key| keys.pressed(*key));

        match (negative, positive) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        }
    }
}

/// Keyboard and mouse mapping table used by the built-in controller.
#[derive(Reflect, Clone, Debug, PartialEq, Eq)]
pub struct TopDownCameraInputBindingTable {
    pub keyboard_pan_x: TopDownCameraKeyAxisBinding,
    pub keyboard_pan_y: TopDownCameraKeyAxisBinding,
    pub keyboard_rotate: TopDownCameraKeyAxisBinding,
    pub keyboard_zoom: TopDownCameraKeyAxisBinding,
    pub mouse_drag_buttons: Vec<MouseButton>,
}

impl TopDownCameraInputBindingTable {
    pub fn keyboard_pan(&self, keys: &ButtonInput<KeyCode>) -> Vec2 {
        Vec2::new(
            self.keyboard_pan_x.value(keys),
            self.keyboard_pan_y.value(keys),
        )
    }

    pub fn keyboard_rotate(&self, keys: &ButtonInput<KeyCode>) -> f32 {
        self.keyboard_rotate.value(keys)
    }

    pub fn keyboard_zoom(&self, keys: &ButtonInput<KeyCode>) -> f32 {
        self.keyboard_zoom.value(keys)
    }

    pub fn mouse_drag_active(&self, buttons: &ButtonInput<MouseButton>) -> bool {
        self.mouse_drag_buttons
            .iter()
            .any(|button| buttons.pressed(*button))
    }
}

impl Default for TopDownCameraInputBindingTable {
    fn default() -> Self {
        Self {
            keyboard_pan_x: TopDownCameraKeyAxisBinding::new(
                [KeyCode::KeyA, KeyCode::ArrowLeft],
                [KeyCode::KeyD, KeyCode::ArrowRight],
            ),
            keyboard_pan_y: TopDownCameraKeyAxisBinding::new(
                [KeyCode::KeyS, KeyCode::ArrowDown],
                [KeyCode::KeyW, KeyCode::ArrowUp],
            ),
            keyboard_rotate: TopDownCameraKeyAxisBinding::new([KeyCode::KeyQ], [KeyCode::KeyE]),
            keyboard_zoom: TopDownCameraKeyAxisBinding::new(
                [KeyCode::Equal, KeyCode::NumpadAdd],
                [KeyCode::Minus, KeyCode::NumpadSubtract],
            ),
            mouse_drag_buttons: vec![MouseButton::Middle],
        }
    }
}

/// Which camera target is allowed to consume built-in input on a frame.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TopDownCameraInputTargetFilter {
    /// Apply input to every camera with matching input components.
    AnyCamera,
    /// Only apply input to active cameras.
    #[default]
    ActiveCamera,
    /// Only apply input to active cameras whose viewport currently contains the cursor.
    ActiveViewport,
}

/// Built-in input policy: bindings plus camera-target filtering.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct TopDownCameraInputPolicy {
    pub target_filter: TopDownCameraInputTargetFilter,
    pub bindings: TopDownCameraInputBindingTable,
}

impl Default for TopDownCameraInputPolicy {
    fn default() -> Self {
        Self {
            target_filter: TopDownCameraInputTargetFilter::ActiveCamera,
            bindings: TopDownCameraInputBindingTable::default(),
        }
    }
}

/// Configuration component for built-in top-down camera input handling.
///
/// Attach this to any entity that also has [`TopDownCamera`],
/// [`TopDownCameraSettings`], and [`TopDownCameraInputPolicy`] to enable the
/// built-in controller.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct TopDownCameraInput {
    /// Enable keyboard panning.
    pub keyboard_pan_enabled: bool,
    /// Keyboard pan speed in world units per second.
    pub keyboard_pan_speed: f32,

    /// Enable mouse drag panning.
    pub mouse_drag_enabled: bool,

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

    /// Enable keyboard rotation.
    pub keyboard_rotate_enabled: bool,
    /// Rotation speed in radians per second.
    pub keyboard_rotate_speed: f32,

    /// Enable keyboard zoom.
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

/// Optional plugin that adds built-in input handling for top-down cameras.
///
/// Attach both [`TopDownCameraInput`] and [`TopDownCameraInputPolicy`] to any
/// camera entity that also has [`TopDownCamera`] and [`TopDownCameraSettings`]
/// to activate input processing.
pub struct TopDownCameraInputPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl TopDownCameraInputPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }
}

impl Default for TopDownCameraInputPlugin {
    fn default() -> Self {
        Self::new(Update)
    }
}

impl Plugin for TopDownCameraInputPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TopDownCameraInput>()
            .register_type::<TopDownCameraInputBindingTable>()
            .register_type::<TopDownCameraInputPolicy>()
            .register_type::<TopDownCameraInputTargetFilter>()
            .register_type::<TopDownCameraKeyAxisBinding>()
            .configure_sets(
                self.update_schedule,
                TopDownCameraInputSystems::ApplyControls
                    .before(TopDownCameraSystems::ResolveTarget),
            )
            .add_systems(
                self.update_schedule,
                (
                    keyboard_pan_system,
                    mouse_drag_pan_system,
                    scroll_zoom_system,
                    edge_scroll_system,
                    keyboard_rotate_system,
                    keyboard_zoom_system,
                )
                    .in_set(TopDownCameraInputSystems::ApplyControls),
            );
    }
}

fn keyboard_pan_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraInputPolicy,
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &Camera,
        Option<&RenderTarget>,
        &mut TopDownCamera,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let primary_window = primary_window.single().ok();

    for (input, policy, settings, runtime, camera_component, render_target, mut camera) in
        &mut cameras
    {
        if !input.keyboard_pan_enabled {
            continue;
        }
        if !matches_keyboard_target(
            policy,
            camera_component,
            render_target,
            &windows,
            primary_window,
        ) {
            continue;
        }

        let mut direction = policy.bindings.keyboard_pan(&keys);
        if direction == Vec2::ZERO {
            continue;
        }

        direction = direction.normalize();

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
    windows: Query<&Window>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraInputPolicy,
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &Camera,
        Option<&RenderTarget>,
        &mut TopDownCamera,
    )>,
) {
    let accumulated = accumulated_motion.delta;
    if accumulated == Vec2::ZERO {
        return;
    }

    let primary_window = primary_window.single().ok();

    for (input, policy, settings, runtime, camera_component, render_target, mut camera) in
        &mut cameras
    {
        if !input.mouse_drag_enabled {
            continue;
        }
        if !policy.bindings.mouse_drag_active(&mouse_buttons) {
            continue;
        }
        if !matches_pointer_target(
            policy,
            camera_component,
            render_target,
            &windows,
            primary_window,
        ) {
            continue;
        }

        let frame = planar_frame(settings.mode, runtime.yaw);
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
    windows: Query<&Window>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraInputPolicy,
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &Camera,
        Option<&RenderTarget>,
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

    let primary_window = primary_window.single().ok();

    for (input, policy, settings, runtime, camera_component, render_target, mut camera) in
        &mut cameras
    {
        if !input.scroll_zoom_enabled {
            continue;
        }
        if !matches_pointer_target(
            policy,
            camera_component,
            render_target,
            &windows,
            primary_window,
        ) {
            continue;
        }

        let old_zoom = camera.zoom;
        camera.zoom -= scroll_delta * input.scroll_zoom_sensitivity * camera.zoom;
        camera.zoom = camera.zoom.clamp(settings.zoom_min, settings.zoom_max);

        if !input.zoom_to_cursor || (camera.zoom - old_zoom).abs() < f32::EPSILON {
            continue;
        }

        let Some((cursor_pos, viewport_size)) =
            cursor_in_viewport(camera_component, render_target, &windows, primary_window)
        else {
            continue;
        };

        match settings.mode {
            TopDownCameraMode::Flat2d { .. } => {
                let normalized = Vec2::new(
                    (cursor_pos.x / viewport_size.x - 0.5) * 2.0,
                    -(cursor_pos.y / viewport_size.y - 0.5) * 2.0,
                );
                let aspect = viewport_size.x / viewport_size.y.max(1.0);

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

fn edge_scroll_system(
    time: Res<Time>,
    windows: Query<&Window>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraInputPolicy,
        &TopDownCameraSettings,
        &TopDownCameraRuntime,
        &Camera,
        Option<&RenderTarget>,
        &mut TopDownCamera,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let primary_window = primary_window.single().ok();

    for (input, policy, settings, runtime, camera_component, render_target, mut camera) in
        &mut cameras
    {
        if !input.edge_scroll_enabled {
            continue;
        }

        let margin = input.edge_scroll_margin;
        if margin <= 0.0 {
            continue;
        }

        let Some((cursor_pos, viewport_size)) =
            cursor_in_viewport(camera_component, render_target, &windows, primary_window)
        else {
            continue;
        };

        if !matches_pointer_target(
            policy,
            camera_component,
            render_target,
            &windows,
            primary_window,
        ) {
            continue;
        }

        let mut direction = Vec2::ZERO;

        if cursor_pos.x > viewport_size.x - margin {
            direction.x += ((cursor_pos.x - (viewport_size.x - margin)) / margin).clamp(0.0, 1.0);
        }
        if cursor_pos.x < margin {
            direction.x -= ((margin - cursor_pos.x) / margin).clamp(0.0, 1.0);
        }
        if cursor_pos.y < margin {
            direction.y += ((margin - cursor_pos.y) / margin).clamp(0.0, 1.0);
        }
        if cursor_pos.y > viewport_size.y - margin {
            direction.y -= ((cursor_pos.y - (viewport_size.y - margin)) / margin).clamp(0.0, 1.0);
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
    windows: Query<&Window>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraInputPolicy,
        &Camera,
        Option<&RenderTarget>,
        &mut TopDownCamera,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let primary_window = primary_window.single().ok();

    for (input, policy, camera_component, render_target, mut camera) in &mut cameras {
        if !input.keyboard_rotate_enabled {
            continue;
        }
        if !matches_keyboard_target(
            policy,
            camera_component,
            render_target,
            &windows,
            primary_window,
        ) {
            continue;
        }

        let rotation = policy.bindings.keyboard_rotate(&keys);
        if rotation != 0.0 {
            camera.target_yaw += rotation * input.keyboard_rotate_speed * dt;
        }
    }
}

fn keyboard_zoom_system(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    primary_window: Query<Entity, With<bevy::window::PrimaryWindow>>,
    mut cameras: Query<(
        &TopDownCameraInput,
        &TopDownCameraInputPolicy,
        &TopDownCameraSettings,
        &Camera,
        Option<&RenderTarget>,
        &mut TopDownCamera,
    )>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let primary_window = primary_window.single().ok();

    for (input, policy, settings, camera_component, render_target, mut camera) in &mut cameras {
        if !input.keyboard_zoom_enabled {
            continue;
        }
        if !matches_keyboard_target(
            policy,
            camera_component,
            render_target,
            &windows,
            primary_window,
        ) {
            continue;
        }

        let zoom_dir = policy.bindings.keyboard_zoom(&keys);
        if zoom_dir != 0.0 {
            camera.zoom += zoom_dir * input.keyboard_zoom_speed * dt;
            camera.zoom = camera.zoom.clamp(settings.zoom_min, settings.zoom_max);
        }
    }
}

fn matches_keyboard_target(
    policy: &TopDownCameraInputPolicy,
    camera: &Camera,
    render_target: Option<&RenderTarget>,
    windows: &Query<&Window>,
    primary_window: Option<Entity>,
) -> bool {
    matches_input_target(
        policy.target_filter,
        camera,
        render_target,
        windows,
        primary_window,
        false,
    )
}

fn matches_pointer_target(
    policy: &TopDownCameraInputPolicy,
    camera: &Camera,
    render_target: Option<&RenderTarget>,
    windows: &Query<&Window>,
    primary_window: Option<Entity>,
) -> bool {
    matches_input_target(
        policy.target_filter,
        camera,
        render_target,
        windows,
        primary_window,
        true,
    )
}

fn matches_input_target(
    filter: TopDownCameraInputTargetFilter,
    camera: &Camera,
    render_target: Option<&RenderTarget>,
    windows: &Query<&Window>,
    primary_window: Option<Entity>,
    require_cursor_in_viewport: bool,
) -> bool {
    if !matches!(filter, TopDownCameraInputTargetFilter::AnyCamera) && !camera.is_active {
        return false;
    }

    let window_entity = camera_window_entity(render_target, primary_window);
    if let Some(window_entity) = window_entity {
        let Ok(window) = windows.get(window_entity) else {
            return false;
        };
        if !matches!(filter, TopDownCameraInputTargetFilter::AnyCamera) && !window.focused {
            return false;
        }
    }

    if require_cursor_in_viewport
        || matches!(filter, TopDownCameraInputTargetFilter::ActiveViewport)
    {
        return cursor_in_viewport(camera, render_target, windows, primary_window).is_some();
    }

    true
}

fn cursor_in_viewport(
    camera: &Camera,
    render_target: Option<&RenderTarget>,
    windows: &Query<&Window>,
    primary_window: Option<Entity>,
) -> Option<(Vec2, Vec2)> {
    let window_entity = camera_window_entity(render_target, primary_window)?;
    let window = windows.get(window_entity).ok()?;
    let cursor_position = window.cursor_position()?;
    let viewport = camera.logical_viewport_rect()?;
    let viewport_size = viewport.size();
    let viewport_cursor = cursor_position - viewport.min;

    if viewport_cursor.x < 0.0
        || viewport_cursor.y < 0.0
        || viewport_cursor.x > viewport_size.x
        || viewport_cursor.y > viewport_size.y
    {
        return None;
    }

    Some((viewport_cursor, viewport_size))
}

fn camera_window_entity(
    render_target: Option<&RenderTarget>,
    primary_window: Option<Entity>,
) -> Option<Entity> {
    let render_target = render_target?;

    match render_target {
        RenderTarget::Window(WindowRef::Primary) => primary_window,
        RenderTarget::Window(WindowRef::Entity(entity)) => Some(*entity),
        _ => None,
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
