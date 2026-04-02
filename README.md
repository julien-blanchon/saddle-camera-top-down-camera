# Saddle Camera Top Down Camera

Reusable top-down framing camera for Bevy.

The crate solves the generic follow-framing problem for 2D and near-top-down 3D games: it resolves a tracked anchor, applies a rectangular dead zone, smooths the resulting goal, syncs transform and projection, and optionally visualizes the framing logic with debug gizmos. The runtime stays Bevy-only and project-agnostic.

## What It Is For

- 2D action-adventure and survivors-style follow cameras
- angled 3D top-down or isometric-style follow cameras
- dead-zone framing that avoids camera jitter on small target motion
- smooth follow with per-axis planar damping plus separate height, yaw, and zoom damping
- orthographic zoom, plus perspective 3D distance zoom
- runtime target swapping and crate-local BRP/E2E verification

## What It Is Not For

- a bundled input stack
- gameplay-specific target selection rules
- polygon or spline confiners
- visible-extent bounds solving
- spring-arm collision avoidance

## Quick Start

```toml
[dependencies]
saddle-camera-top-down-camera = { git = "https://github.com/julien-blanchon/saddle-camera-top-down-camera" }
bevy = "0.18"
```

```rust,no_run
use bevy::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCamera, TopDownCameraPlugin, TopDownCameraSettings, TopDownCameraTarget,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TopDownCameraPlugin::default()))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        TopDownCameraTarget::default(),
    ));

    commands.spawn((
        Name::new("Top Down Camera"),
        Camera2d,
        TopDownCamera::new(Vec3::ZERO),
        TopDownCameraSettings::flat_2d(999.0),
    ));
}
```

For always-on tools and examples, `TopDownCameraPlugin::always_on(Update)` is the convenience constructor.

## Public API

| Type | Purpose |
| --- | --- |
| `TopDownCameraPlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `TopDownCameraSystems` | Public ordering hooks: `ResolveTarget`, `ComputeGoal`, `ApplySmoothing`, `SyncTransform`, `SyncProjection`, `DebugDraw` |
| `TopDownCamera` | Desired camera state: target anchor, yaw, zoom, explicit tracked target, follow toggle, snap requests |
| `TopDownCameraSettings` | Tuning surface for mode, dead zone, framing bias, damping, bounds, and zoom limits |
| `TopDownCameraTarget` | Follow candidate marker with priority, anchor offset, optional velocity look-ahead, and enable flag |
| `TopDownCameraRuntime` | Smoothed runtime state plus the active target, tracked point, and current anchor |
| `TopDownCameraBounds` | Center-only planar bounds clamp for the camera anchor |
| `TopDownCameraDebug` | Opt-in debug gizmos for dead zone, bounds, follow anchor, and tracked point |

## Dead Zone Semantics

The dead zone is defined in camera-local planar units, not normalized screen fractions:

- `Flat2d` mode uses world `XY`
- `Tilted3d` mode uses world `XZ`, aligned to the camera's current yaw

While the tracked point stays inside that rectangle, the camera anchor does not move. Once the tracked point leaves the rectangle, the camera goal moves only by the excess needed to place the target back on the dead-zone edge.

This makes the solve projection-agnostic and reusable across 2D and 3D. The tradeoff is that the dead zone is not expressed directly in pixels or viewport percentages.

## Zoom Semantics

- Orthographic cameras:
  `TopDownCamera.zoom` drives `OrthographicProjection::scale`
- Perspective `Tilted3d` cameras:
  `TopDownCamera.zoom` drives the camera distance from the follow anchor

`TopDownCameraSettings::zoom_min`, `zoom_max`, and `zoom_speed` are interpreted in those same units.

For orthographic cameras, visible extents still depend on `OrthographicProjection::scaling_mode`. The crate only owns `scale`; consumers should pick a scaling mode that matches their game. The crate-local 3D lab uses `ScalingMode::FixedVertical` so zoom behaves in stable world units instead of window pixels.

## Bounds Semantics

`TopDownCameraBounds` clamp the camera anchor center only:

- `Flat2d`: bounds apply to the anchor on `XY`
- `Tilted3d`: bounds apply to the anchor on `XZ`

The current implementation does not shrink bounds based on visible extents, so an orthographic camera can still show past the edge if its zoom scale is large enough. That tradeoff is deliberate for the first version and is documented here rather than hidden.

## Input Model

The shared runtime crate does not own input. Consumers should mutate `TopDownCamera` directly or adapt their own input layer into it.

The examples and lab include a small `bevy_enhanced_input` bridge, but that code lives outside the runtime crate so downstream games can swap it for their own bindings without forking the camera logic.

Rotation lock is achieved by leaving `TopDownCamera.target_yaw` unchanged. `Flat2d` mode always locks rotation to identity; `Tilted3d` only rotates when a consumer mutates yaw.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic_2d` | Minimal 2D orthographic follow with a centered dead zone | `cargo run -p saddle-camera-top-down-camera-example-basic-2d` |
| `basic_3d` | Perspective `Camera3d` follow with pitch, yaw, and velocity look-ahead | `cargo run -p saddle-camera-top-down-camera-example-basic-3d` |
| `bounds` | 2D center-bounds clamp with debug gizmos | `cargo run -p saddle-camera-top-down-camera-example-bounds` |
| `target_switching` | Explicit runtime retargeting between two actors | `cargo run -p saddle-camera-top-down-camera-example-target-switching` |
| `optional_controls` | `bevy_enhanced_input` bridge that moves the target and adjusts camera yaw and zoom | `cargo run -p saddle-camera-top-down-camera-example-optional-controls` |

## Workspace Lab

The richer lab app lives inside the crate at `shared/camera/saddle-camera-top-down-camera/examples/lab`:

```bash
cargo run -p saddle-camera-top-down-camera-lab
```

With E2E enabled:

```bash
cargo run -p saddle-camera-top-down-camera-lab --features e2e -- smoke_launch
cargo run -p saddle-camera-top-down-camera-lab --features e2e -- top_down_camera_smoke
cargo run -p saddle-camera-top-down-camera-lab --features e2e -- top_down_camera_follow
cargo run -p saddle-camera-top-down-camera-lab --features e2e -- top_down_camera_bounds
cargo run -p saddle-camera-top-down-camera-lab --features e2e -- top_down_camera_zoom
cargo run -p saddle-camera-top-down-camera-lab --features e2e -- top_down_camera_target_switch
```

## Known Limitations

- bounds clamp the anchor center only
- the runtime reads `Transform`, not `GlobalTransform`, so parented targets should expose a dedicated anchor entity if they need precise same-frame follow
- 2D mode intentionally ignores camera rotation
- there is no built-in collision/confiner system beyond planar center bounds
- window resize does not change the dead-zone solve because it runs in planar world units, but a resize can still change visible extents depending on the projection scaling mode

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
