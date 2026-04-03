use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCameraPlugin, TopDownCameraSettings, TopDownCameraSystems, TopDownCameraTarget,
};

#[derive(Component)]
struct SoftZoneRunner;

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
        "soft_zone_framing",
        "The runner can drift inside a broad soft zone before the camera recenters.\nUse the pane to collapse the soft zone back to the dead zone and feel the framing change live.",
        Color::srgb(0.34, 0.80, 0.98),
    );

    let target = common::spawn_target_2d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Soft Zone Runner",
        Vec3::new(-260.0, 0.0, 0.4),
        Color::srgb(0.96, 0.46, 0.22),
    );
    commands.entity(target).insert((
        SoftZoneRunner,
        TopDownCameraTarget {
            priority: 10,
            ..default()
        },
    ));

    let camera_settings = TopDownCameraSettings {
        dead_zone: Vec2::new(120.0, 90.0),
        soft_zone: Vec2::new(380.0, 250.0),
        damping: saddle_camera_top_down_camera::TopDownCameraDamping {
            planar_x: 5.5,
            planar_y: 5.5,
            ..default()
        },
        ..TopDownCameraSettings::flat_2d(999.0)
    };

    common::spawn_camera_2d(
        &mut commands,
        "Soft Zone Camera",
        Vec3::ZERO,
        1.0,
        camera_settings.clone(),
        true,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleTopDownPane::from_setup(&camera_settings, 1.0, 0.0, true, true),
    );
}

fn animate_target(time: Res<Time>, mut movers: Query<&mut Transform, With<SoftZoneRunner>>) {
    let Ok(mut transform) = movers.single_mut() else {
        return;
    };

    let t = time.elapsed_secs() * 0.58;
    transform.translation.x = 340.0 * (t * 0.9).cos();
    transform.translation.y = 220.0 * t.sin() + 68.0 * (t * 2.3).sin();
}
