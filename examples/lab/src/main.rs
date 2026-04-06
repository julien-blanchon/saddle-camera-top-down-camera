use saddle_camera_top_down_camera_example_common as common;
#[cfg(feature = "e2e")]
mod e2e;

use bevy::prelude::*;
#[cfg(all(feature = "brp", not(target_arch = "wasm32")))]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
#[cfg(all(feature = "brp", not(target_arch = "wasm32")))]
use bevy_brp_extras::BrpExtrasPlugin;
use saddle_camera_top_down_camera::{
    TopDownCamera, TopDownCameraPlugin, TopDownCameraRuntime, TopDownCameraSettings,
    TopDownCameraSystems, TopDownCameraTarget,
};

#[derive(Component)]
struct PrimaryTarget;

#[derive(Component)]
struct SecondaryTarget;

#[derive(Component)]
struct LabOverlay;

/// When this resource is present, `enforce_authored_lab_defaults` is skipped.
/// E2E scenarios insert this when they reconfigure camera settings.
#[derive(Resource)]
pub struct LabDefaultsOverridden;

#[derive(Resource, Clone, Copy)]
pub struct LabCameraEntity(pub Entity);

#[derive(Resource, Clone, Copy)]
pub struct LabPrimaryTargetEntity(pub Entity);

#[derive(Resource, Clone, Copy)]
pub struct LabSecondaryTargetEntity(pub Entity);

const LAB_BOUNDS_MIN: Vec2 = Vec2::new(-12.0, -12.0);
const LAB_BOUNDS_MAX: Vec2 = Vec2::new(12.0, 12.0);

fn main() {
    let mut app = App::new();
    common::apply_example_defaults(&mut app);
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "saddle_camera_top_down_camera_lab".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }),
        TopDownCameraPlugin::default(),
        common::ExampleTopDownCameraControlsPlugin,
    ));
    common::install_pane(&mut app);
    #[cfg(all(feature = "brp", not(target_arch = "wasm32")))]
    app.add_plugins((
        RemotePlugin::default(),
        BrpExtrasPlugin::with_http_plugin(RemoteHttpPlugin::default()),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::TopDownCameraLabE2EPlugin);

    app.insert_resource(common::ExampleTargetCycle::default());
    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            animate_secondary_target.before(TopDownCameraSystems::ResolveTarget),
            update_overlay.after(TopDownCameraSystems::ApplySmoothing),
        ),
    );
    app.run();
}

fn setup(
    mut commands: Commands,
    mut cycle: ResMut<common::ExampleTargetCycle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bounds = saddle_camera_top_down_camera::TopDownCameraBounds {
        min: LAB_BOUNDS_MIN,
        max: LAB_BOUNDS_MAX,
    };

    common::spawn_reference_world_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "saddle_camera_top_down_camera_lab",
        "WASD moves the active target. Q/E yaws. Z/X zooms. Tab switches tracked target.\nThe lab uses an orthographic Camera3d to verify 3D framing plus projection-scale zoom.",
        Color::srgb(0.94, 0.62, 0.20),
    );
    spawn_bounds_markers(&mut commands, &mut meshes, &mut materials, bounds);

    let primary = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Primary Target",
        Vec3::new(-4.0, 0.75, 0.0),
        Color::srgb(0.94, 0.22, 0.34),
    );
    commands.entity(primary).insert((
        PrimaryTarget,
        TopDownCameraTarget {
            priority: 10,
            anchor_offset: Vec3::Y * 0.75,
            look_ahead_time: Vec2::splat(0.18),
            max_look_ahead: Vec2::splat(1.4),
            ..default()
        },
    ));
    common::attach_target_controls(&mut commands, primary);

    let secondary = common::spawn_target_3d(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Secondary Target",
        Vec3::new(7.0, 0.75, 7.0),
        Color::srgb(0.24, 0.76, 0.98),
    );
    commands.entity(secondary).insert((
        SecondaryTarget,
        TopDownCameraTarget {
            priority: 5,
            anchor_offset: Vec3::Y * 0.75,
            ..default()
        },
    ));

    let camera_settings = TopDownCameraSettings {
        mode: saddle_camera_top_down_camera::TopDownCameraMode::tilted_3d(
            58.0_f32.to_radians(),
            22.0,
        ),
        dead_zone: Vec2::new(3.4, 2.3),
        soft_zone: Vec2::new(5.8, 3.9),
        bias: Vec2::new(0.0, -0.2),
        bounds: Some(bounds),
        zoom_min: 0.65,
        zoom_max: 1.9,
        zoom_speed: 0.75,
        ..default()
    };

    let camera = common::spawn_camera_3d_orthographic(
        &mut commands,
        "Top Down Camera",
        common::EXAMPLE_3D_ANCHOR,
        0.45,
        1.0,
        camera_settings.clone(),
        true,
    );
    common::attach_camera_controls(&mut commands, camera);
    common::queue_example_pane(
        &mut commands,
        common::ExampleTopDownPane::from_setup(&camera_settings, 1.0, 0.45, true, true),
    );

    commands.entity(camera).insert(TopDownCamera {
        tracked_target: Some(primary),
        ..TopDownCamera::looking_at_3d(common::EXAMPLE_3D_ANCHOR, 0.45, 1.0)
    });

    cycle.entities = vec![primary, secondary];
    commands.insert_resource(LabCameraEntity(camera));
    commands.insert_resource(LabPrimaryTargetEntity(primary));
    commands.insert_resource(LabSecondaryTargetEntity(secondary));

    commands.spawn((
        Name::new("Lab Overlay"),
        LabOverlay,
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.82)),
        Text::new(String::new()),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn animate_secondary_target(
    time: Res<Time>,
    mut targets: Query<&mut Transform, With<SecondaryTarget>>,
) {
    let Ok(mut transform) = targets.single_mut() else {
        return;
    };

    let t = time.elapsed_secs() * 0.55;
    transform.translation.x = 7.0 + 4.5 * t.cos();
    transform.translation.z = 7.0 + 3.0 * (t * 1.3).sin();
    transform.translation.y = 0.75;
}

fn spawn_bounds_markers(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    bounds: saddle_camera_top_down_camera::TopDownCameraBounds,
) {
    let frame_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.62, 0.20),
        emissive: LinearRgba::from(Color::srgb(0.20, 0.10, 0.02)),
        perceptual_roughness: 0.92,
        ..default()
    });
    let corner_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.82, 0.96),
        emissive: LinearRgba::from(Color::srgb(0.06, 0.14, 0.18)),
        perceptual_roughness: 0.4,
        ..default()
    });
    let wall_thickness = 0.35;
    let wall_height = 0.4;
    let size = bounds.max - bounds.min;
    let center = (bounds.min + bounds.max) * 0.5;

    commands.spawn((
        Name::new("Bounds Floor Frame"),
        Mesh3d(meshes.add(Cuboid::new(size.x, 0.08, size.y))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.94, 0.64, 0.22, 0.10),
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            ..default()
        })),
        Transform::from_xyz(center.x, 0.02, center.y),
    ));

    commands.spawn((
        Name::new("Bounds North Wall"),
        Mesh3d(meshes.add(Cuboid::new(
            size.x + wall_thickness,
            wall_height,
            wall_thickness,
        ))),
        MeshMaterial3d(frame_material.clone()),
        Transform::from_xyz(center.x, wall_height * 0.5, bounds.max.y),
    ));
    commands.spawn((
        Name::new("Bounds South Wall"),
        Mesh3d(meshes.add(Cuboid::new(
            size.x + wall_thickness,
            wall_height,
            wall_thickness,
        ))),
        MeshMaterial3d(frame_material.clone()),
        Transform::from_xyz(center.x, wall_height * 0.5, bounds.min.y),
    ));
    commands.spawn((
        Name::new("Bounds East Wall"),
        Mesh3d(meshes.add(Cuboid::new(
            wall_thickness,
            wall_height,
            size.y + wall_thickness,
        ))),
        MeshMaterial3d(frame_material.clone()),
        Transform::from_xyz(bounds.max.x, wall_height * 0.5, center.y),
    ));
    commands.spawn((
        Name::new("Bounds West Wall"),
        Mesh3d(meshes.add(Cuboid::new(
            wall_thickness,
            wall_height,
            size.y + wall_thickness,
        ))),
        MeshMaterial3d(frame_material.clone()),
        Transform::from_xyz(bounds.min.x, wall_height * 0.5, center.y),
    ));

    for (index, corner) in [
        Vec3::new(bounds.min.x, 1.2, bounds.min.y),
        Vec3::new(bounds.min.x, 1.2, bounds.max.y),
        Vec3::new(bounds.max.x, 1.2, bounds.min.y),
        Vec3::new(bounds.max.x, 1.2, bounds.max.y),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Bounds Corner {}", index + 1)),
            Mesh3d(meshes.add(Cylinder::new(0.35, 2.4))),
            MeshMaterial3d(corner_material.clone()),
            Transform::from_translation(corner),
        ));
    }
}

fn update_overlay(
    camera_entity: Res<LabCameraEntity>,
    primary_entity: Res<LabPrimaryTargetEntity>,
    secondary_entity: Res<LabSecondaryTargetEntity>,
    cameras: Query<(&TopDownCamera, &TopDownCameraRuntime, Option<&Projection>)>,
    names: Query<&Name>,
    targets: Query<&Transform>,
    mut overlays: Query<&mut Text, With<LabOverlay>>,
) {
    let Ok((camera, runtime, projection)) = cameras.get(camera_entity.0) else {
        return;
    };
    let Ok(primary) = targets.get(primary_entity.0) else {
        return;
    };
    let Ok(secondary) = targets.get(secondary_entity.0) else {
        return;
    };
    let Ok(mut text) = overlays.single_mut() else {
        return;
    };
    let active_target_name = runtime
        .active_target
        .and_then(|entity| names.get(entity).ok())
        .map(|name| name.as_str().to_owned())
        .unwrap_or_else(|| "None".to_owned());
    let projection_label = match projection {
        Some(Projection::Orthographic(orthographic)) => {
            format!("ortho scale {:.2}", orthographic.scale)
        }
        Some(Projection::Perspective(_)) => "perspective".to_owned(),
        Some(Projection::Custom(_)) => "custom".to_owned(),
        None => "n/a".to_owned(),
    };

    *text = Text::new(format!(
        "Top Down Camera Lab\nactive target {active_target_name}\nprojection {projection_label}\ntracked point {:.2?}\nfollow anchor {:.2?}\ngoal anchor {:.2?}\nyaw {:.2}\nzoom {:.2}\nprimary {:.2?}\nsecondary {:.2?}",
        runtime.tracked_point,
        runtime.follow_anchor,
        runtime.goal_anchor,
        runtime.yaw,
        runtime.zoom,
        primary.translation,
        secondary.translation,
    ));
    let _ = camera;
}
