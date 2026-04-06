# Saddle Camera Top Down Camera

Reusable top-down framing camera for Bevy.

The crate solves the generic follow-framing problem for 2D and near-top-down 3D games: it resolves a tracked anchor, applies a rectangular dead zone, smooths the resulting goal, syncs transform and projection, and optionally visualizes the framing logic with debug gizmos. The runtime stays Bevy-only and project-agnostic.

## What It Is For

- 2D action-adventure and survivors-style follow cameras
- angled 3D top-down or isometric-style follow cameras
- dead-zone framing that avoids camera jitter on small target motion
- optional soft-zone framing that gently recenters before the camera reaches a hard follow response
- smooth follow with per-axis planar damping plus separate height, yaw, and zoom damping
- orthographic zoom, plus perspective 3D distance zoom
- runtime target swapping and crate-local BRP/E2E verification

## What It Is Not For

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
| `TopDownCameraSettings` | Tuning surface for mode, dead zone, soft zone, framing bias, damping, bounds, and zoom limits |
| `TopDownCameraTarget` | Follow candidate marker with priority, anchor offset, optional velocity look-ahead, and enable flag |
| `TopDownCameraRuntime` | Smoothed runtime state plus the active target, tracked point, and current anchor |
| `TopDownCameraBounds` | Center-only planar bounds clamp for the camera anchor |
| `TopDownCameraDebug` | Opt-in debug gizmos for dead zone, bounds, follow anchor, and tracked point |
| `TopDownCameraInput` | Neutral tuning surface for built-in pan, drag, zoom, rotate, and edge-scroll behavior |
| `TopDownCameraInputPolicy` | Bindable input policy component with active-camera filtering and mapping tables |
| `TopDownCameraInputBindingTable` | Keyboard and mouse mapping data used by the built-in controller |
| `TopDownCameraInputPlugin` | Optional plugin that adds input processing for cameras with `TopDownCameraInput` plus `TopDownCameraInputPolicy`; `new(schedule)` injects the update schedule |

## Dead Zone Semantics

The dead zone is defined in camera-local planar units, not normalized screen fractions:

- `Flat2d` mode uses world `XY`
- `Tilted3d` mode uses world `XZ`, aligned to the camera's current yaw

While the tracked point stays inside that rectangle, the camera anchor does not move. Once the tracked point leaves the rectangle, the camera goal moves only by the excess needed to place the target back on the dead-zone edge.

This makes the solve projection-agnostic and reusable across 2D and 3D. The tradeoff is that the dead zone is not expressed directly in pixels or viewport percentages.

## Soft Zone Semantics

`soft_zone` is an optional outer framing band that sits outside the dead zone:

- when `soft_zone == dead_zone`, the solve keeps the original dead-zone-only behavior
- when the target leaves the dead zone but stays inside the soft zone, the camera recenters only partially
- once the target reaches the outer edge, the solve applies the full dead-zone correction

This mirrors the common “dead zone + soft follow” pattern from action-adventure and ARPG cameras while keeping the solve in planar world units.

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

`TopDownCameraSettings::bounds_soft_margin` controls the clamping feel:
- `0.0` (default): hard clamp at the boundary edge
- `> 0.0`: exponential falloff that gently pushes the anchor back, creating a rubber-band feel

The current implementation does not shrink bounds based on visible extents, so an orthographic camera can still show past the edge if its zoom scale is large enough. That tradeoff is deliberate for the first version and is documented here rather than hidden.

## Input Model

The crate provides an optional `TopDownCameraInputPlugin` with a neutral `TopDownCameraInput` tuning component and a bindable `TopDownCameraInputPolicy`. Attach both to any camera entity alongside `TopDownCamera` and `TopDownCameraSettings` to enable:

- **Keyboard panning** (WASD / arrows) with zoom-scaled speed
- **Mouse drag panning** (configurable button, default middle)
- **Scroll wheel zoom** with optional zoom-to-cursor
- **Edge scrolling** (camera moves when cursor is near screen edges)
- **Keyboard rotation** (Q/E for yaw in `Tilted3d` mode)
- **Keyboard zoom** (+/- keys)

`TopDownCameraInputPolicy` exposes:

- a `TopDownCameraInputBindingTable` for keyboard axes and mouse drag buttons
- `TopDownCameraInputTargetFilter` for `AnyCamera`, `ActiveCamera`, or `ActiveViewport` dispatch
- viewport-aware mouse routing so drag, scroll, and edge-scroll do not broadcast across every camera

The core crate keeps only neutral defaults. Genre presets such as the RTS and ARPG setups now live in example code, where they compose `TopDownCameraInput` and `TopDownCameraInputPolicy` explicitly.

Consumers who need custom input handling can skip `TopDownCameraInputPlugin`, or reuse the public binding-table types while mutating `TopDownCamera` directly.

`TopDownCameraInputPlugin::default()` runs on `Update`. Use `TopDownCameraInputPlugin::new(schedule)` when the camera runtime is driven from a custom schedule and order it before `TopDownCameraSystems::ResolveTarget`.

The examples and lab also include a `bevy_enhanced_input` bridge for target movement and camera controls, but that code lives outside the runtime crate.

Rotation lock is achieved by leaving `TopDownCamera.target_yaw` unchanged. `Flat2d` mode always locks rotation to identity; `Tilted3d` only rotates when a consumer mutates yaw.

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic_2d` | 2D arena follow with dead-zone / soft-zone tuning exposed through `saddle-pane` | `cargo run -p saddle-camera-top-down-camera-example-basic-2d` |
| `basic_3d` | Perspective `Camera3d` follow with pitch, yaw, and zoom surfaced live in `saddle-pane` | `cargo run -p saddle-camera-top-down-camera-example-basic-3d` |
| `bounds` | 2D center-bounds clamp with debug gizmos and live framing controls | `cargo run -p saddle-camera-top-down-camera-example-bounds` |
| `target_switching` | Explicit runtime retargeting between two actors while the pane edits framing in real time | `cargo run -p saddle-camera-top-down-camera-example-target-switching` |
| `soft_zone_framing` | Dedicated soft-zone showcase for the new partial recentering behavior | `cargo run -p saddle-camera-top-down-camera-example-soft-zone-framing` |
| `optional_controls` | `bevy_enhanced_input` bridge that moves the target and adjusts camera yaw and zoom | `cargo run -p saddle-camera-top-down-camera-example-optional-controls` |
| `strategy_game` | Strategy/RTS camera with edge scrolling, zoom-to-cursor, map bounds, and soft clamping | `cargo run -p saddle-camera-top-down-camera-example-strategy-game` |
| `arpg_camera` | ARPG-style follow camera with character movement, look-ahead, and target switching | `cargo run -p saddle-camera-top-down-camera-example-arpg-camera` |

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
cargo run -p saddle-camera-top-down-camera-lab --features e2e -- top_down_camera_soft_zone
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
