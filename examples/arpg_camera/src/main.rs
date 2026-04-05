use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCameraInput, TopDownCameraInputPlugin, TopDownCameraPlugin, TopDownCameraSettings,
    TopDownCameraTarget,
};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins,
        TopDownCameraPlugin::default(),
        TopDownCameraInputPlugin,
        common::ExampleTopDownCameraControlsPlugin,
    ));
    common::install_pane(&mut app);
    app.insert_resource(common::ExampleTargetCycle::default());
    app.add_systems(Startup, setup);
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
        "arpg_camera",
        "ARPG-style camera following a character.\n\
         WASD moves the hero.  Scroll wheel zooms.  Q/E rotates.\n\
         Tab switches between hero and companion.  Camera auto-follows.",
        Color::srgb(0.82, 0.42, 0.92),
    );

    // Main hero character
    let hero = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Hero",
        Vec3::new(0.0, 0.75, 0.0),
        Color::srgb(0.94, 0.22, 0.32),
    );
    commands.entity(hero).insert(TopDownCameraTarget {
        priority: 10,
        anchor_offset: Vec3::Y * 0.75,
        look_ahead_time: Vec2::new(0.15, 0.15),
        max_look_ahead: Vec2::splat(3.0),
        ..default()
    });

    // Companion NPC that wanders nearby
    let companion = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Companion",
        Vec3::new(3.0, 0.75, 2.0),
        Color::srgb(0.26, 0.72, 0.96),
    );
    commands.entity(companion).insert(TopDownCameraTarget {
        priority: 5,
        anchor_offset: Vec3::Y * 0.75,
        look_ahead_time: Vec2::new(0.12, 0.12),
        max_look_ahead: Vec2::splat(2.0),
        ..default()
    });

    let camera_settings = TopDownCameraSettings {
        mode: saddle_camera_top_down_camera::TopDownCameraMode::tilted_3d(
            58.0_f32.to_radians(),
            18.0,
        ),
        dead_zone: Vec2::new(1.5, 1.0),
        soft_zone: Vec2::new(3.5, 2.5),
        bias: Vec2::new(0.0, -0.3),
        zoom_min: 8.0,
        zoom_max: 26.0,
        zoom_speed: 1.8,
        ..default()
    };

    let camera = common::spawn_camera_3d_perspective(
        &mut commands,
        "ARPG Camera",
        common::EXAMPLE_3D_ANCHOR,
        0.3,
        14.0,
        camera_settings.clone(),
        true,
    );

    // Attach ARPG input preset (no keyboard pan, no drag, just zoom + rotate)
    commands.entity(camera).insert((
        TopDownCameraInput::arpg(),
        saddle_camera_top_down_camera::TopDownCamera {
            tracked_target: Some(hero),
            ..saddle_camera_top_down_camera::TopDownCamera::looking_at_3d(
                common::EXAMPLE_3D_ANCHOR,
                0.3,
                14.0,
            )
        },
    ));

    common::attach_target_controls(&mut commands, hero);
    common::attach_camera_controls(&mut commands, camera);
    common::queue_example_pane(
        &mut commands,
        common::ExampleTopDownPane::from_setup(&camera_settings, 14.0, 0.3, true, true),
    );
    cycle.entities = vec![hero, companion];
}
