use bevy::{platform::collections::HashSet, prelude::*};

use crate::billboard::BillboardCamera;
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_walls_system).add_systems(
            FixedUpdate,
            (gather_walls_system, move_player_system, move_camera_system).chain(),
        );
    }
}

#[derive(Component, Debug)]
pub struct Player {
    pub velocity: Vec3,
    pub facing_direction: f32,
    /// A moving average of velocity intent.
    pub recent_velocity: Vec3,
    /// The location where they want to pick up or drop items.
    pub cursor: Vec3,
}

#[derive(Component)]
pub struct Wall {
    pub enabled: bool,
}

#[derive(Component)]
#[require(Wall { enabled: true})]
pub struct Water {}

#[derive(Component)]
pub struct Bridge {}

#[derive(Resource, Default)]
pub struct WallGrid {
    pub walls: HashSet<IVec2>,
}

pub fn setup_walls_system(mut commands: Commands) {
    commands.insert_resource(WallGrid {
        walls: HashSet::new(),
    });
}

pub fn gather_walls_system(
    wall_entities: Query<(&Transform, &Wall)>,
    mut wall_grid: ResMut<WallGrid>,
) {
    wall_grid.walls.clear();
    for (wall_transform, wall) in wall_entities.iter() {
        if !wall.enabled {
            continue;
        }
        let wall_square = wall_transform.translation.xz().round().as_ivec2();
        wall_grid.walls.insert(wall_square);
    }
}

pub fn move_player_system(
    time: Res<Time>,
    mut players: Query<(
        &Transform,
        &mut Player,
        &mut avian3d::prelude::LinearVelocity,
    )>,
    camera: Query<&Transform, (With<BillboardCamera>, Without<Player>)>,
    key: Res<ButtonInput<KeyCode>>,
    wall_grid: Res<WallGrid>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };

    let forward = (camera.forward().normalize() * Vec3::new(1., 0., 1.)).normalize_or_zero();
    let right = -Vec3::Y.cross(forward);
    let dt = time.delta_secs();
    for (player_transform, mut player, mut player_velocity) in players.iter_mut() {
        let mut target_velocity = Vec3::ZERO;
        if key.pressed(KeyCode::KeyD) {
            player.facing_direction = 1.;
            target_velocity += right;
        }
        if key.pressed(KeyCode::KeyA) {
            player.facing_direction = -1.;
            target_velocity -= right;
        }
        if key.pressed(KeyCode::KeyW) {
            target_velocity += forward;
        }
        if key.pressed(KeyCode::KeyS) {
            target_velocity -= forward;
        }
        target_velocity *= 3.5;

        player.recent_velocity = player
            .recent_velocity
            .lerp(target_velocity.clamp_length(0., 1.), (dt * 6.).min(1.));
        if player.recent_velocity.length() < 0.25
            && target_velocity
                .normalize_or_zero()
                .dot(player.recent_velocity.normalize_or_zero())
                > -0.1
        {
            player.recent_velocity = player.recent_velocity.normalize_or_zero() * 0.25;
        }

        // Find nearby walls and push the player out of them.
        let mut delta_push = Vec2::ZERO;
        let player_radius = 0.35;
        let block_half_size = 0.5;

        player.velocity = player.velocity.lerp(target_velocity, (dt * 12.).min(1.));

        let current_velocity = player.velocity;
        player.cursor += dt * current_velocity * 6.; // Update faster, so it leads the player.
        if player.cursor.distance(player_transform.translation) > 1. {
            player.cursor = player_transform.translation.move_towards(player.cursor, 1.);
        }

        // player_transform.translation.x += delta_push.x;
        // player_transform.translation.z += delta_push.y;

        player_velocity.0.x = player.velocity.x;
        player_velocity.0.z = player.velocity.z;
    }
}

pub fn move_camera_system(
    mut camera: Query<&mut Transform, With<BillboardCamera>>,
    player: Query<&Transform, (With<Player>, Without<BillboardCamera>)>,
) {
    let Ok(player) = player.single() else {
        return;
    };
    for mut camera in camera.iter_mut() {
        *camera = Transform::from_translation(player.translation + Vec3::new(0., 8.5, 9.5))
            .looking_at(player.translation, Vec3::Y);
    }
}
