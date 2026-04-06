use bevy::prelude::*;

use crate::{TopDownCamera, TopDownCameraRuntime, TopDownCameraSettings};

/// A single additive effect layer for the top-down camera.
///
/// Effect layers compose additively — each layer's values are multiplied by its
/// weight and summed together. This lets multiple independent systems (screen
/// shake, breathing, hit flinch, etc.) contribute without knowing about each
/// other.
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct TopDownCameraEffectLayer {
    /// World-space offset applied to the follow anchor.
    pub anchor_offset: Vec3,
    /// Additive change to the zoom level.
    pub zoom_delta: f32,
    /// Additive change to the yaw angle (radians).
    pub yaw_delta: f32,
    /// Additive change to the field of view (perspective cameras only).
    pub fov_delta: f32,
    /// Blending weight. 0.0 = no contribution, 1.0 = full contribution.
    pub weight: f32,
    /// When false the layer is skipped during composition.
    pub enabled: bool,
}

impl TopDownCameraEffectLayer {
    /// Create a layer with explicit weight.
    pub fn weighted(
        anchor_offset: Vec3,
        zoom_delta: f32,
        yaw_delta: f32,
        fov_delta: f32,
        weight: f32,
    ) -> Self {
        Self {
            anchor_offset,
            zoom_delta,
            yaw_delta,
            fov_delta,
            weight,
            enabled: true,
        }
    }

    /// Create a layer with only an anchor offset (weight 1.0).
    pub fn anchor(offset: Vec3) -> Self {
        Self {
            anchor_offset: offset,
            ..default()
        }
    }

    /// Create a layer with only a zoom delta (weight 1.0).
    pub fn zoom(delta: f32) -> Self {
        Self {
            zoom_delta: delta,
            ..default()
        }
    }

    /// Create a layer with only a yaw delta (weight 1.0).
    pub fn yaw(delta: f32) -> Self {
        Self {
            yaw_delta: delta,
            ..default()
        }
    }
}

impl Default for TopDownCameraEffectLayer {
    fn default() -> Self {
        Self {
            anchor_offset: Vec3::ZERO,
            zoom_delta: 0.0,
            yaw_delta: 0.0,
            fov_delta: 0.0,
            weight: 1.0,
            enabled: true,
        }
    }
}

/// The composed result of all active effect layers.
#[derive(Reflect, Debug, Clone, Default, PartialEq)]
pub struct TopDownCameraEffectStack {
    pub anchor_offset: Vec3,
    pub zoom_delta: f32,
    pub yaw_delta: f32,
    pub fov_delta: f32,
}

impl TopDownCameraEffectStack {
    pub fn add_layer(&mut self, layer: &TopDownCameraEffectLayer) {
        if !layer.enabled || layer.weight <= 0.0 {
            return;
        }
        self.anchor_offset += layer.anchor_offset * layer.weight;
        self.zoom_delta += layer.zoom_delta * layer.weight;
        self.yaw_delta += layer.yaw_delta * layer.weight;
        self.fov_delta += layer.fov_delta * layer.weight;
    }

    pub fn is_zero(&self) -> bool {
        self.anchor_offset == Vec3::ZERO
            && self.zoom_delta == 0.0
            && self.yaw_delta == 0.0
            && self.fov_delta == 0.0
    }
}

/// A named effect layer for user-driven custom effects.
#[derive(Reflect, Debug, Clone)]
pub struct NamedEffectLayer {
    pub name: String,
    pub layer: TopDownCameraEffectLayer,
}

/// Multiple named custom effect layers that compose additively with the
/// camera's follow state. Attach this component to your `TopDownCamera`
/// entity and write systems that call [`set`](TopDownCameraCustomEffects::set)
/// to push custom camera motion (screen shake, breathing sway, etc.).
///
/// ```rust,ignore
/// fn update_screen_shake(
///     time: Res<Time>,
///     mut q: Query<&mut TopDownCameraCustomEffects, With<TopDownCamera>>,
/// ) {
///     for mut custom in &mut q {
///         let t = time.elapsed_secs();
///         custom.set("shake", TopDownCameraEffectLayer::anchor(
///             Vec3::new((t * 30.0).sin() * 0.1, 0.0, (t * 25.0).cos() * 0.1),
///         ));
///     }
/// }
/// ```
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct TopDownCameraCustomEffects {
    pub layers: Vec<NamedEffectLayer>,
}

impl TopDownCameraCustomEffects {
    /// Insert or replace a named layer.
    pub fn set(&mut self, name: impl Into<String>, layer: TopDownCameraEffectLayer) {
        let name = name.into();
        if let Some(existing) = self.layers.iter_mut().find(|l| l.name == name) {
            existing.layer = layer;
        } else {
            self.layers.push(NamedEffectLayer { name, layer });
        }
    }

    /// Remove a named layer, returning it if it existed.
    pub fn remove(&mut self, name: &str) -> Option<TopDownCameraEffectLayer> {
        if let Some(pos) = self.layers.iter().position(|l| l.name == name) {
            Some(self.layers.swap_remove(pos).layer)
        } else {
            None
        }
    }

    /// Get an immutable reference to a named layer.
    pub fn get(&self, name: &str) -> Option<&TopDownCameraEffectLayer> {
        self.layers
            .iter()
            .find(|l| l.name == name)
            .map(|l| &l.layer)
    }

    /// Get a mutable reference to a named layer.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut TopDownCameraEffectLayer> {
        self.layers
            .iter_mut()
            .find(|l| l.name == name)
            .map(|l| &mut l.layer)
    }

    /// Number of active (enabled) layers.
    pub fn active_count(&self) -> usize {
        self.layers.iter().filter(|l| l.layer.enabled).count()
    }

    /// Iterate over all layers.
    pub fn iter(&self) -> impl Iterator<Item = &NamedEffectLayer> {
        self.layers.iter()
    }
}

/// System that composes all custom effect layers into the runtime's render
/// fields. Runs after smoothing, before transform sync.
pub(crate) fn compose_effects(
    mut cameras: Query<(
        &TopDownCamera,
        &TopDownCameraSettings,
        &mut TopDownCameraRuntime,
        Option<&TopDownCameraCustomEffects>,
    )>,
) {
    for (_camera, settings, mut runtime, custom_effects) in &mut cameras {
        let mut stack = TopDownCameraEffectStack::default();

        if let Some(custom) = custom_effects {
            for named in custom.iter() {
                stack.add_layer(&named.layer);
            }
        }

        runtime.render_anchor = runtime.follow_anchor + stack.anchor_offset;
        runtime.render_yaw = runtime.yaw + stack.yaw_delta;
        runtime.render_zoom = (runtime.zoom + stack.zoom_delta).clamp(
            settings.zoom_min.min(settings.zoom_max),
            settings.zoom_max.max(settings.zoom_min),
        );
        runtime.render_fov_delta = stack.fov_delta;
    }
}

#[cfg(test)]
#[path = "effects_tests.rs"]
mod tests;
