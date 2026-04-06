use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_pane::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCamera, TopDownCameraCustomEffects, TopDownCameraEffectLayer, TopDownCameraPlugin,
    TopDownCameraRuntime, TopDownCameraSettings, TopDownCameraSystems, TopDownCameraTarget,
};
use std::f32::consts::TAU;

/// Settings for the three custom effects, exposed via saddle-pane for live
/// tweaking.
#[derive(Resource, Debug, Clone, Copy, PartialEq, saddle_pane::prelude::Pane)]
#[pane(title = "Custom Effects", position = "bottom-right")]
struct EffectSettings {
    #[pane(toggle)]
    screen_shake_enabled: bool,
    #[pane(slider, min = 0.0, max = 1.5, step = 0.01)]
    screen_shake_intensity: f32,
    #[pane(slider, min = 5.0, max = 40.0, step = 0.5)]
    screen_shake_frequency: f32,

    #[pane(toggle)]
    breathing_enabled: bool,
    #[pane(slider, min = 0.0, max = 0.8, step = 0.01)]
    breathing_amplitude: f32,
    #[pane(slider, min = 0.5, max = 3.0, step = 0.1)]
    breathing_frequency: f32,

    #[pane(toggle)]
    zoom_pulse_enabled: bool,
    #[pane(slider, min = 0.0, max = 4.0, step = 0.1)]
    zoom_pulse_amplitude: f32,
    #[pane(slider, min = 0.3, max = 2.0, step = 0.1)]
    zoom_pulse_frequency: f32,
}

impl Default for EffectSettings {
    fn default() -> Self {
        Self {
            screen_shake_enabled: true,
            screen_shake_intensity: 0.25,
            screen_shake_frequency: 20.0,
            breathing_enabled: true,
            breathing_amplitude: 0.12,
            breathing_frequency: 1.2,
            zoom_pulse_enabled: false,
            zoom_pulse_amplitude: 1.5,
            zoom_pulse_frequency: 0.6,
        }
    }
}

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins,
        TopDownCameraPlugin::default(),
        common::ExampleTopDownCameraControlsPlugin,
    ));
    common::install_pane(&mut app);
    app.register_pane::<EffectSettings>();
    app.insert_resource(common::ExampleTargetCycle::default());
    app.insert_resource(EffectSettings::default());
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            update_screen_shake,
            update_breathing,
            update_zoom_pulse,
            update_effect_overlay,
        )
            .chain()
            .before(TopDownCameraSystems::ComposeEffects),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cycle: ResMut<common::ExampleTargetCycle>,
) {
    common::spawn_reference_world_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "custom_effects",
        "WASD moves the target. Q/E yaws, Z/X zooms.\nThree custom effects compose additively:\n  - Screen shake (high-frequency anchor jitter)\n  - Breathing sway (slow anchor oscillation)\n  - Zoom pulse (periodic zoom change)\nToggle and tune each in the bottom-right pane.",
        Color::srgb(0.96, 0.38, 0.72),
    );

    let target = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Player",
        Vec3::new(0.0, 0.75, 0.0),
        Color::srgb(0.96, 0.38, 0.72),
    );
    commands.entity(target).insert(TopDownCameraTarget {
        priority: 10,
        anchor_offset: Vec3::Y * 0.75,
        look_ahead_time: Vec2::splat(0.12),
        max_look_ahead: Vec2::splat(2.0),
        ..default()
    });

    let camera_settings = TopDownCameraSettings {
        mode: saddle_camera_top_down_camera::TopDownCameraMode::tilted_3d(
            58.0_f32.to_radians(),
            18.0,
        ),
        dead_zone: Vec2::new(2.0, 1.5),
        soft_zone: Vec2::new(4.0, 3.0),
        zoom_min: 8.0,
        zoom_max: 26.0,
        zoom_speed: 1.5,
        ..default()
    };

    let camera = common::spawn_camera_3d_perspective(
        &mut commands,
        "Top Down Camera",
        common::EXAMPLE_3D_ANCHOR,
        0.0,
        16.0,
        camera_settings.clone(),
        true,
    );
    commands.entity(camera).insert((
        TopDownCamera {
            tracked_target: Some(target),
            ..TopDownCamera::looking_at_3d(common::EXAMPLE_3D_ANCHOR, 0.0, 16.0)
        },
        // Attach the custom effects component — effect systems will write to it
        TopDownCameraCustomEffects::default(),
    ));

    common::attach_target_controls(&mut commands, target);
    common::attach_camera_controls(&mut commands, camera);
    common::queue_example_pane(
        &mut commands,
        common::ExampleTopDownPane::from_setup(&camera_settings, 16.0, 0.0, true, true),
    );
    cycle.entities = vec![target];

    // Overlay to show active effects
    commands.spawn((
        Name::new("Effect Overlay"),
        EffectOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            bottom: Val::Px(18.0),
            width: Val::Px(360.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.82)),
        Text::new(String::new()),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

#[derive(Component)]
struct EffectOverlay;

/// Screen shake: high-frequency random-ish anchor jitter using layered sines.
fn update_screen_shake(
    time: Res<Time>,
    settings: Res<EffectSettings>,
    mut query: Query<&mut TopDownCameraCustomEffects, With<TopDownCamera>>,
) {
    for mut custom in &mut query {
        if !settings.screen_shake_enabled {
            custom.remove("screen_shake");
            continue;
        }
        let t = time.elapsed_secs() * settings.screen_shake_frequency * TAU;
        let x = (t * 1.0).sin() * 0.7 + (t * 2.3).sin() * 0.3;
        let z = (t * 0.9).cos() * 0.6 + (t * 1.7).cos() * 0.4;
        let y = (t * 1.4).sin() * 0.2;
        custom.set(
            "screen_shake",
            TopDownCameraEffectLayer::anchor(
                Vec3::new(x, y, z) * settings.screen_shake_intensity,
            ),
        );
    }
}

/// Breathing sway: slow, gentle anchor oscillation.
fn update_breathing(
    time: Res<Time>,
    settings: Res<EffectSettings>,
    mut query: Query<&mut TopDownCameraCustomEffects, With<TopDownCamera>>,
) {
    for mut custom in &mut query {
        if !settings.breathing_enabled {
            custom.remove("breathing");
            continue;
        }
        let t = time.elapsed_secs() * settings.breathing_frequency * TAU;
        let x = t.sin() * settings.breathing_amplitude * 0.4;
        let z = (t * 0.7).cos() * settings.breathing_amplitude * 0.3;
        let y = (t * 0.5).sin() * settings.breathing_amplitude;
        custom.set(
            "breathing",
            TopDownCameraEffectLayer::anchor(Vec3::new(x, y, z)),
        );
    }
}

/// Zoom pulse: periodic zoom change for a "heartbeat" feel.
fn update_zoom_pulse(
    time: Res<Time>,
    settings: Res<EffectSettings>,
    mut query: Query<&mut TopDownCameraCustomEffects, With<TopDownCamera>>,
) {
    for mut custom in &mut query {
        if !settings.zoom_pulse_enabled {
            custom.remove("zoom_pulse");
            continue;
        }
        let t = time.elapsed_secs() * settings.zoom_pulse_frequency * TAU;
        // Double-tap envelope: two quick pulses per cycle
        let phase = t.rem_euclid(TAU) / TAU;
        let envelope = if phase < 0.1 {
            (phase / 0.1 * std::f32::consts::PI).sin()
        } else if phase < 0.25 {
            ((phase - 0.15) / 0.1 * std::f32::consts::PI)
                .sin()
                .max(0.0)
                * 0.6
        } else {
            0.0
        };
        custom.set(
            "zoom_pulse",
            TopDownCameraEffectLayer::zoom(envelope * settings.zoom_pulse_amplitude),
        );
    }
}

fn update_effect_overlay(
    query: Query<(&TopDownCameraCustomEffects, &TopDownCameraRuntime), With<TopDownCamera>>,
    mut overlays: Query<&mut Text, With<EffectOverlay>>,
) {
    let Ok((custom, runtime)) = query.single() else {
        return;
    };
    let Ok(mut text) = overlays.single_mut() else {
        return;
    };

    let mut lines = vec![format!(
        "Active effects: {} / {} layers",
        custom.active_count(),
        custom.layers.len()
    )];
    for named in custom.iter() {
        let l = &named.layer;
        lines.push(format!(
            "  {} — anchor={:.3?} zoom={:.3} yaw={:.3} w={:.1} {}",
            named.name,
            l.anchor_offset,
            l.zoom_delta,
            l.yaw_delta,
            l.weight,
            if l.enabled { "ON" } else { "OFF" },
        ));
    }
    lines.push(format!(
        "render_anchor={:.2?}\nrender_zoom={:.2} render_yaw={:.2}",
        runtime.render_anchor, runtime.render_zoom, runtime.render_yaw
    ));
    *text = Text::new(lines.join("\n"));
}
