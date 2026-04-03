use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCameraPlugin, TopDownCameraSettings, TopDownCameraSystems, TopDownCameraTarget,
};

#[derive(Component)]
struct Basic2dMover;

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((DefaultPlugins, TopDownCameraPlugin::default()));
    common::install_pane(&mut app);
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
        "basic_2d",
        "A single tracked actor moves through a 2D arena.\nThe camera uses a centered dead zone and smooth orthographic follow.",
        Color::srgb(0.95, 0.54, 0.18),
    );

    let target = common::spawn_target_2d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Tracked Target",
        Vec3::new(-220.0, -120.0, 0.4),
        Color::srgb(0.92, 0.26, 0.38),
    );
    commands.entity(target).insert((
        Basic2dMover,
        TopDownCameraTarget {
            priority: 10,
            ..default()
        },
    ));

    let camera_settings = TopDownCameraSettings {
        dead_zone: Vec2::new(180.0, 140.0),
        soft_zone: Vec2::new(280.0, 220.0),
        damping: saddle_camera_top_down_camera::TopDownCameraDamping {
            planar_x: 7.0,
            planar_y: 7.0,
            ..default()
        },
        ..TopDownCameraSettings::flat_2d(999.0)
    };

    common::spawn_camera_2d(
        &mut commands,
        "Top Down Camera",
        Vec3::ZERO,
        1.0,
        camera_settings.clone(),
        false,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleTopDownPane::from_setup(&camera_settings, 1.0, 0.0, true, false),
    );
}

fn animate_target(time: Res<Time>, mut movers: Query<&mut Transform, With<Basic2dMover>>) {
    let Ok(mut transform) = movers.single_mut() else {
        return;
    };

    let t = time.elapsed_secs() * 0.75;
    transform.translation.x = 260.0 * t.cos();
    transform.translation.y = 180.0 * (t * 1.35).sin();
}
