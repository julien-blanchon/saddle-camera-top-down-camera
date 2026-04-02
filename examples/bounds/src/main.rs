use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCameraBounds, TopDownCameraDebug, TopDownCameraPlugin, TopDownCameraSettings,
    TopDownCameraSystems, TopDownCameraTarget,
};

#[derive(Component)]
struct BoundsMover;

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((DefaultPlugins, TopDownCameraPlugin::default()));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        animate_target.before(TopDownCameraSystems::ResolveTarget),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    common::spawn_reference_world_2d(
        &mut commands,
        "bounds",
        "The target keeps trying to leave the arena.\nThe camera dead-zone solve still runs, but the resulting anchor is clamped to center bounds.",
        Color::srgb(0.92, 0.72, 0.24),
    );

    let target = common::spawn_target_2d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Bounds Target",
        Vec3::new(-420.0, -260.0, 0.4),
        Color::srgb(0.98, 0.26, 0.46),
    );
    commands.entity(target).insert((
        BoundsMover,
        TopDownCameraTarget {
            priority: 10,
            ..default()
        },
    ));

    let camera = common::spawn_camera_2d(
        &mut commands,
        "Top Down Camera",
        Vec3::ZERO,
        1.0,
        TopDownCameraSettings {
            dead_zone: Vec2::new(140.0, 120.0),
            bounds: Some(TopDownCameraBounds {
                min: Vec2::new(-300.0, -220.0),
                max: Vec2::new(300.0, 220.0),
            }),
            damping: saddle_camera_top_down_camera::TopDownCameraDamping {
                planar_x: 7.0,
                planar_y: 7.0,
                ..default()
            },
            ..TopDownCameraSettings::flat_2d(999.0)
        },
        true,
    );
    commands
        .entity(camera)
        .insert(TopDownCameraDebug::default());
}

fn animate_target(time: Res<Time>, mut movers: Query<&mut Transform, With<BoundsMover>>) {
    let Ok(mut transform) = movers.single_mut() else {
        return;
    };

    let t = time.elapsed_secs() * 0.5;
    transform.translation.x = 420.0 * (t.cos() * 1.2);
    transform.translation.y = 300.0 * (t.sin() * 1.15);
}
