use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCameraPlugin, TopDownCameraSettings, TopDownCameraSystems, TopDownCameraTarget,
};

#[derive(Component)]
struct Basic3dMover;

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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    common::spawn_reference_world_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "basic_3d",
        "A perspective `Camera3d` follows a moving hero.\nYaw is manual, zoom is arm distance, and the dead zone is expressed in planar world units.",
        Color::srgb(0.26, 0.74, 0.96),
    );

    let target = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Tracked Hero",
        Vec3::new(8.0, 0.75, -6.0),
        Color::srgb(0.94, 0.22, 0.34),
    );
    commands.entity(target).insert((
        Basic3dMover,
        TopDownCameraTarget {
            priority: 10,
            anchor_offset: Vec3::Y * 0.75,
            look_ahead_time: Vec2::splat(0.15),
            max_look_ahead: Vec2::splat(1.2),
            ..default()
        },
    ));

    let camera_settings = TopDownCameraSettings {
        mode: saddle_camera_top_down_camera::TopDownCameraMode::tilted_3d(
            60.0_f32.to_radians(),
            18.0,
        ),
        dead_zone: Vec2::new(3.8, 2.6),
        soft_zone: Vec2::new(6.2, 4.2),
        damping: saddle_camera_top_down_camera::TopDownCameraDamping {
            planar_x: 8.0,
            planar_y: 10.0,
            height: 12.0,
            zoom: 8.0,
            yaw: 10.0,
        },
        zoom_min: 8.0,
        zoom_max: 30.0,
        zoom_speed: 1.5,
        ..default()
    };

    common::spawn_camera_3d_perspective(
        &mut commands,
        "Top Down Camera",
        common::EXAMPLE_3D_ANCHOR,
        0.3,
        18.0,
        camera_settings.clone(),
        false,
    );
    common::queue_example_pane(
        &mut commands,
        common::ExampleTopDownPane::from_setup(&camera_settings, 18.0, 0.3, true, false),
    );
}

fn animate_target(time: Res<Time>, mut movers: Query<&mut Transform, With<Basic3dMover>>) {
    let Ok(mut transform) = movers.single_mut() else {
        return;
    };

    let t = time.elapsed_secs() * 0.6;
    transform.translation.x = 9.0 * t.cos();
    transform.translation.z = 7.0 * t.sin();
    transform.translation.y = 0.75;
}
