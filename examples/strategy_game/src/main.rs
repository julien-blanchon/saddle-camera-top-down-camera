use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCameraBounds, TopDownCameraInputPlugin, TopDownCameraPlugin, TopDownCameraSettings,
    TopDownCameraTarget,
};

/// Marker for the automated patrol unit.
#[derive(Component)]
struct PatrolUnit;

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins,
        TopDownCameraPlugin::default(),
        TopDownCameraInputPlugin::default(),
    ));
    common::install_pane(&mut app);
    app.add_systems(Startup, setup);
    app.add_systems(Update, animate_patrol);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_reference_world_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "strategy_game",
        "Strategy/RTS camera with edge scrolling, zoom-to-cursor, and map bounds.\n\
         Move cursor to screen edges to scroll.  Scroll wheel zooms toward cursor.\n\
         WASD pans, Q/E rotates, +/- zooms via keyboard.",
        Color::srgb(0.22, 0.82, 0.44),
    );

    // Spawn a patrol unit that wanders the map
    let patrol = commands
        .spawn((
            Name::new("Patrol Unit"),
            PatrolUnit,
            Mesh3d(meshes.add(Capsule3d::new(0.35, 0.8).mesh().rings(8).latitudes(10))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.94, 0.28, 0.36),
                perceptual_roughness: 0.3,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.75, 0.0),
            TopDownCameraTarget {
                priority: 1,
                anchor_offset: Vec3::Y * 0.75,
                ..default()
            },
        ))
        .id();

    // Spawn a few static units for visual reference
    for (i, (x, z)) in [(-6.0, -4.0), (8.0, 2.0), (-3.0, 7.0), (5.0, -6.0)]
        .iter()
        .enumerate()
    {
        commands.spawn((
            Name::new(format!("Static Unit {}", i)),
            Mesh3d(meshes.add(Capsule3d::new(0.3, 0.6).mesh().rings(6).latitudes(8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.5, 0.85),
                perceptual_roughness: 0.4,
                ..default()
            })),
            Transform::from_xyz(*x, 0.55, *z),
        ));
    }

    let camera_settings = TopDownCameraSettings {
        mode: saddle_camera_top_down_camera::TopDownCameraMode::tilted_3d(
            55.0_f32.to_radians(),
            20.0,
        ),
        dead_zone: Vec2::new(2.0, 2.0),
        soft_zone: Vec2::new(4.0, 3.0),
        bounds: Some(TopDownCameraBounds {
            min: Vec2::new(-20.0, -20.0),
            max: Vec2::new(20.0, 20.0),
        }),
        bounds_soft_margin: 2.0,
        zoom_min: 6.0,
        zoom_max: 30.0,
        zoom_speed: 2.0,
        ..default()
    };

    let camera = common::spawn_camera_3d_perspective(
        &mut commands,
        "Strategy Camera",
        common::EXAMPLE_3D_ANCHOR,
        0.0,
        18.0,
        camera_settings.clone(),
        true,
    );

    let (input, policy) = common::strategy_camera_input();

    commands.entity(camera).insert((
        input,
        policy,
        saddle_camera_top_down_camera::TopDownCamera {
            tracked_target: Some(patrol),
            follow_enabled: false, // Strategy cameras don't auto-follow
            ..saddle_camera_top_down_camera::TopDownCamera::looking_at_3d(
                common::EXAMPLE_3D_ANCHOR,
                0.0,
                18.0,
            )
        },
    ));

    common::queue_example_pane(
        &mut commands,
        common::ExampleTopDownPane::from_setup(&camera_settings, 18.0, 0.0, false, true),
    );
}

fn animate_patrol(time: Res<Time>, mut units: Query<&mut Transform, With<PatrolUnit>>) {
    let t = time.elapsed_secs() * 0.35;
    for mut transform in &mut units {
        transform.translation.x = 8.0 * t.cos();
        transform.translation.z = 6.0 * (t * 1.3).sin();
    }
}
