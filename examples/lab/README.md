# `top_down_camera_lab`

Crate-local showcase and verification app for `top_down_camera`.

## Purpose

- verify dead-zone follow, yaw, zoom, bounds, and target switching in a richer 3D scene
- expose the runtime state through reflected ECS components and an on-screen overlay
- provide a BRP/E2E-friendly app without adding project-specific dependencies to the shared runtime crate

## Run

```bash
cargo run -p top_down_camera_lab
```

## E2E

```bash
cargo run -p top_down_camera_lab --features e2e -- smoke_launch
cargo run -p top_down_camera_lab --features e2e -- top_down_camera_smoke
cargo run -p top_down_camera_lab --features e2e -- top_down_camera_follow
cargo run -p top_down_camera_lab --features e2e -- top_down_camera_bounds
cargo run -p top_down_camera_lab --features e2e -- top_down_camera_zoom
cargo run -p top_down_camera_lab --features e2e -- top_down_camera_target_switch
```

## BRP

```bash
cargo run -p top_down_camera_lab
uv run --project .codex/skills/bevy-brp/script brp world query \
  bevy_ecs::name::Name top_down_camera::components::TopDownCameraRuntime
uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/top_down_camera_lab.png
```

Use the reflected type path reported by `brp world list`, not the crate-root re-export name.
