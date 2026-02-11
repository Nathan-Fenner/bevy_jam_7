use bevy::{
    asset::RenderAssetUsages,
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

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

    let room_collider_image =
        image::open("assets/room4_collider.png").expect("can load room collider image");
    let room_collider_image = room_collider_image.as_rgba8().expect("is rgba8");

    type ImageLayer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

    let is_solid = |layer: &ImageLayer, p: IVec2| -> bool {
        if p.x < 0 || p.y < 0 || p.x >= layer.width() as i32 || p.y >= layer.height() as i32 {
            return false;
        }
        layer[(p.x as u32, p.y as u32)].0[3] >= 128
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
            let h = if is_solid(&room_image, IVec2::new(x as i32, y as i32)) {
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

    let room_size = 3.;

    for v in &vert_list {
        attr_pos.push(Vec3::new(
            (v.x as f32 / room_image.width() as f32 - 0.5) * room_size,
            match v.y {
                2 => 1.,
                1 => 0.05,
                _ => 0.,
            },
            (v.z as f32 / room_image.height() as f32 - 0.5) * room_size,
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
            if is_solid(&room_image, neighbor_pixel) {
                sum_uv += (shift.as_vec2() + 0.5) * 0.1 / room_image.width() as f32;
            }
        }

        attr_uv0.push(sum_uv);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, attr_pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, attr_uv0);
    mesh.insert_indices(bevy::mesh::Indices::U32(triangles));

    let mut room_colliders: Vec<(Vec3, avian3d::prelude::Rotation, avian3d::prelude::Collider)> =
        Vec::new();

    {
        let mut visited: HashSet<IVec2> = HashSet::new();
        for x in 0..room_collider_image.width() as i32 {
            for y in 0..room_collider_image.height() as i32 {
                let p = IVec2::new(x, y);
                if !is_solid(&room_collider_image, p) || visited.contains(&p) {
                    continue;
                }

                visited.insert(p);
                let mut region_list: Vec<IVec2> = Vec::new();
                let mut region_stack = vec![p];

                while let Some(c) = region_stack.pop() {
                    region_list.push(c);
                    for dir in [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y] {
                        let n = c + dir;
                        if !is_solid(&room_collider_image, n)
                            || visited.contains(&n)
                            || room_collider_image[(n.x as u32, n.y as u32)].0[..3]
                                != room_collider_image[(p.x as u32, p.y as u32)].0[..3]
                        {
                            continue;
                        }
                        visited.insert(n);
                        region_stack.push(n);
                    }
                }

                let room_collider_height = 1.2;

                let mut bound_min = p;
                let mut bound_max = p;
                for &c in &region_list {
                    bound_min = bound_min.min(c);
                    bound_max = bound_max.max(c);
                }
                let lower = (to_uv(bound_min) - 0.5) * room_size;
                let upper = (to_uv(bound_max + IVec2::new(1, 1)) - 0.5) * room_size;
                room_colliders.push((
                    Vec3::new(
                        (lower.x + upper.x) / 2.,
                        room_collider_height / 2.,
                        (lower.y + upper.y) / 2.,
                    ),
                    avian3d::prelude::Rotation::IDENTITY,
                    avian3d::prelude::Collider::cuboid(
                        upper.x - lower.x,
                        room_collider_height,
                        upper.y - lower.y,
                    ),
                ));
            }
        }
    }

    let room_collider = avian3d::prelude::Collider::compound(room_colliders);

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
        avian3d::prelude::RigidBody::Static,
        room_collider,
    ));
}
