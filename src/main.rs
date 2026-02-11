use bevy::prelude::*;

use crate::{
    billboard::{Billboard, BillboardCamera, BillboardPlugin},
    blueprint::{Blueprint, BlueprintPlugin, Door},
    item::{Item, ItemPlugin},
    player::{PlayerPlugin, Wall, Water},
    rooms::RoomsPlugin,
};

pub mod billboard;
pub mod blueprint;
pub mod item;
pub mod player;
pub mod rooms;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics in web builds on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: bevy::asset::AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(avian3d::PhysicsPlugins::default())
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins((
            RoomsPlugin,
            ItemPlugin,
            BillboardPlugin,
            PlayerPlugin,
            BlueprintPlugin,
        ))
        .add_systems(Startup, setup)
        .insert_resource(Time::<Virtual>::from_max_delta(
            std::time::Duration::from_millis(60),
        ))
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let level = image::open("assets/level.png").expect("can load level");
    let level = level.as_rgb8().unwrap();

    type LevelColor = [u8; 3];
    const COLOR_PLAYER: LevelColor = [255, 0, 0];
    const COLOR_FLOOR: LevelColor = [255, 255, 255];
    const COLOR_WATER: LevelColor = [128, 128, 255];
    const COLOR_WALL: LevelColor = [128, 128, 128];

    const COLOR_FENCE: LevelColor = [255, 64, 0];
    const COLOR_BRIDGE: LevelColor = [128, 64, 0];
    const COLOR_DOOR: LevelColor = [255, 128, 0];
    const COLOR_BLUEPRINT: LevelColor = [0, 0, 255];
    const COLOR_BRICK_WALL: LevelColor = [255, 60, 0];

    let wall_mesh = meshes.add(Cuboid::default());
    let wall_material = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(1., 0.5, 0.2),
        perceptual_roughness: 1.,
        ..default()
    });

    let floor_material = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.7, 0.9, 0.8),
        perceptual_roughness: 1.,
        ..default()
    });

    let water_material = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.2, 0.4, 0.6),
        perceptual_roughness: 0.25,
        ..default()
    });

    let bridge_material = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(0.9, 0.8, 0.5),
        ..default()
    });

    let cube_collider = avian3d::prelude::Collider::cuboid(1., 1., 1.);

    let ground_collider = avian3d::prelude::Collider::cuboid(1000., 1., 1000.);

    commands.spawn((
        avian3d::prelude::RigidBody::Static,
        ground_collider,
        Transform::from_translation(Vec3::new(0., -0.5, 0.)),
    ));

    for x in 0..level.width() {
        for y in 0..level.height() {
            let pixel: LevelColor = level[(x, y)].0;
            let at = Vec3::new(x as f32, 0., y as f32);

            if pixel == COLOR_PLAYER {
                // spawn player object

                /*

                         RigidBody::Dynamic,
                Collider::capsule(0.5, 1.5),
                Transform::from_xyz(0.0, 3.0, 0.0),
                         */

                commands.spawn((
                    avian3d::prelude::RigidBody::Dynamic,
                    avian3d::prelude::Collider::capsule(0.3, 0.8),
                    avian3d::prelude::LockedAxes::ROTATION_LOCKED,
                    player::Player {
                        velocity: Vec3::ZERO,
                        recent_velocity: Vec3::ZERO,
                        facing_direction: 1.,
                        cursor: Vec3::new(1., 0., 0.),
                    },
                    Billboard {
                        image: "duck_realtor.png".to_string(),
                    },
                    Transform::from_translation(at + Vec3::new(0.0, 3.9, 0.0)),
                ));
            }
            if pixel == COLOR_DOOR {
                commands.spawn((
                    Billboard {
                        image: "door.png".to_string(),
                    },
                    Transform::from_translation(at + Vec3::new(0.0, 0.45, 0.0)),
                    Door,
                ));
            }
            if pixel == COLOR_BLUEPRINT {
                commands.spawn((
                    Billboard {
                        image: "blueprint.png".to_string(),
                    },
                    Blueprint,
                    Transform::from_translation(at + Vec3::new(0.0, 0.45, 0.0)),
                ));
            }
            if pixel == COLOR_WALL {
                commands.spawn((
                    player::Wall { enabled: true },
                    avian3d::prelude::RigidBody::Static,
                    cube_collider.clone(),
                    Mesh3d(wall_mesh.clone()),
                    MeshMaterial3d(wall_material.clone()),
                    Transform::from_translation(at + Vec3::new(0., 0.5, 0.)),
                ));
            }
            if pixel == COLOR_BRICK_WALL {
                commands.spawn((
                    avian3d::prelude::RigidBody::Static,
                    cube_collider.clone(),
                    player::Wall { enabled: true },
                    Item {
                        glued: Vec::new(),
                        is_held: None,
                    },
                    Billboard {
                        image: "brick_wall.png".to_string(),
                    },
                    Transform::from_translation(at + Vec3::new(0., 0.5, 0.)),
                ));
            }
            if pixel != COLOR_WATER {
                commands.spawn((
                    Mesh3d(wall_mesh.clone()),
                    MeshMaterial3d(floor_material.clone()),
                    Transform::from_translation(at + Vec3::new(0., -0.5, 0.)),
                ));
            }
            if pixel == COLOR_WATER {
                commands.spawn((
                    Mesh3d(wall_mesh.clone()),
                    MeshMaterial3d(water_material.clone()),
                    Transform::from_translation(at + Vec3::new(0., -0.75, 0.)),
                    Water {},
                ));
            }
        }
    }

    // npc
    commands.spawn((
        Billboard {
            image: "blue_bird.png".to_string(),
        },
        Transform::from_xyz(2.0, 0.45, 0.0),
    ));

    // wall
    commands.spawn((
        Item {
            is_held: None,
            glued: Vec::new(),
        },
        Billboard {
            image: "apple.png".to_string(),
        },
        Transform::from_translation(Vec3::new(2., 0.5, 1.)),
    ));

    commands.spawn((
        Item {
            is_held: None,
            glued: vec![IVec2::new(-1, 0), IVec2::new(1, 0)],
        },
        Billboard {
            image: "fence.png".to_string(),
        },
        Wall { enabled: true },
        Transform::from_translation(Vec3::new(-2., 0.5, 1.)),
    ));
    commands.spawn((
        Item {
            is_held: None,
            glued: vec![IVec2::new(1, 0), IVec2::new(2, 0)],
        },
        Billboard {
            image: "fence.png".to_string(),
        },
        Wall { enabled: true },
        Transform::from_translation(Vec3::new(-3., 0.5, 1.)),
    ));
    commands.spawn((
        Item {
            is_held: None,
            glued: vec![IVec2::new(-1, 0), IVec2::new(-2, 0)],
        },
        Billboard {
            image: "fence.png".to_string(),
        },
        Wall { enabled: true },
        Transform::from_translation(Vec3::new(-1., 0.5, 1.)),
    ));

    commands.spawn((
        Billboard {
            image: "ghost_of_real_estate.png".to_string(),
        },
        Transform::from_translation(Vec3::new(12., 0.5, 17.)),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 8.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        BillboardCamera,
    ));
}
