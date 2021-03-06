use bevy::{input::mouse::MouseMotion, window::WindowMode};
use bevy::prelude::*;
use concord::logger::{console_info, get_console_logs};

use concord::{ConcordPlugins, editor::SelectableTag};
use bevy_kajiya::{kajiya_render::{KajiyaDescriptor, KajiyaCameraBundle, KajiyaCamera, KajiyaMeshInstanceBundle, KajiyaMeshInstance, EnvironmentSettings}, BevyKajiyaPlugins};
use dolly::prelude::{CameraRig, Position, Smooth, YawPitch};

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Concord Prototype".to_string(),
            width: 1920. / 1.5,
            height: 1080. / 1.5,
            vsync: false,
            resizable: false,
            mode: WindowMode::Windowed,
            scale_factor_override: Some(1.5),
            ..Default::default()
        })
        .insert_resource(KajiyaDescriptor {
            scene_name: "main".to_string(),
            gi_volume_scale: 6.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyKajiyaPlugins)
        .add_plugins(ConcordPlugins)
        .add_startup_system(setup_camera_rig)
        // .add_startup_system(setup_world)
        .add_system(sun_move)
        .add_system(rotator_system)
        .add_system(drive_camera)
        .run();

    }

#[derive(Component, Copy, Clone)]
struct BodyTag;

// TODO: Added<CameraRig> to fetch loaded cam's trans first and apply to rig cam
fn setup_camera_rig(mut commands: Commands) {
    
    // Not required, just a nice camera driver to give easy, smooth, camera controls.
    let camera_rig = CameraRig::builder()
        .with(Position::new(dolly::glam::Vec3::new(0.0, 2.5, 10.0)))
        .with(YawPitch::new().rotation_quat(dolly::glam::Quat::IDENTITY))
        .with(Smooth::new_position_rotation(1.0, 1.0))
        .build();

    commands.insert_resource(camera_rig);

}

fn setup_world(mut commands: Commands, windows: Res<Windows>) {

    // Spawn an entity to control the kajiya renderer camera.  Only 1 camera is allowed at the moment.
    // The cameara bundle also provides the EnvironmentSettings components to give the user access to
    // the sun state.
    let window = windows.get_primary().unwrap();
    commands.spawn_bundle(KajiyaCameraBundle {
        camera: KajiyaCamera {
            aspect_ratio: window.requested_width() / window.requested_height(),
            ..KajiyaCamera::default()
        },
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn_bundle(KajiyaMeshInstanceBundle {
            mesh_instance: KajiyaMeshInstance { 
                mesh: "smiley_box".to_string(),
                ..Default::default()
            },
            transform: Transform::from_translation(Vec3::new(0.0,0.0,0.4)).with_scale(Vec3::splat(0.1)),
            ..Default::default()
        }).insert(BodyTag);
    });

    // Spawn a new "user" mesh instance with the "ring" mesh
    commands.spawn_bundle(KajiyaMeshInstanceBundle {
        mesh_instance: KajiyaMeshInstance { 
            mesh: "ring".to_string(),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
        ..Default::default()
    }).insert(Rotator {
        ccw: true,
    });
    
    commands.spawn_bundle(KajiyaMeshInstanceBundle {
        mesh_instance: KajiyaMeshInstance { 
            mesh: "ring".to_string(),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(-5.0, -0.001, -5.0)),
        ..Default::default()
    }).insert(SelectableTag);

    commands.spawn_bundle(KajiyaMeshInstanceBundle {
        mesh_instance: KajiyaMeshInstance { 
            mesh: "336_lrm".to_string(),
            selection_bb_size: 100.0,
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(-5.0, 5.0, -5.0)).with_scale(Vec3::splat(0.01)),
        ..Default::default()
    }).insert(SelectableTag);

    commands.spawn_bundle(KajiyaMeshInstanceBundle {
        mesh_instance: KajiyaMeshInstance { 
            mesh: "ring".to_string(),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(-5.0, 10., -5.0)),
        ..Default::default()
    }).insert(SelectableTag);

    commands.spawn_bundle(KajiyaMeshInstanceBundle {
        mesh_instance: KajiyaMeshInstance { 
            mesh: "mirror".to_string(),
            ..Default::default()
        },
        transform: Transform::from_translation(Vec3::new(20.0, 0.0, 0.0)),
        ..Default::default()
    });

}

fn sun_move(time: Res<Time>, mut query: Query<&mut EnvironmentSettings, With<KajiyaCamera>>, mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
) {
    let mut env = if let Some(mut env) = query.iter_mut().next() {
        env
    } else {
        return;
    };

    let mut mouse_delta = Vec2::ZERO;
    let mouse_sensitivity = 0.005;
    if mouse_buttons.pressed(MouseButton::Middle) || (keys.pressed(KeyCode::LControl) && mouse_buttons.pressed(MouseButton::Right)) {
        for event in mouse_motion_events.iter() {
            mouse_delta += event.delta;
        }
        env.sun_theta_phi.0 += mouse_sensitivity * mouse_delta.x;
        env.sun_theta_phi.1 += mouse_sensitivity * mouse_delta.y;
    } else {
        // let time_scale = 0.0005;
        // let theta = 180.0 * (time.time_since_startup().as_secs_f32() * time_scale).sin();
        // let phi = 180.0 * (time.time_since_startup().as_secs_f32() * time_scale).sin();
        // env.sun_theta_phi = (theta, phi);
    }
}

/// this component indicates what entities should rotate
#[derive(Component, Clone)]
struct Rotator {
    ccw: bool,
}

/// rotates the parent, which will result in the child also rotating
fn rotator_system(time: Res<Time>, mut query: Query<(&mut Transform, &Rotator)>) {
    for (mut transform, rotator) in query.iter_mut() {
        let ang_vel = if rotator.ccw {
            1.0
        } else {
            -1.0
        };

        transform.scale = Vec3::splat((time.seconds_since_startup()).sin() as f32 + 1.0);
        transform.rotation *= Quat::from_rotation_y(ang_vel * time.delta_seconds());
    }
}

fn drive_camera(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut camera_rig: ResMut<CameraRig>,
    mut query: Query<&mut Transform, With<KajiyaCamera>>,
) {
    
    let mut camera_transform = if let Some(camera_transform) = query.iter_mut().next() {
        camera_transform
    } else {
        return;
    };

    let time_delta_seconds: f32 = time.delta_seconds();

    let mut move_vec = Vec3::ZERO;
    let mut boost = 0.0;

    if keys.pressed(KeyCode::LShift) {
        boost = 1.0;
    }
    if keys.pressed(KeyCode::LControl) {
        boost = -1.0;
    }

    if keys.pressed(KeyCode::W) {
        move_vec.z -= 1.0;
    }
    if keys.pressed(KeyCode::S) {
        move_vec.z += 1.0;
    }
    if keys.pressed(KeyCode::A) {
        move_vec.x -= 1.0;
    }
    if keys.pressed(KeyCode::D) {
        move_vec.x += 1.0;
    }

    if keys.pressed(KeyCode::Q) {
        move_vec.y += 1.0;
    }
    if keys.pressed(KeyCode::E) {
        move_vec.y -= 1.0;
    }

    let mut mouse_delta = Vec2::ZERO;
    if mouse_buttons.pressed(MouseButton::Right) {
        for event in mouse_motion_events.iter() {
            mouse_delta += event.delta;
        }
    }

    let move_vec = camera_rig.final_transform.rotation * move_vec * 10.0f32.powf(boost);

    camera_rig
        .driver_mut::<Position>()
        .translate(move_vec * time_delta_seconds * 2.5);

    camera_rig
        .driver_mut::<YawPitch>()
        .rotate_yaw_pitch(-0.1 * mouse_delta.x, -0.1 * mouse_delta.y);

    camera_rig.update(time_delta_seconds);

    camera_transform.translation = camera_rig.final_transform.position;
    camera_transform.rotation = camera_rig.final_transform.rotation;
}