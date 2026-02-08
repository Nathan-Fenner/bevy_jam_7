use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    billboard::Billboard,
    player::{Player, Wall, WallGrid, gather_walls_system},
};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_grab_system)
            .add_systems(Update, grab_item_system.after(gather_walls_system));
    }
}

#[derive(Component)]
pub struct Item {
    pub is_held: bool,
}

#[derive(Component)]
pub struct GrabIcon;

#[derive(Component)]
pub struct PointIcon;

#[derive(Resource)]
pub struct GrabIconEntity(Entity);

#[derive(Resource)]
pub struct PointIconEntity(Entity);

pub fn setup_grab_system(mut commands: Commands) {
    let grab_icon = commands
        .spawn((
            GrabIcon,
            Billboard {
                image: "grab_icon.png".to_string(),
            },
            Transform::from_scale(Vec3::splat(0.)),
        ))
        .id();
    commands.insert_resource(GrabIconEntity(grab_icon));

    let point_icon = commands
        .spawn((
            PointIcon,
            Billboard {
                image: "point_icon.png".to_string(),
            },
            Transform::from_scale(Vec3::splat(0.)),
        ))
        .id();
    commands.insert_resource(PointIconEntity(point_icon));
}

pub fn grab_item_system(
    time: Res<Time>,
    mut items: Query<(Entity, &mut Transform, &mut Item)>,
    player: Query<(&Transform, &Player), Without<Item>>,
    key: Res<ButtonInput<KeyCode>>,
    walls: Res<WallGrid>,
    mut is_wall: Query<&mut Wall>,
    grab_icon: Res<GrabIconEntity>,
    point_icon: Res<PointIconEntity>,
    mut arbitrary_transform: Query<&mut Transform, (Without<Player>, Without<Item>)>,
) {
    let dt = time.delta_secs();

    let Ok((player_transform, player)) = player.single() else {
        return;
    };

    let player_cursor = player.cursor.round().xz().as_ivec2();

    let mut is_holding = false;

    // Tracks the squares where items cannot be placed, because an item is already there.
    let mut item_blocked_squares: HashSet<IVec2> = HashSet::new();
    for (_item_entity, item_transform, item) in items.iter() {
        if item.is_held {
            continue;
        }
        item_blocked_squares.insert(item_transform.translation.xz().round().as_ivec2());
    }

    let mut set_icon_point = false;

    for (item_entity, mut item_transform, mut item) in items.iter_mut() {
        if item.is_held {
            is_holding = true;
            item_transform.translation = item_transform.translation.lerp(
                player_transform.translation + Vec3::Y * 0.75,
                (dt * 15.).min(1.),
            );

            // Attempt to place at the cursor position, assuming there is room.
            if !walls.walls.contains(&player_cursor)
                && !item_blocked_squares.contains(&player_cursor)
            {
                // If it is a wall, do not allow it placed on top of the player.
                if !is_wall.contains(item_entity)
                    || player_cursor
                        .as_vec2()
                        .distance(player_transform.translation.xz())
                        > 0.5
                {
                    *arbitrary_transform.get_mut(point_icon.0).unwrap() =
                        Transform::from_translation(
                            Vec3::new(player_cursor.x as f32, 0., player_cursor.y as f32)
                                + Vec3::Y * 0.5,
                        );
                    set_icon_point = true;

                    if key.just_pressed(KeyCode::KeyE) {
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

    let mut set_icon_grab = false;

    for (item_entity, item_transform, mut item) in items.iter_mut() {
        if item_transform.translation.xz().round().as_ivec2() == player_cursor {
            if !is_holding {
                *arbitrary_transform.get_mut(grab_icon.0).unwrap() =
                    Transform::from_translation(item_transform.translation + Vec3::Y * 0.75);
                set_icon_grab = true;
            }

            if !is_holding && key.just_pressed(KeyCode::KeyE) {
                item.is_held = true;

                if let Ok(mut wall) = is_wall.get_mut(item_entity) {
                    // Disable the wall while it is being carried.
                    wall.enabled = false;
                }
            }
        }
    }

    if !set_icon_grab {
        *arbitrary_transform.get_mut(grab_icon.0).unwrap() = Transform::from_scale(Vec3::splat(0.));
    }
    if !set_icon_point {
        *arbitrary_transform.get_mut(point_icon.0).unwrap() =
            Transform::from_scale(Vec3::splat(0.));
    }
}
