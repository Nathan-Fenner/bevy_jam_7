use bevy::prelude::*;
use rand::Rng;

use crate::{
    billboard::{Billboard, BillboardCamera, BillboardPlugin},
    item::{Item, ItemPlugin},
    player::{PlayerPlugin, Wall},
};

pub mod billboard;
pub mod item;
pub mod player;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        }))
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins((ItemPlugin, BillboardPlugin, PlayerPlugin))
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // player
    commands.spawn((
        player::Player {
            velocity: Vec3::ZERO,
            recent_velocity: Vec3::ZERO,
            facing_direction: 1.,
            cursor: Vec3::new(1., 0., 0.),
        },
        Billboard {
            image: "duck_realtor.png".to_string(),
        },
        Transform::from_xyz(0.0, 0.45, 0.0),
    ));

    // wall

    let mut rng = rand::rng();

    let wall_mesh = meshes.add(Cuboid::default());
    let wall_material = materials.add(Color::linear_rgb(1., 0.5, 0.2));
    for x in -5..=5i32 {
        for z in -5..=5i32 {
            if x.abs() <= 2 && z.abs() <= 2 {
                continue;
            }

            if rng.random_bool(0.5) {
                continue;
            }

            commands.spawn((
                player::Wall { enabled: true },
                Mesh3d(wall_mesh.clone()),
                MeshMaterial3d(wall_material.clone()),
                Transform::from_translation(Vec3::new(x as f32, 0.5, z as f32)),
            ));
        }
    }
    commands.spawn((
        Item { is_held: false },
        Billboard {
            image: "apple.png".to_string(),
        },
        Transform::from_translation(Vec3::new(2., 0.5, 1.)),
    ));

    commands.spawn((
        Item { is_held: false },
        Billboard {
            image: "fence.png".to_string(),
        },
        Wall { enabled: true },
        Transform::from_translation(Vec3::new(-2., 0.5, 1.)),
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        BillboardCamera,
    ));
}
