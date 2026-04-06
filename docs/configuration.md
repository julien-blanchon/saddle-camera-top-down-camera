# Configuration

## `TopDownCamera`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `target_anchor` | `Vec3` | `Vec3::ZERO` | Any finite value | Desired anchor point. In 2D this is world `XY`; in 3D the follow solve uses `XZ` plus the target's `Y`. |
| `target_yaw` | `f32` | `0.0` | Any finite radians value | Desired camera yaw for `Tilted3d`. Follow logic does not modify it. |
| `zoom` | `f32` | `1.0` | Clamped by `TopDownCameraSettings::zoom_min` and `zoom_max` | Orthographic scale for orthographic cameras, or distance for perspective `Tilted3d` cameras. |
| `tracked_target` | `Option<Entity>` | `None` | Valid entity or `None` | Explicit target override. `None` enables automatic target selection from `TopDownCameraTarget`. |
| `follow_enabled` | `bool` | `true` | `true` or `false` | Suspends automatic anchor solving while still keeping the runtime active. |
| `snap` | `bool` | `false` | `true` or `false` | When set, the runtime snaps anchor, yaw, and zoom to their desired targets on the next update. |

## `TopDownCameraSettings`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `mode` | `TopDownCameraMode` | `Flat2d { depth: 1000.0 }` | `Flat2d` or `Tilted3d` | Selects the view-mode transform solve. |
| `dead_zone` | `Vec2` | `(96.0, 72.0)` | Non-negative size | Size of the rectangular dead zone in camera-local planar units. |
| `soft_zone` | `Vec2` | `(96.0, 72.0)` | Per-axis value greater than or equal to `dead_zone` recommended | Optional outer framing band that blends from “no motion” into full recentering. |
| `bias` | `Vec2` | `Vec2::ZERO` | Any finite value | Offsets the point the target is held around inside the dead zone. |
| `damping` | `TopDownCameraDamping` | See below | Non-negative decay rates | Controls smoothing for planar motion, height, zoom, and yaw. |
| `bounds` | `Option<TopDownCameraBounds>` | `None` | `None` or finite min/max pair | Center-only clamp for the camera anchor on the active follow plane. |
| `bounds_soft_margin` | `f32` | `0.0` | Non-negative | Soft margin around bounds. `0.0` = hard clamp. `> 0.0` = exponential rubber-band falloff in world units. |
| `zoom_min` | `f32` | `0.5` | Less than or equal to `zoom_max` | Lower zoom clamp. For perspective 3D this is minimum distance. |
| `zoom_max` | `f32` | `4.0` | Greater than or equal to `zoom_min` | Upper zoom clamp. For perspective 3D this is maximum distance. |
| `zoom_speed` | `f32` | `0.2` | Non-negative | Convenience tuning for example or consumer input adapters. The runtime itself does not read input. |

### Dead Zone Semantics

`dead_zone` is a size, not half-extents:

- width = `dead_zone.x`
- height = `dead_zone.y`

The solver internally uses half-extents when checking whether the tracked point lies outside the rectangle.

### Soft Zone Semantics

`soft_zone` uses the same planar units as `dead_zone`:

- matching `dead_zone` preserves the legacy behavior
- larger values add a gentler recentering band outside the dead zone
- values smaller than `dead_zone` are treated as if they matched `dead_zone`

### Bias Semantics

`bias` is expressed in the same planar units as `dead_zone`:

- positive `x` biases the tracked point toward the camera's right
- positive `y` biases upward in `Flat2d`
- positive `y` biases away from the camera in `Tilted3d`, because the second planar axis is the camera's forward direction projected onto the ground plane

For predictable framing, keep the absolute value of each bias component smaller than half of the corresponding dead-zone size.

## `TopDownCameraMode`

### `Flat2d { depth }`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `depth` | `f32` | `1000.0` | Any finite value | Fixed camera `Z` translation written in 2D mode. |

### `Tilted3d { pitch, orthographic_distance }`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `pitch` | `f32` | No default constructor field | `0 < pitch < π/2` recommended | Downward tilt used by the 3D transform solve. |
| `orthographic_distance` | `f32` | No default constructor field | Positive | Camera distance used only when the projection is orthographic. Perspective cameras use `TopDownCamera.zoom` instead. |

For orthographic cameras, `TopDownCamera.zoom` still only changes `OrthographicProjection::scale`. The visible world size is therefore the combination of `scale` and the projection's `scaling_mode`.

## `TopDownCameraDamping`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `planar_x` | `f32` | `9.0` | Non-negative | Decay rate for the first planar axis. In 2D this is `X`; in 3D it is camera-local right. |
| `planar_y` | `f32` | `9.0` | Non-negative | Decay rate for the second planar axis. In 2D this is `Y`; in 3D it is camera-local forward on the ground plane. |
| `height` | `f32` | `11.0` | Non-negative | Vertical smoothing for `Tilted3d` anchor height. Ignored in `Flat2d`. |
| `zoom` | `f32` | `12.0` | Non-negative | Zoom smoothing for orthographic scale or perspective distance. |
| `yaw` | `f32` | `10.0` | Non-negative | Yaw smoothing for `Tilted3d`. |

Higher values are snappier because the crate uses Bevy's frame-rate-stable `smooth_nudge` decay model.

## `TopDownCameraBounds`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `min` | `Vec2` | No default | Finite | Lower planar bound. |
| `max` | `Vec2` | No default | Finite, should be greater than or equal to `min` | Upper planar bound. |

Bounds clamp the anchor center only:

- 2D clamps `XY`
- 3D clamps `XZ`

They do not currently shrink to keep the full visible frustum inside the region.

## `TopDownCameraTarget`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `priority` | `i32` | `0` | Any integer | Automatic target selection prefers higher priority. |
| `anchor_offset` | `Vec3` | `Vec3::ZERO` | Any finite value | Shifts the tracked point relative to the target transform. Useful for character chest height in 3D. |
| `look_ahead_time` | `Vec2` | `Vec2::ZERO` | Non-negative recommended | Multiplies target planar velocity to produce optional motion look-ahead. |
| `max_look_ahead` | `Vec2` | `Vec2::splat(INFINITY)` | Non-negative recommended | Per-axis clamp for the look-ahead offset. |
| `enabled` | `bool` | `true` | `true` or `false` | Disabled targets are ignored by automatic target selection. |

### Look-Ahead Semantics

Look-ahead is resolved per camera using the camera's current planar frame:

- in 2D it follows world `XY`
- in 3D it follows camera-local right and forward on `XZ`

Because the look-ahead is camera-relative in 3D, rotating the camera changes which world-space directions feed the two look-ahead axes.

The crate smooths the sampled target velocity internally before applying look-ahead so fixed-step or noisy movement produces less camera jitter. `max_look_ahead` should still be set to a finite range for teleport-prone targets.

### Target Selection Rules

- `TopDownCamera.tracked_target = Some(entity)`:
  explicit tracking wins over automatic priority and only requires that the entity has a `Transform`
- `TopDownCamera.tracked_target = None`:
  the camera follows the highest-priority enabled `TopDownCameraTarget`
- equal priorities:
  the lower entity index wins for deterministic behavior
- no valid target:
  the runtime keeps the existing desired anchor instead of inventing a new one

## `TopDownCameraInput`

Optional tuning component for built-in input handling. Requires both
`TopDownCameraInputPlugin` and `TopDownCameraInputPolicy`.

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `keyboard_pan_enabled` | `bool` | `true` | Enable WASD / arrow key panning. |
| `keyboard_pan_speed` | `f32` | `10.0` | Pan speed in world units per second. Scaled by zoom in `Flat2d`. |
| `mouse_drag_enabled` | `bool` | `true` | Enable mouse drag panning. |
| `scroll_zoom_enabled` | `bool` | `true` | Enable scroll wheel zoom. |
| `scroll_zoom_sensitivity` | `f32` | `0.15` | Zoom per scroll line (proportional to current zoom). |
| `zoom_to_cursor` | `bool` | `true` | Zoom toward cursor position instead of screen center. |
| `edge_scroll_enabled` | `bool` | `false` | Enable edge scrolling when cursor is near screen edges. |
| `edge_scroll_margin` | `f32` | `30.0` | Edge zone width in pixels. |
| `edge_scroll_speed` | `f32` | `8.0` | Edge scroll speed in world units per second. |
| `keyboard_rotate_enabled` | `bool` | `true` | Enable Q/E keyboard rotation (`Tilted3d` only). |
| `keyboard_rotate_speed` | `f32` | `1.8` | Rotation speed in radians per second. |
| `keyboard_zoom_enabled` | `bool` | `true` | Enable +/- keyboard zoom. |
| `keyboard_zoom_speed` | `f32` | `2.0` | Keyboard zoom speed in units per second. |

## `TopDownCameraInputPolicy`

Bindable policy component for the built-in controller.

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `target_filter` | `TopDownCameraInputTargetFilter` | `ActiveCamera` | Controls which cameras are allowed to consume built-in input on a frame. |
| `bindings` | `TopDownCameraInputBindingTable` | See below | Keyboard and mouse mapping data for the built-in controller. |

## `TopDownCameraInputTargetFilter`

| Variant | Effect |
| --- | --- |
| `AnyCamera` | Apply input to every camera with `TopDownCameraInput` and `TopDownCameraInputPolicy`. |
| `ActiveCamera` | Apply input only to active cameras. |
| `ActiveViewport` | Apply input only to active cameras whose viewport currently contains the cursor. |

## `TopDownCameraInputBindingTable`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `keyboard_pan_x` | `TopDownCameraKeyAxisBinding` | `A / Left` and `D / Right` | Horizontal keyboard pan axis. |
| `keyboard_pan_y` | `TopDownCameraKeyAxisBinding` | `S / Down` and `W / Up` | Vertical keyboard pan axis. |
| `keyboard_rotate` | `TopDownCameraKeyAxisBinding` | `Q` and `E` | Signed yaw axis. |
| `keyboard_zoom` | `TopDownCameraKeyAxisBinding` | `=` / `NumpadAdd` and `-` / `NumpadSubtract` | Signed keyboard zoom axis. |
| `mouse_drag_buttons` | `Vec<MouseButton>` | `[Middle]` | Any pressed button in the list enables mouse drag panning. |

The crate keeps only neutral defaults for these bindings. Genre-specific presets now live in example code, where `TopDownCameraInput` and `TopDownCameraInputPolicy` are composed explicitly.

## `TopDownCameraKeyAxisBinding`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `negative` | `Vec<KeyCode>` | `[]` | Keys that drive the axis toward `-1.0`. |
| `positive` | `Vec<KeyCode>` | `[]` | Keys that drive the axis toward `1.0`. |

## `TopDownCameraDebug`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `draw_dead_zone` | `bool` | `true` | Draw the dead-zone rectangle on the follow plane. |
| `draw_bounds` | `bool` | `true` | Draw the center-bounds rectangle when bounds are configured. |
| `draw_targets` | `bool` | `true` | Draw the follow anchor, goal anchor, tracked point, and the line between anchor and target. |

## `TopDownCameraEffectLayer`

| Field | Type | Default | Valid Range | Effect |
| --- | --- | --- | --- | --- |
| `anchor_offset` | `Vec3` | `Vec3::ZERO` | Any finite value | World-space offset applied to the follow anchor. |
| `zoom_delta` | `f32` | `0.0` | Any finite value | Additive change to the zoom level. Clamped to `zoom_min..zoom_max` after composition. |
| `yaw_delta` | `f32` | `0.0` | Any finite radians | Additive change to the yaw angle. |
| `fov_delta` | `f32` | `0.0` | Any finite value | FOV offset exposed on `TopDownCameraRuntime::render_fov_delta`. Not auto-applied to the projection; consumers read it and apply it themselves. |
| `weight` | `f32` | `1.0` | Non-negative | Blending weight. Each field is multiplied by `weight` before summing. |
| `enabled` | `bool` | `true` | `true` or `false` | Disabled layers are skipped during composition. |

### Convenience Constructors

- `TopDownCameraEffectLayer::anchor(offset)` — anchor offset only, weight 1.0
- `TopDownCameraEffectLayer::zoom(delta)` — zoom delta only, weight 1.0
- `TopDownCameraEffectLayer::yaw(delta)` — yaw delta only, weight 1.0
- `TopDownCameraEffectLayer::weighted(anchor_offset, zoom_delta, yaw_delta, fov_delta, weight)` — full control

## `TopDownCameraCustomEffects`

Attach to a `TopDownCamera` entity. Multiple named layers compose additively.

| Method | Effect |
| --- | --- |
| `set(name, layer)` | Insert or replace a named layer. |
| `remove(name)` | Remove a named layer. Returns the layer if it existed. |
| `get(name)` / `get_mut(name)` | Access a layer by name. |
| `active_count()` | Count of enabled layers. |
| `iter()` | Iterate all layers. |

The compose system runs in `TopDownCameraSystems::ComposeEffects` and writes the summed result to:

- `TopDownCameraRuntime::render_anchor` — follow anchor + all anchor offsets
- `TopDownCameraRuntime::render_yaw` — yaw + all yaw deltas
- `TopDownCameraRuntime::render_zoom` — zoom + all zoom deltas (clamped)
- `TopDownCameraRuntime::render_fov_delta` — summed FOV delta for consumer use
