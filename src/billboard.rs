use bevy::{platform::collections::HashMap, prelude::*};

use crate::player::Player;

#[derive(Resource)]
pub struct BillboardMaterials {
    pub materials: HashMap<String, Handle<StandardMaterial>>,
    pub mesh: Handle<Mesh>,
}

pub struct BillboardPlugin;

impl Plugin for BillboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_billboards_system)
            .add_systems(
                Update,
                (
                    add_mesh_system,
                    track_billboards_system.after(crate::player::move_player_system),
                ),
            );
    }
}

#[derive(Component)]
pub struct Billboard {
    pub image: String,
}

#[derive(Component)]
pub struct BillboardCamera;

fn add_mesh_system(
    mut commands: Commands,
    billboards: Query<(Entity, &Billboard), Without<Mesh3d>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cached_materials: ResMut<BillboardMaterials>,
    asset_server: Res<AssetServer>,
) {
    for (billboard_entity, billboard) in billboards.iter() {
        let mesh = cached_materials.mesh.clone();
        let material = cached_materials
            .materials
            .entry(billboard.image.clone())
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    unlit: true,
                    base_color_texture: Some(asset_server.load(&billboard.image)),
                    alpha_mode: AlphaMode::Mask(0.5),
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })
            });

        commands
            .entity(billboard_entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material.clone())));
    }
}

fn setup_billboards_system(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Plane3d::new(-Vec3::Z, Vec2::splat(0.5)));
    commands.insert_resource(BillboardMaterials {
        mesh,
        materials: HashMap::new(),
    });
}

fn track_billboards_system(
    time: Res<Time>,
    camera: Query<&Transform, With<BillboardCamera>>,
    mut billboards: Query<
        (&mut Transform, Option<&Player>),
        (Without<BillboardCamera>, With<Billboard>),
    >,
) {
    let total_time = time.elapsed_secs();
    let Ok(camera) = camera.single() else {
        return;
    };
    for (mut billboard_transform, player) in billboards.iter_mut() {
        let facing_direction = match player {
            Some(player) => {
                if player.facing_direction == 0. {
                    1.
                } else {
                    player.facing_direction
                }
            }
            None => 1.,
        };

        let wiggle = match player {
            Some(player) => player.velocity.length().min(1.) * (total_time * 19.).cos() * 0.1,
            None => 0.,
        };

        let target = -*camera.forward() * Vec3::new(1., 0., 1.) * facing_direction;

        billboard_transform.look_to(target, -Vec3::Y);
        billboard_transform.rotate_local_x(-0.5 * facing_direction);
        billboard_transform.rotate_local_z(wiggle);
    }
}
