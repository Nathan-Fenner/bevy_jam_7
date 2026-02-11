use bevy::{asset::RenderAssetUsages, platform::collections::HashMap, prelude::*};

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_rooms);
    }
}

#[derive(Resource)]
pub struct RoomInfo {}

pub fn setup_rooms(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let room_image = image::open("assets/room4.png").expect("can load room image");
    let room_image = room_image.as_rgba8().expect("is rgba8");

    // TODO: optimize mesh, or whatever

    let is_solid = |p: IVec2| -> bool {
        if p.x < 0
            || p.y < 0
            || p.x >= room_image.width() as i32
            || p.y >= room_image.height() as i32
        {
            return false;
        }
        room_image[(p.x as u32, p.y as u32)].0[3] >= 128
    };

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        RenderAssetUsages::all(),
    );
    let mut verts: HashMap<IVec3, u32> = HashMap::new();
    let mut vert_list: Vec<IVec3> = Vec::new();

    let mut add_vertex = |v: IVec3| -> u32 {
        if let Some(v) = verts.get(&v) {
            return *v;
        };
        let index = vert_list.len() as u32;
        verts.insert(v, index);
        vert_list.push(v);
        index
    };

    let mut triangles: Vec<u32> = Vec::new();

    for x in 0..room_image.width() {
        for y in 0..room_image.height() {
            let h = if is_solid(IVec2::new(x as i32, y as i32)) {
                2
            } else {
                1
            };
            // Otherwise, create a pixel for this cell.

            let p = IVec3::new(x as i32, h, y as i32);

            let around = [
                p,
                p + IVec3::new(0, 0, 1),
                p + IVec3::new(1, 0, 1),
                p + IVec3::new(1, 0, 0),
            ];

            triangles.push(add_vertex(around[0]));
            triangles.push(add_vertex(around[1]));
            triangles.push(add_vertex(around[3]));

            triangles.push(add_vertex(around[1]));
            triangles.push(add_vertex(around[2]));
            triangles.push(add_vertex(around[3]));

            for i in 0..4 {
                let a0 = around[i];
                let a1 = around[i] * IVec3::new(1, 0, 1);
                let b0 = around[(i + 1) % 4];
                let b1 = around[(i + 1) % 4] * IVec3::new(1, 0, 1);

                triangles.push(add_vertex(a0));
                triangles.push(add_vertex(a1));
                triangles.push(add_vertex(b0));

                triangles.push(add_vertex(a1));
                triangles.push(add_vertex(b1));
                triangles.push(add_vertex(b0));
            }

            // triangles.push(add_vertex(p + IVec3::new(1, 0, 0)));
        }
    }

    let mut attr_pos: Vec<Vec3> = Vec::new();
    let mut attr_uv0: Vec<Vec2> = Vec::new();

    let to_uv = |v: IVec2| -> Vec2 {
        Vec2::new(
            v.x as f32 / room_image.width() as f32,
            v.y as f32 / room_image.height() as f32,
        )
    };

    for v in &vert_list {
        attr_pos.push(Vec3::new(
            (v.x as f32 / room_image.width() as f32 - 0.5) * 3.,
            match v.y {
                2 => 1.,
                1 => 0.05,
                _ => 0.,
            },
            (v.z as f32 / room_image.height() as f32 - 0.5) * 3.,
        ));

        // Nudge the UV inward slightly.

        let mut sum_uv = to_uv(v.xz());
        for shift in [
            IVec2::new(0, 0),
            IVec2::new(-1, 0),
            IVec2::new(0, -1),
            IVec2::new(-1, -1),
        ] {
            let neighbor_pixel = v.xz() + shift;
            if is_solid(neighbor_pixel) {
                sum_uv += (shift.as_vec2() + 0.5) * 0.1 / room_image.width() as f32;
            }
        }

        attr_uv0.push(sum_uv);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, attr_pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, attr_uv0);
    mesh.insert_indices(bevy::mesh::Indices::U32(triangles));
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    let mesh_handle = meshes.add(mesh);

    println!("SPAWN THE ROOM");
    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(asset_server.load_with_settings(
                "room4.png",
                |settings: &mut bevy::image::ImageLoaderSettings| {
                    settings.sampler = bevy::image::ImageSampler::Descriptor(
                        bevy::image::ImageSamplerDescriptor {
                            mag_filter: bevy::image::ImageFilterMode::Nearest,
                            ..default()
                        },
                    );
                },
            )),
            base_color: Color::linear_rgb(0.7, 0.8, 0.9),
            ..default()
        })),
    ));
}
