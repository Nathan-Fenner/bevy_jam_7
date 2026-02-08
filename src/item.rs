use bevy::prelude::*;

use crate::player::{Player, Wall, WallGrid, gather_walls_system};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, grab_item_system.after(gather_walls_system));
    }
}

#[derive(Component)]
pub struct Item {
    pub is_held: bool,
}

pub fn grab_item_system(
    time: Res<Time>,
    mut items: Query<(Entity, &mut Transform, &mut Item)>,
    player: Query<(&Transform, &Player), Without<Item>>,
    mut gizmos: Gizmos,
    key: Res<ButtonInput<KeyCode>>,
    walls: Res<WallGrid>,
    mut is_wall: Query<&mut Wall>,
) {
    let dt = time.delta_secs();

    let Ok((player_transform, player)) = player.single() else {
        return;
    };

    let player_cursor = player.cursor.round().xz().as_ivec2();

    let mut is_holding = false;

    for (item_entity, mut item_transform, mut item) in items.iter_mut() {
        if item.is_held {
            is_holding = true;
            item_transform.translation = item_transform.translation.lerp(
                player_transform.translation + Vec3::Y * 0.75,
                (dt * 15.).min(1.),
            );
            if key.just_pressed(KeyCode::KeyE) {
                // Attempt to place at the cursor position, assuming there is room.
                if !walls.walls.contains(&player_cursor) {
                    // If it is a wall, do not allow it placed on top of the player.
                    if player_cursor
                        .as_vec2()
                        .distance(player_transform.translation.xz())
                        > 0.5
                    {
                        item_transform.translation = Vec3::Y * 0.5
                            + Vec3::new(player_cursor.x as f32, 0., player_cursor.y as f32);
                        item.is_held = false;

                        if let Ok(mut wall) = is_wall.get_mut(item_entity) {
                            wall.enabled = true;
                        }
                    }
                }
            }
        }
    }

    for (item_entity, item_transform, mut item) in items.iter_mut() {
        if item_transform.translation.xz().round().as_ivec2() == player_cursor {
            gizmos.line(
                item_transform.translation - Vec3::Y,
                item_transform.translation + Vec3::Y,
                Color::linear_rgb(1., 0., 0.),
            );

            if !is_holding && key.just_pressed(KeyCode::KeyE) {
                item.is_held = true;

                if let Ok(mut wall) = is_wall.get_mut(item_entity) {
                    // Disable the wall while it is being carried.
                    wall.enabled = false;
                }
            }
        }
    }
}
