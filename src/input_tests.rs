use super::*;

#[test]
fn default_input_enables_expected_features() {
    let input = TopDownCameraInput::default();
    assert!(input.keyboard_pan_enabled);
    assert!(input.mouse_drag_enabled);
    assert!(input.scroll_zoom_enabled);
    assert!(input.zoom_to_cursor);
    assert!(!input.edge_scroll_enabled);
    assert!(input.keyboard_rotate_enabled);
    assert!(input.keyboard_zoom_enabled);
    assert_eq!(input.mouse_drag_button, MouseButton::Middle);
}

#[test]
fn strategy_preset_enables_edge_scroll() {
    let input = TopDownCameraInput::strategy();
    assert!(input.edge_scroll_enabled);
    assert!(input.keyboard_pan_enabled);
    assert!(input.zoom_to_cursor);
}

#[test]
fn arpg_preset_disables_manual_pan() {
    let input = TopDownCameraInput::arpg();
    assert!(!input.keyboard_pan_enabled);
    assert!(!input.mouse_drag_enabled);
    assert!(!input.edge_scroll_enabled);
    assert!(input.scroll_zoom_enabled);
}
