use saddle_camera_top_down_camera_example_common as common;

use bevy::prelude::*;
use saddle_camera_top_down_camera::{
    TopDownCamera, TopDownCameraPlugin, TopDownCameraSettings, TopDownCameraSystems,
    TopDownCameraTarget,
};

#[derive(Component)]
struct LeftTarget;

#[derive(Component)]
struct RightTarget;

#[derive(Resource)]
struct CameraEntity(Entity);

#[derive(Resource)]
struct SwitchTimer(Timer);

#[derive(Resource)]
struct TargetPair {
    left: Entity,
    right: Entity,
}

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((DefaultPlugins, TopDownCameraPlugin::default()));
    app.insert_resource(SwitchTimer(Timer::from_seconds(2.2, TimerMode::Repeating)));
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            animate_targets.before(TopDownCameraSystems::ResolveTarget),
            switch_targets,
        ),
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
        "target_switching",
        "Two heroes move on different loops.\nThe camera swaps its explicit tracked target every few seconds without changing any runtime systems.",
        Color::srgb(0.84, 0.60, 0.20),
    );

    let left = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Left Hero",
        Vec3::new(-8.0, 0.75, -4.0),
        Color::srgb(0.94, 0.28, 0.42),
    );
    commands.entity(left).insert((
        LeftTarget,
        TopDownCameraTarget {
            anchor_offset: Vec3::Y * 0.75,
            ..default()
        },
    ));

    let right = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Right Hero",
        Vec3::new(8.0, 0.75, 4.0),
        Color::srgb(0.20, 0.76, 0.96),
    );
    commands.entity(right).insert((
        RightTarget,
        TopDownCameraTarget {
            anchor_offset: Vec3::Y * 0.75,
            ..default()
        },
    ));

    let camera = common::spawn_camera_3d_perspective(
        &mut commands,
        "Top Down Camera",
        common::EXAMPLE_3D_ANCHOR,
        0.4,
        16.0,
        TopDownCameraSettings {
            mode: saddle_camera_top_down_camera::TopDownCameraMode::tilted_3d(58.0_f32.to_radians(), 16.0),
            dead_zone: Vec2::new(3.4, 2.2),
            zoom_min: 10.0,
            zoom_max: 24.0,
            zoom_speed: 1.2,
            ..default()
        },
        true,
    );
    commands.insert_resource(CameraEntity(camera));
    commands.insert_resource(TargetPair { left, right });

    commands.entity(camera).insert(TopDownCamera {
        tracked_target: Some(left),
        ..TopDownCamera::looking_at_3d(common::EXAMPLE_3D_ANCHOR, 0.4, 16.0)
    });
}

fn animate_targets(
    time: Res<Time>,
    mut left: Query<&mut Transform, (With<LeftTarget>, Without<RightTarget>)>,
    mut right: Query<&mut Transform, (With<RightTarget>, Without<LeftTarget>)>,
) {
    let t = time.elapsed_secs() * 0.7;

    if let Ok(mut transform) = left.single_mut() {
        transform.translation.x = -8.0 + 4.0 * t.cos();
        transform.translation.z = -2.0 + 3.0 * (t * 1.2).sin();
    }

    if let Ok(mut transform) = right.single_mut() {
        transform.translation.x = 8.0 + 5.0 * (t * 0.9).sin();
        transform.translation.z = 4.0 + 4.0 * t.cos();
    }
}

fn switch_targets(
    time: Res<Time>,
    pair: Res<TargetPair>,
    camera_entity: Res<CameraEntity>,
    mut timer: ResMut<SwitchTimer>,
    mut cameras: Query<&mut TopDownCamera>,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let Ok(mut camera) = cameras.get_mut(camera_entity.0) else {
        return;
    };
    camera.tracked_target = Some(match camera.tracked_target {
        Some(current) if current == pair.left => pair.right,
        _ => pair.left,
    });
}
