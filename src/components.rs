use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct TopDownCamera {
    pub target_anchor: Vec3,
    pub target_yaw: f32,
    pub zoom: f32,
    pub tracked_target: Option<Entity>,
    pub follow_enabled: bool,
    pub snap: bool,
}

impl TopDownCamera {
    pub fn new(target_anchor: Vec3) -> Self {
        Self {
            target_anchor,
            ..default()
        }
    }

    pub fn looking_at_3d(target_anchor: Vec3, yaw: f32, distance: f32) -> Self {
        Self {
            target_anchor,
            target_yaw: yaw,
            zoom: distance,
            ..default()
        }
    }

    pub fn snap_to(&mut self, target_anchor: Vec3, target_yaw: f32, zoom: f32) {
        self.target_anchor = target_anchor;
        self.target_yaw = target_yaw;
        self.zoom = zoom;
        self.snap = true;
    }
}

impl Default for TopDownCamera {
    fn default() -> Self {
        Self {
            target_anchor: Vec3::ZERO,
            target_yaw: 0.0,
            zoom: 1.0,
            tracked_target: None,
            follow_enabled: true,
            snap: false,
        }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub enum TopDownCameraMode {
    Flat2d {
        depth: f32,
    },
    Tilted3d {
        pitch: f32,
        orthographic_distance: f32,
    },
}

impl TopDownCameraMode {
    pub fn flat_2d(depth: f32) -> Self {
        Self::Flat2d { depth }
    }

    pub fn tilted_3d(pitch: f32, orthographic_distance: f32) -> Self {
        Self::Tilted3d {
            pitch,
            orthographic_distance,
        }
    }
}

impl Default for TopDownCameraMode {
    fn default() -> Self {
        Self::Flat2d { depth: 1000.0 }
    }
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct TopDownCameraBounds {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Reflect, Clone, Copy, Debug, PartialEq)]
pub struct TopDownCameraDamping {
    pub planar_x: f32,
    pub planar_y: f32,
    pub height: f32,
    pub zoom: f32,
    pub yaw: f32,
}

impl Default for TopDownCameraDamping {
    fn default() -> Self {
        Self {
            planar_x: 9.0,
            planar_y: 9.0,
            height: 11.0,
            zoom: 12.0,
            yaw: 10.0,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct TopDownCameraSettings {
    pub mode: TopDownCameraMode,
    pub dead_zone: Vec2,
    pub soft_zone: Vec2,
    pub bias: Vec2,
    pub damping: TopDownCameraDamping,
    pub bounds: Option<TopDownCameraBounds>,
    /// Soft margin in world units around the bounds.
    /// When greater than zero, the camera is gently pushed back with
    /// exponential falloff instead of hard-clamped, creating a rubber-band
    /// feel. Set to `0.0` (default) for the original hard-clamp behavior.
    pub bounds_soft_margin: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub zoom_speed: f32,
}

impl TopDownCameraSettings {
    pub fn flat_2d(depth: f32) -> Self {
        Self {
            mode: TopDownCameraMode::flat_2d(depth),
            ..default()
        }
    }

    pub fn tilted_3d(pitch: f32, orthographic_distance: f32) -> Self {
        Self {
            mode: TopDownCameraMode::tilted_3d(pitch, orthographic_distance),
            zoom_min: 6.0,
            zoom_max: 36.0,
            zoom_speed: 2.0,
            ..Self::default()
        }
    }
}

impl Default for TopDownCameraSettings {
    fn default() -> Self {
        Self {
            mode: TopDownCameraMode::default(),
            dead_zone: Vec2::new(96.0, 72.0),
            soft_zone: Vec2::new(96.0, 72.0),
            bias: Vec2::ZERO,
            damping: TopDownCameraDamping::default(),
            bounds: None,
            bounds_soft_margin: 0.0,
            zoom_min: 0.5,
            zoom_max: 4.0,
            zoom_speed: 0.2,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug)]
#[reflect(Component)]
pub struct TopDownCameraTarget {
    pub priority: i32,
    pub anchor_offset: Vec3,
    pub look_ahead_time: Vec2,
    pub max_look_ahead: Vec2,
    pub enabled: bool,
}

impl Default for TopDownCameraTarget {
    fn default() -> Self {
        Self {
            priority: 0,
            anchor_offset: Vec3::ZERO,
            look_ahead_time: Vec2::ZERO,
            max_look_ahead: Vec2::splat(f32::INFINITY),
            enabled: true,
        }
    }
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct TopDownCameraRuntime {
    pub active_target: Option<Entity>,
    pub follow_anchor: Vec3,
    pub goal_anchor: Vec3,
    pub tracked_point: Vec3,
    pub yaw: f32,
    pub zoom: f32,
    /// Follow anchor with custom effects applied. Written by the compose
    /// effects stage; read by transform sync.
    pub render_anchor: Vec3,
    /// Yaw with custom effects applied.
    pub render_yaw: f32,
    /// Zoom with custom effects applied and clamped.
    pub render_zoom: f32,
    /// FOV delta from custom effects (perspective cameras only).
    pub render_fov_delta: f32,
}

impl TopDownCameraRuntime {
    pub fn from_camera(camera: &TopDownCamera) -> Self {
        Self {
            active_target: camera.tracked_target,
            follow_anchor: camera.target_anchor,
            goal_anchor: camera.target_anchor,
            tracked_point: camera.target_anchor,
            yaw: camera.target_yaw,
            zoom: camera.zoom,
            render_anchor: camera.target_anchor,
            render_yaw: camera.target_yaw,
            render_zoom: camera.zoom,
            render_fov_delta: 0.0,
        }
    }
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq)]
#[reflect(Component)]
pub struct TopDownCameraDebug {
    pub draw_dead_zone: bool,
    pub draw_bounds: bool,
    pub draw_targets: bool,
}

impl Default for TopDownCameraDebug {
    fn default() -> Self {
        Self {
            draw_dead_zone: true,
            draw_bounds: true,
            draw_targets: true,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub(crate) struct TopDownCameraTargetState {
    pub previous_anchor: Vec3,
    pub velocity: Vec3,
}
