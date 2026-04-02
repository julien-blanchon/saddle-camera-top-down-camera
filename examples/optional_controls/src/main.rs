use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_top_down_camera::{TopDownCameraPlugin, TopDownCameraSettings, TopDownCameraTarget};

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins,
        TopDownCameraPlugin::default(),
        common::ExampleTopDownCameraControlsPlugin,
    ));
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
        "optional_controls",
        "WASD moves the active target.\nQ/E yaws the camera, Z/X zooms, and Tab switches targets.\nThis input path lives in example code, not in the shared runtime crate.",
        Color::srgb(0.22, 0.72, 0.96),
    );

    let target_a = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Target A",
        Vec3::new(-4.0, 0.75, 0.0),
        Color::srgb(0.94, 0.22, 0.32),
    );
    commands.entity(target_a).insert(TopDownCameraTarget {
        priority: 10,
        anchor_offset: Vec3::Y * 0.75,
        ..default()
    });

    let target_b = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Target B",
        Vec3::new(5.0, 0.75, 5.0),
        Color::srgb(0.96, 0.74, 0.18),
    );
    commands.entity(target_b).insert(TopDownCameraTarget {
        priority: 5,
        anchor_offset: Vec3::Y * 0.75,
        ..default()
    });

    let camera = common::spawn_camera_3d_perspective(
        &mut commands,
        "Top Down Camera",
        common::EXAMPLE_3D_ANCHOR,
        0.4,
        16.0,
        TopDownCameraSettings {
            mode: saddle_camera_top_down_camera::TopDownCameraMode::tilted_3d(58.0_f32.to_radians(), 18.0),
            dead_zone: Vec2::new(3.6, 2.4),
            bias: Vec2::new(0.0, -0.2),
            zoom_min: 8.0,
            zoom_max: 26.0,
            zoom_speed: 1.5,
            ..default()
        },
        true,
    );

    common::attach_target_controls(&mut commands, target_a);
    common::attach_camera_controls(&mut commands, camera);
    cycle.entities = vec![target_a, target_b];
    commands
        .entity(camera)
        .insert(saddle_camera_top_down_camera::TopDownCamera {
            tracked_target: Some(target_a),
            ..saddle_camera_top_down_camera::TopDownCamera::looking_at_3d(common::EXAMPLE_3D_ANCHOR, 0.4, 16.0)
        });
}
