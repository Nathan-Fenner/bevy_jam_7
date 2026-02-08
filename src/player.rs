use bevy::prelude::*;

use crate::billboard::BillboardCamera;
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, move_player_system);
    }
}

#[derive(Component, Debug, Default)]
pub struct Player {
    pub velocity: Vec3,
    pub facing_direction: f32,
}

pub fn move_player_system(
    time: Res<Time>,
    mut players: Query<(&mut Transform, &mut Player)>,
    camera: Query<&Transform, (With<BillboardCamera>, Without<Player>)>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let forward = (camera.forward().normalize() * Vec3::new(1., 0., 1.)).normalize_or_zero();
    let right = -Vec3::Y.cross(forward);
    let dt = time.delta_secs();
    for (mut player_transform, mut player) in players.iter_mut() {
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
        target_velocity *= 5.5;

        player.velocity = player.velocity.lerp(target_velocity, (dt * 12.).min(1.));
        player_transform.translation += dt * player.velocity;
    }
}
