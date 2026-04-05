use std::time::Duration;

use bevy::{app::AppExit, camera::ScalingMode, light::GlobalAmbientLight, prelude::*};
use bevy_enhanced_input::context::InputContextAppExt;
use bevy_enhanced_input::prelude::{
    Action, Bidirectional, Bindings, Cardinal, EnhancedInputPlugin, Fire, InputAction, Start,
    actions, bindings,
};
use bevy_flair::prelude::InlineStyle;
use saddle_camera_top_down_camera::{
    TopDownCamera, TopDownCameraDebug, TopDownCameraMode, TopDownCameraSettings,
};
use saddle_pane::prelude::*;

const PANE_DARK_THEME_VARS: &[(&str, &str)] = &[
    ("--pane-elevation-1", "#28292e"),
    ("--pane-elevation-2", "#222327"),
    ("--pane-elevation-3", "rgba(187, 188, 196, 0.10)"),
    ("--pane-border", "#3c3d44"),
    ("--pane-border-focus", "#7090b0"),
    ("--pane-border-subtle", "#333438"),
    ("--pane-text-primary", "#bbbcc4"),
    ("--pane-text-secondary", "#78797f"),
    ("--pane-text-muted", "#5c5d64"),
    ("--pane-text-on-accent", "#ffffff"),
    ("--pane-text-brighter", "#d0d1d8"),
    ("--pane-text-monitor", "#9a9ba2"),
    ("--pane-text-log", "#8a8b92"),
    ("--pane-accent", "#4a6fa5"),
    ("--pane-accent-hover", "#5a8fd5"),
    ("--pane-accent-active", "#3a5f95"),
    ("--pane-accent-subtle", "rgba(74, 111, 165, 0.15)"),
    ("--pane-accent-fill", "rgba(74, 111, 165, 0.60)"),
    ("--pane-accent-fill-hover", "rgba(90, 143, 213, 0.70)"),
    ("--pane-accent-fill-active", "rgba(90, 143, 213, 0.80)"),
    ("--pane-accent-checked", "rgba(74, 111, 165, 0.25)"),
    ("--pane-accent-checked-hover", "rgba(74, 111, 165, 0.35)"),
    ("--pane-accent-indicator", "rgba(74, 111, 165, 0.80)"),
    ("--pane-accent-knob", "#7aacdf"),
    ("--pane-widget-bg", "rgba(187, 188, 196, 0.10)"),
    ("--pane-widget-hover", "rgba(187, 188, 196, 0.15)"),
    ("--pane-widget-focus", "rgba(187, 188, 196, 0.20)"),
    ("--pane-widget-active", "rgba(187, 188, 196, 0.25)"),
    ("--pane-widget-bg-muted", "rgba(187, 188, 196, 0.06)"),
    ("--pane-tab-hover-bg", "rgba(187, 188, 196, 0.06)"),
    ("--pane-hover-bg", "rgba(255, 255, 255, 0.03)"),
    ("--pane-active-bg", "rgba(255, 255, 255, 0.05)"),
    ("--pane-popup-bg", "#1e1f24"),
    ("--pane-bg-dark", "rgba(0, 0, 0, 0.25)"),
];

pub const EXAMPLE_3D_ANCHOR: Vec3 = Vec3::new(0.0, 0.75, 0.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExampleMovePlane {
    Xy,
    Xz,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ExampleMover {
    pub speed: f32,
    pub plane: ExampleMovePlane,
}

#[derive(Resource, Default)]
pub struct ExampleTargetCycle {
    pub entities: Vec<Entity>,
    pub index: usize,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Pane)]
#[pane(title = "Top Down Camera", position = "top-right")]
pub struct ExampleTopDownPane {
    #[pane(toggle)]
    pub follow_enabled: bool,
    #[pane(toggle)]
    pub debug_gizmos: bool,
    #[pane(slider, min = 0.0, max = 220.0, step = 0.1)]
    pub dead_zone_x: f32,
    #[pane(slider, min = 0.0, max = 220.0, step = 0.1)]
    pub dead_zone_y: f32,
    #[pane(slider, min = 0.0, max = 260.0, step = 0.1)]
    pub soft_zone_x: f32,
    #[pane(slider, min = 0.0, max = 260.0, step = 0.1)]
    pub soft_zone_y: f32,
    #[pane(slider, min = -120.0, max = 120.0, step = 0.1)]
    pub bias_x: f32,
    #[pane(slider, min = -120.0, max = 120.0, step = 0.1)]
    pub bias_y: f32,
    #[pane(slider, min = 0.5, max = 36.0, step = 0.1)]
    pub zoom: f32,
    #[pane(slider, min = 0.1, max = 4.0, step = 0.05)]
    pub zoom_speed: f32,
    #[pane(slider, min = 0.5, max = 24.0, step = 0.1)]
    pub planar_damping: f32,
    #[pane(slider, min = 0.5, max = 24.0, step = 0.1)]
    pub zoom_damping: f32,
    #[pane(slider, min = 0.5, max = 24.0, step = 0.1)]
    pub yaw_damping: f32,
    #[pane(slider, min = -3.14, max = 3.14, step = 0.01)]
    pub yaw_radians: f32,
    #[pane(slider, min = 30.0, max = 88.0, step = 0.5)]
    pub pitch_degrees: f32,
}

impl Default for ExampleTopDownPane {
    fn default() -> Self {
        Self {
            follow_enabled: true,
            debug_gizmos: false,
            dead_zone_x: 96.0,
            dead_zone_y: 72.0,
            soft_zone_x: 96.0,
            soft_zone_y: 72.0,
            bias_x: 0.0,
            bias_y: 0.0,
            zoom: 1.0,
            zoom_speed: 0.2,
            planar_damping: 9.0,
            zoom_damping: 12.0,
            yaw_damping: 10.0,
            yaw_radians: 0.0,
            pitch_degrees: 60.0,
        }
    }
}

impl ExampleTopDownPane {
    pub fn from_setup(
        settings: &TopDownCameraSettings,
        zoom: f32,
        yaw_radians: f32,
        follow_enabled: bool,
        debug_gizmos: bool,
    ) -> Self {
        let pitch_degrees = match settings.mode {
            TopDownCameraMode::Flat2d { .. } => 60.0,
            TopDownCameraMode::Tilted3d { pitch, .. } => pitch.to_degrees(),
        };

        Self {
            follow_enabled,
            debug_gizmos,
            dead_zone_x: settings.dead_zone.x,
            dead_zone_y: settings.dead_zone.y,
            soft_zone_x: settings.soft_zone.x,
            soft_zone_y: settings.soft_zone.y,
            bias_x: settings.bias.x,
            bias_y: settings.bias.y,
            zoom,
            zoom_speed: settings.zoom_speed,
            planar_damping: settings.damping.planar_x.max(settings.damping.planar_y),
            zoom_damping: settings.damping.zoom,
            yaw_damping: settings.damping.yaw,
            yaw_radians,
            pitch_degrees,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ExampleTopDownPaneBootstrap(pub ExampleTopDownPane);

pub fn queue_example_pane(commands: &mut Commands, pane: ExampleTopDownPane) {
    commands.insert_resource(ExampleTopDownPaneBootstrap(pane));
}

#[derive(Resource)]
struct AutoExitAfter(Timer);

#[derive(Component, Default)]
pub struct ExampleMoveContext;

#[derive(Component, Default)]
pub struct ExampleCameraContext;

#[derive(Debug, InputAction)]
#[action_output(Vec2)]
pub struct ExampleMoveAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
pub struct ExampleZoomAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
pub struct ExampleYawAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ExampleSwitchTargetAction;

pub struct ExampleTopDownCameraControlsPlugin;

impl Plugin for ExampleTopDownCameraControlsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EnhancedInputPlugin>() {
            app.add_plugins(EnhancedInputPlugin);
        }

        app.add_input_context::<ExampleMoveContext>()
            .add_input_context::<ExampleCameraContext>()
            .add_observer(apply_move_input)
            .add_observer(apply_zoom_input)
            .add_observer(apply_yaw_input)
            .add_observer(apply_target_switch);
    }
}

pub fn apply_example_defaults(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.045, 0.055, 0.085)));

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(timer) = auto_exit_from_env() {
        app.insert_resource(timer);
        app.add_systems(Update, auto_exit_after);
    }
}

pub fn install_pane(app: &mut App) {
    app.add_plugins((
        bevy_flair::FlairPlugin,
        bevy_input_focus::InputDispatchPlugin,
        bevy_ui_widgets::UiWidgetsPlugins,
        bevy_input_focus::tab_navigation::TabNavigationPlugin,
        PanePlugin,
    ))
    .register_pane::<ExampleTopDownPane>()
    .add_systems(
        PreUpdate,
        (
            prime_pane_theme_vars,
            apply_bootstrapped_pane,
            sync_example_pane,
        )
            .chain(),
    );
}

fn prime_pane_theme_vars(mut panes: Query<&mut InlineStyle, Added<PaneRoot>>) {
    for mut style in &mut panes {
        for &(key, value) in PANE_DARK_THEME_VARS {
            style.set(key, value.to_owned());
        }
    }
}

fn apply_bootstrapped_pane(
    bootstrap: Option<Res<ExampleTopDownPaneBootstrap>>,
    mut pane: ResMut<ExampleTopDownPane>,
) {
    let Some(bootstrap) = bootstrap else {
        return;
    };

    if *pane == ExampleTopDownPane::default() {
        *pane = bootstrap.0;
    }
}

pub fn spawn_overlay(commands: &mut Commands, title: &str, body: &str) {
    commands.spawn((
        Name::new("Example Overlay"),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(18.0),
            top: Val::Px(18.0),
            width: Val::Px(420.0),
            padding: UiRect::all(Val::Px(14.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.84)),
        Text::new(format!("{title}\n{body}")),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn spawn_reference_world_2d(commands: &mut Commands, title: &str, body: &str, accent: Color) {
    commands.spawn((
        Name::new("Arena Background"),
        Sprite::from_color(Color::srgb(0.10, 0.12, 0.18), Vec2::new(1200.0, 900.0)),
    ));

    for x in -6..=6 {
        commands.spawn((
            Name::new("Arena Column"),
            Sprite::from_color(Color::srgba(0.16, 0.20, 0.28, 0.45), Vec2::new(4.0, 860.0)),
            Transform::from_xyz(x as f32 * 80.0, 0.0, -0.1),
        ));
    }
    for y in -4..=4 {
        commands.spawn((
            Name::new("Arena Row"),
            Sprite::from_color(Color::srgba(0.16, 0.20, 0.28, 0.45), Vec2::new(1160.0, 4.0)),
            Transform::from_xyz(0.0, y as f32 * 80.0, -0.1),
        ));
    }

    for index in 0..6 {
        let x = -340.0 + index as f32 * 136.0;
        let y = if index % 2 == 0 { -220.0 } else { 220.0 };
        commands.spawn((
            Name::new("Reference Landmark"),
            Sprite::from_color(accent.with_alpha(0.75), Vec2::new(48.0, 48.0)),
            Transform::from_xyz(x, y, 0.2),
        ));
    }

    spawn_overlay(commands, title, body);
}

pub fn spawn_reference_world_3d(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    title: &str,
    body: &str,
    accent: Color,
) {
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.55, 0.58, 0.64),
        brightness: 140.0,
        affects_lightmapped_meshes: true,
    });

    commands.spawn((
        Name::new("Reference Sun"),
        DirectionalLight {
            illuminance: 36_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, 0.8, 0.0)),
    ));

    commands.spawn((
        Name::new("Reference Ground"),
        Mesh3d(meshes.add(Plane3d::default().mesh().size(48.0, 48.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.11, 0.14, 0.18),
            perceptual_roughness: 1.0,
            ..default()
        })),
    ));

    for x in -2..=2 {
        for z in -2..=2 {
            if x == 0 && z == 0 {
                continue;
            }
            let height = if (x + z) % 2 == 0 { 1.8 } else { 1.2 };
            commands.spawn((
                Name::new("Reference Pillar"),
                Mesh3d(meshes.add(Cuboid::new(1.4, height, 1.4))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: if x == 2 && z == -2 {
                        accent
                    } else {
                        Color::srgb(0.24, 0.28, 0.36)
                    },
                    perceptual_roughness: 0.82,
                    ..default()
                })),
                Transform::from_xyz(x as f32 * 6.0, height * 0.5, z as f32 * 6.0),
            ));
        }
    }

    commands.spawn((
        Name::new("Reference Ring"),
        Mesh3d(meshes.add(Torus::new(4.5, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: accent.with_alpha(0.92),
            emissive: accent.into(),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.04, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
    ));

    for x in -4..=4 {
        for z in -4..=4 {
            commands.spawn((
                Name::new("Reference Floor Tile"),
                Mesh3d(meshes.add(Cuboid::new(1.35, 0.08, 1.35))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: if (x + z) % 2 == 0 {
                        Color::srgb(0.16, 0.19, 0.24)
                    } else {
                        Color::srgb(0.09, 0.11, 0.16)
                    },
                    perceptual_roughness: 1.0,
                    ..default()
                })),
                Transform::from_xyz(x as f32 * 2.1, 0.04, z as f32 * 2.1),
            ));
        }
    }

    spawn_overlay(commands, title, body);
}

pub fn spawn_target_2d(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    name: &str,
    translation: Vec3,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_owned()),
            ExampleMover {
                speed: 260.0,
                plane: ExampleMovePlane::Xy,
            },
            Mesh2d(meshes.add(Circle::new(26.0))),
            MeshMaterial2d(materials.add(color)),
            Transform::from_translation(translation),
        ))
        .id()
}

pub fn spawn_target_3d(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    translation: Vec3,
    color: Color,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_owned()),
            ExampleMover {
                speed: 7.5,
                plane: ExampleMovePlane::Xz,
            },
            Mesh3d(meshes.add(Capsule3d::new(0.45, 1.2).mesh().rings(8).latitudes(10))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.28,
                ..default()
            })),
            Transform::from_translation(translation),
        ))
        .id()
}

pub fn spawn_camera_2d(
    commands: &mut Commands,
    name: &str,
    target_anchor: Vec3,
    zoom: f32,
    settings: TopDownCameraSettings,
    debug: bool,
) -> Entity {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        Camera2d,
        TopDownCamera {
            target_anchor,
            zoom,
            ..default()
        },
        settings,
    ));

    if debug {
        entity.insert(TopDownCameraDebug::default());
    }

    entity.id()
}

pub fn spawn_camera_3d_perspective(
    commands: &mut Commands,
    name: &str,
    target_anchor: Vec3,
    yaw: f32,
    distance: f32,
    settings: TopDownCameraSettings,
    debug: bool,
) -> Entity {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 42.0_f32.to_radians(),
            ..default()
        }),
        TopDownCamera::looking_at_3d(target_anchor, yaw, distance),
        settings,
    ));

    if debug {
        entity.insert(TopDownCameraDebug::default());
    }

    entity.id()
}

pub fn spawn_camera_3d_orthographic(
    commands: &mut Commands,
    name: &str,
    target_anchor: Vec3,
    yaw: f32,
    scale: f32,
    settings: TopDownCameraSettings,
    debug: bool,
) -> Entity {
    let mut entity = commands.spawn((
        Name::new(name.to_owned()),
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scale,
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 18.0,
            },
            ..OrthographicProjection::default_3d()
        }),
        TopDownCamera::looking_at_3d(target_anchor, yaw, scale),
        settings,
    ));

    if debug {
        entity.insert(TopDownCameraDebug::default());
    }

    entity.id()
}

pub fn attach_target_controls(commands: &mut Commands, target: Entity) {
    commands.entity(target).insert((
        ExampleMoveContext,
        actions!(ExampleMoveContext[
            (
                Action::<ExampleMoveAction>::new(),
                Bindings::spawn((Cardinal::wasd_keys(),)),
            ),
        ]),
    ));
}

pub fn attach_camera_controls(commands: &mut Commands, camera: Entity) {
    commands.entity(camera).insert((
        ExampleCameraContext,
        actions!(ExampleCameraContext[
            (
                Action::<ExampleZoomAction>::new(),
                Bindings::spawn((Bidirectional::new(KeyCode::KeyX, KeyCode::KeyZ),)),
            ),
            (
                Action::<ExampleYawAction>::new(),
                Bindings::spawn((Bidirectional::new(KeyCode::KeyE, KeyCode::KeyQ),)),
            ),
            (
                Action::<ExampleSwitchTargetAction>::new(),
                bindings![KeyCode::Tab],
            ),
        ]),
    ));
}

fn apply_move_input(
    event: On<Fire<ExampleMoveAction>>,
    time: Res<Time>,
    mut movers: Query<(&ExampleMover, &mut Transform)>,
) {
    let Ok((mover, mut transform)) = movers.get_mut(event.context) else {
        return;
    };

    let delta = event.value * mover.speed * time.delta_secs();
    match mover.plane {
        ExampleMovePlane::Xy => {
            transform.translation.x += delta.x;
            transform.translation.y += delta.y;
        }
        ExampleMovePlane::Xz => {
            transform.translation.x += delta.x;
            transform.translation.z -= delta.y;
        }
    }
}

fn apply_zoom_input(
    event: On<Fire<ExampleZoomAction>>,
    time: Res<Time>,
    mut pane: ResMut<ExampleTopDownPane>,
    mut cameras: Query<(&TopDownCameraSettings, &mut TopDownCamera)>,
) {
    let Ok((settings, mut camera)) = cameras.get_mut(event.context) else {
        return;
    };

    camera.zoom -= event.value * settings.zoom_speed * 6.0 * time.delta_secs();
    pane.zoom = camera.zoom.clamp(settings.zoom_min, settings.zoom_max);
}

fn apply_yaw_input(
    event: On<Fire<ExampleYawAction>>,
    time: Res<Time>,
    mut pane: ResMut<ExampleTopDownPane>,
    mut cameras: Query<&mut TopDownCamera>,
) {
    let Ok(mut camera) = cameras.get_mut(event.context) else {
        return;
    };

    camera.target_yaw += event.value * 1.8 * time.delta_secs();
    pane.yaw_radians = camera.target_yaw;
}

fn apply_target_switch(
    event: On<Start<ExampleSwitchTargetAction>>,
    mut cycle: ResMut<ExampleTargetCycle>,
    mut cameras: Query<&mut TopDownCamera>,
) {
    if cycle.entities.is_empty() {
        return;
    }

    let Ok(mut camera) = cameras.get_mut(event.context) else {
        return;
    };

    cycle.index = (cycle.index + 1) % cycle.entities.len();
    camera.tracked_target = Some(cycle.entities[cycle.index]);
}

fn auto_exit_from_env() -> Option<AutoExitAfter> {
    let millis = std::env::var("TOP_DOWN_CAMERA_AUTO_EXIT_MS")
        .ok()?
        .parse::<u64>()
        .ok()?;
    Some(AutoExitAfter(Timer::new(
        Duration::from_millis(millis),
        TimerMode::Once,
    )))
}

fn auto_exit_after(
    time: Res<Time>,
    mut timer: ResMut<AutoExitAfter>,
    mut exit: MessageWriter<AppExit>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        exit.write(AppExit::Success);
    }
}

fn sync_example_pane(
    mut pane: ResMut<ExampleTopDownPane>,
    bootstrap: Option<Res<ExampleTopDownPaneBootstrap>>,
    mut commands: Commands,
    mut cameras: Query<(
        Entity,
        &mut TopDownCamera,
        &mut TopDownCameraSettings,
        Option<&TopDownCameraDebug>,
    )>,
) {
    let has_bootstrap = bootstrap.is_some();
    if let Some(bootstrap) = bootstrap {
        if *pane == ExampleTopDownPane::default() && bootstrap.0 != *pane {
            *pane = bootstrap.0;
        }
    }

    let effective_pane = *pane;

    for (entity, mut camera, mut settings, debug) in &mut cameras {
        let scene_pane = ExampleTopDownPane::from_setup(
            &settings,
            camera.zoom,
            camera.target_yaw,
            camera.follow_enabled,
            debug.is_some(),
        );
        if !has_bootstrap && *pane == ExampleTopDownPane::default() && scene_pane != *pane {
            *pane = scene_pane;
            return;
        }

        let dead_zone = Vec2::new(
            effective_pane.dead_zone_x.max(0.0),
            effective_pane.dead_zone_y.max(0.0),
        );
        let soft_zone = Vec2::new(
            effective_pane.soft_zone_x.max(dead_zone.x),
            effective_pane.soft_zone_y.max(dead_zone.y),
        );

        settings.dead_zone = dead_zone;
        settings.soft_zone = soft_zone;
        settings.bias = Vec2::new(effective_pane.bias_x, effective_pane.bias_y);
        settings.damping.planar_x = effective_pane.planar_damping;
        settings.damping.planar_y = effective_pane.planar_damping;
        settings.damping.zoom = effective_pane.zoom_damping;
        settings.damping.yaw = effective_pane.yaw_damping;
        settings.zoom_speed = effective_pane.zoom_speed;

        if let TopDownCameraMode::Tilted3d { pitch, .. } = &mut settings.mode {
            *pitch = effective_pane.pitch_degrees.to_radians();
        }

        camera.follow_enabled = effective_pane.follow_enabled;
        camera.target_yaw = effective_pane.yaw_radians;
        camera.zoom = effective_pane
            .zoom
            .clamp(settings.zoom_min, settings.zoom_max);

        if effective_pane.debug_gizmos && debug.is_none() {
            commands
                .entity(entity)
                .insert(TopDownCameraDebug::default());
        } else if !effective_pane.debug_gizmos && debug.is_some() {
            commands.entity(entity).remove::<TopDownCameraDebug>();
        }
    }
}
