# Architecture

## Solve Flow

The crate follows the same split used by the other shared camera crates in this workspace:

1. `TopDownCamera` stores desired state and programmatic intent.
2. `TopDownCameraTarget` exposes follow candidates in the world.
3. `TopDownCameraRuntime` stores the smoothed, rendered state.
4. `Transform` and `Projection` are synced from runtime state in `PostUpdate`.

The per-frame flow is:

1. **ResolveTarget**
   Track target motion, compute optional look-ahead, select the active target, and solve a dead-zone / soft-zone corrected anchor goal.
2. **ComputeGoal**
   Clamp programmatic anchor and zoom requests to bounds and min or max ranges.
3. **ApplySmoothing**
   Smooth planar anchor movement, height, yaw, and zoom independently into `TopDownCameraRuntime`.
4. **ComposeEffects**
   Compose all active `TopDownCameraCustomEffects` layers into `render_anchor`, `render_yaw`, `render_zoom`, and `render_fov_delta`. Without custom effects the render fields equal the raw follow state. User effect systems should run **before** this set.
5. **SyncTransform**
   Write the final camera transform from the composed render state.
6. **SyncProjection**
   Update orthographic projection scale from `render_zoom`.
7. **DebugDraw**
   Optionally draw dead zone, bounds, anchor, and tracked-point gizmos.

## Dead Zone, Soft Zone, And Smoothing

The dead zone solve runs against the current rendered anchor, not the desired anchor. That detail matters:

- if the target stays inside the dead zone, the camera goal does not move
- when the target leaves the dead zone but stays inside the soft zone, the goal moves only part of the excess distance
- once the target reaches the outer edge, the goal moves by the full dead-zone excess
- smoothing then decides how fast the rendered camera catches up to that new goal

This avoids the common bug where the goal drifts every frame and effectively drags the dead zone along with it.

## Planar Frames

The crate deliberately uses a shared planar solve for both 2D and 3D:

- `Flat2d`
  The follow plane is world `XY`
- `Tilted3d`
  The follow plane is world `XZ`, rotated into camera-local axes by the current yaw

That choice keeps the dead-zone math and bounds logic mostly pure and unit-testable. It also avoids tying the core follow solve to pixel-space or viewport-space math.

The tradeoff is important:

- the dead zone is expressed in planar world units
- the solve is stable across projection types
- the dead zone is not a literal viewport percentage

## Bounds

Bounds are currently center-only clamps on the anchor point:

- `Flat2d` clamps anchor `XY`
- `Tilted3d` clamps anchor `XZ`

The system does not currently account for visible extents. Orthographic cameras, especially at high scales, can still show beyond the map edge even though the anchor itself is clamped. That limitation is left explicit rather than approximated badly.

## 2D And 3D Support

The runtime supports two view modes:

- `Flat2d { depth }`
  Writes translation on `XY` with a fixed `Z` depth and identity rotation.
- `Tilted3d { pitch, orthographic_distance }`
  Writes a `Camera3d` transform that looks back at the smoothed anchor from a pitched top-down angle.

Projection-specific behavior is handled separately:

- orthographic cameras use `TopDownCamera.zoom` as `OrthographicProjection::scale`
- perspective `Tilted3d` cameras use `TopDownCamera.zoom` as follow distance
- orthographic `Tilted3d` cameras use `orthographic_distance` for transform distance and `zoom` for scale

The runtime does not change `OrthographicProjection::scaling_mode`. That is intentional: scaling mode is a consumer-level policy choice.

- `WindowSize` keeps world units tied to window pixels
- `FixedVertical`, `FixedHorizontal`, or the `Auto*` modes are usually a better fit when zoom should feel like world-space framing

The crate-local lab uses `FixedVertical` specifically so screenshot and BRP verification stay legible across window sizes.

## Target Selection

The runtime supports two target-resolution paths:

- explicit:
  Set `TopDownCamera.tracked_target` to an entity
- automatic:
  Leave `tracked_target` as `None` and the camera will follow the highest-priority enabled `TopDownCameraTarget`

Explicit tracking only requires `Transform`. A `TopDownCameraTarget` component is optional in that path and only adds anchor offset plus look-ahead behavior.

Tie-breaking for automatic targets is deterministic: lower entity index wins when priorities are equal.

If no target is available, the runtime keeps the last desired anchor and does not synthesize a new one.

## Scheduling Guidance

Order downstream systems around the public system sets:

- movement and physics should run before `TopDownCameraSystems::ResolveTarget`
- scripted teleports or camera mutations should run before `TopDownCameraSystems::ComputeGoal`
- consumers that need the final transform should read after `TopDownCameraSystems::SyncTransform`
- orthographic projection reads should happen after `TopDownCameraSystems::SyncProjection`

The examples intentionally move their targets before `ResolveTarget` for that reason.

## Built-In Input Module

The crate provides an optional `TopDownCameraInputPlugin` with a neutral `TopDownCameraInput` tuning component and a bindable `TopDownCameraInputPolicy`. This keeps the framing runtime decoupled from input handling while still providing an ergonomic built-in controller.

`TopDownCameraInputPlugin::default()` runs on `Update`. `TopDownCameraInputPlugin::new(schedule)` lets consumers align input with the same custom schedule used by `TopDownCameraPlugin`.

The input module runs all systems in `TopDownCameraInputSystems::ApplyControls`, ordered `before(TopDownCameraSystems::ResolveTarget)`, and covers:
- keyboard panning (WASD/arrows, zoom-scaled)
- mouse drag panning (pixel-to-world conversion for both 2D and 3D)
- scroll wheel zoom with optional zoom-to-cursor
- edge scrolling (proportional speed based on cursor distance from edge)
- keyboard rotation (Q/E for yaw)
- keyboard zoom (+/- keys)

Input systems only query entities that have `TopDownCameraInput`, `TopDownCameraInputPolicy`, and `TopDownCamera`. Cameras without those components are unaffected.

The policy layer also filters dispatch:
- `ActiveCamera` keeps inactive cameras from consuming the same input stream
- `ActiveViewport` further requires the cursor to be inside the camera viewport
- pointer-driven paths use viewport-local cursor math so drag, edge-scroll, and zoom-to-cursor do not broadcast across every camera in a multi-camera setup

Genre-flavored presets such as RTS or ARPG input live in example code rather than the runtime crate.

## Bounds Soft Margin

`TopDownCameraSettings::bounds_soft_margin` adds an exponential rubber-band feel to bounds clamping. When `> 0.0`, the camera is allowed to slightly overshoot the boundary but is pulled back with exponential falloff (`margin * (1 - e^(-overshoot/margin))`). This avoids the hard stop that can feel jarring in strategy or exploration games.

## Known Tradeoffs

- parented targets should expose a dedicated anchor entity when same-frame accuracy matters
- visible-extent confining is intentionally not part of v1
- 2D mode keeps rotation locked to identity
- target look-ahead uses an internally smoothed velocity estimate to reduce fixed-step jitter, but teleport-heavy games should still clamp look-ahead with `max_look_ahead`
