use super::*;

#[test]
fn stack_ignores_disabled_layers() {
    let mut stack = TopDownCameraEffectStack::default();
    stack.add_layer(&TopDownCameraEffectLayer {
        anchor_offset: Vec3::X,
        enabled: false,
        ..default()
    });
    assert!(stack.is_zero());
}

#[test]
fn stack_ignores_zero_weight_layers() {
    let mut stack = TopDownCameraEffectStack::default();
    stack.add_layer(&TopDownCameraEffectLayer {
        anchor_offset: Vec3::X,
        weight: 0.0,
        ..default()
    });
    assert!(stack.is_zero());
}

#[test]
fn stack_composes_additively() {
    let mut stack = TopDownCameraEffectStack::default();
    stack.add_layer(&TopDownCameraEffectLayer::anchor(Vec3::new(1.0, 0.0, 0.0)));
    stack.add_layer(&TopDownCameraEffectLayer::anchor(Vec3::new(0.0, 2.0, 0.0)));
    assert_eq!(stack.anchor_offset, Vec3::new(1.0, 2.0, 0.0));
}

#[test]
fn stack_applies_weight() {
    let mut stack = TopDownCameraEffectStack::default();
    stack.add_layer(&TopDownCameraEffectLayer::weighted(
        Vec3::new(4.0, 0.0, 0.0),
        0.0,
        0.0,
        0.0,
        0.5,
    ));
    assert!((stack.anchor_offset.x - 2.0).abs() < f32::EPSILON);
}

#[test]
fn custom_effects_set_and_get() {
    let mut custom = TopDownCameraCustomEffects::default();
    custom.set("shake", TopDownCameraEffectLayer::anchor(Vec3::X));
    assert!(custom.get("shake").is_some());
    assert_eq!(custom.active_count(), 1);
}

#[test]
fn custom_effects_replace_by_name() {
    let mut custom = TopDownCameraCustomEffects::default();
    custom.set("shake", TopDownCameraEffectLayer::anchor(Vec3::X));
    custom.set("shake", TopDownCameraEffectLayer::anchor(Vec3::Y));
    assert_eq!(custom.layers.len(), 1);
    assert_eq!(custom.get("shake").unwrap().anchor_offset, Vec3::Y);
}

#[test]
fn custom_effects_remove() {
    let mut custom = TopDownCameraCustomEffects::default();
    custom.set("shake", TopDownCameraEffectLayer::anchor(Vec3::X));
    let removed = custom.remove("shake");
    assert!(removed.is_some());
    assert_eq!(custom.layers.len(), 0);
    assert!(custom.remove("shake").is_none());
}
