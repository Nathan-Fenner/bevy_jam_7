use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

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
    pub glued: Vec<IVec2>,
    pub is_held: Option<IVec2>,
}

#[derive(Component)]
pub struct GrabIcon;

#[derive(Component)]
pub struct PointIcon;

#[derive(Resource)]
pub struct GrabIconEntity(Entity);

#[derive(Resource)]
pub struct PointIconEntity(Vec<Entity>);

pub fn setup_grab_system(mut commands: Commands) {
    let grab_icon = commands
        .spawn((
            GrabIcon,
            Billboard {
                image: "grab_icon.png".to_string(),
            },
            Transform::from_scale(Vec3::splat(1.)),
        ))
        .id();
    commands.insert_resource(GrabIconEntity(grab_icon));

    let point_icons = (0..10)
        .map(|_| {
            commands
                .spawn((
                    PointIcon,
                    Billboard {
                        image: "point_icon.png".to_string(),
                    },
                    Transform::from_scale(Vec3::splat(0.)),
                ))
                .id()
        })
        .collect::<Vec<Entity>>();
    commands.insert_resource(PointIconEntity(point_icons));
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
        if item.is_held.is_some() {
            continue;
        }
        item_blocked_squares.insert(item_transform.translation.xz().round().as_ivec2());
    }

    #[derive(Copy, Clone)]
    struct ItemType {
        entity: Entity,
        is_wall: bool,
    }

    let mut cursor_place_offsets: HashMap<IVec2, ItemType> = HashMap::new();
    let mut ground_items: HashMap<IVec2, ItemType> = HashMap::new();
    for (item_entity, item_transform, item) in items.iter() {
        if let Some(hold_offset) = item.is_held {
            cursor_place_offsets.insert(
                hold_offset,
                ItemType {
                    entity: item_entity,
                    is_wall: is_wall.contains(item_entity),
                },
            );
        } else {
            ground_items.insert(
                item_transform.translation.xz().round().as_ivec2(),
                ItemType {
                    entity: item_entity,
                    is_wall: is_wall.contains(item_entity),
                },
            );
        }
    }

    let mut can_place_item = true;
    for (&item_offset, item_type) in cursor_place_offsets.iter() {
        let place_at = player_cursor + item_offset;
        if walls.walls.contains(&place_at) {
            // The held item is blocked by a wall.
            can_place_item = false;
            break;
        }
        if item_blocked_squares.contains(&place_at) {
            // Another item is at this location.
            can_place_item = false;
            break;
        }
        if item_type.is_wall
            && place_at
                .as_vec2()
                .distance(player_transform.translation.xz())
                <= 0.5
        {
            // Would place a wall on top of the player.
            can_place_item = false;
            break;
        }
    }

    // Point to placement squares.
    let mut set_point_icon_index = 0;
    if can_place_item {
        let mut keys_sorted = cursor_place_offsets.keys().copied().collect::<Vec<IVec2>>();
        keys_sorted.sort_by_key(|v| (v.x, v.y));
        for hold_offset in keys_sorted {
            let point_icon_entity = point_icon.0[set_point_icon_index];
            set_point_icon_index += 1;

            let icon_transform = &mut *arbitrary_transform.get_mut(point_icon_entity).unwrap();
            icon_transform.scale = if hold_offset == IVec2::ZERO {
                Vec3::splat(1.)
            } else {
                Vec3::splat(0.5)
            };
            let target_position = Vec3::new(
                player_cursor.x as f32 + hold_offset.x as f32,
                0.,
                player_cursor.y as f32 + hold_offset.y as f32,
            ) + Vec3::Y * 0.5;

            if icon_transform.translation.distance(target_position) > 3.7 {
                icon_transform.translation = target_position;
            } else {
                icon_transform.translation = icon_transform
                    .translation
                    .lerp(target_position, (15. * dt).min(1.));
            }
        }
    }

    for (item_entity, mut item_transform, mut item) in items.iter_mut() {
        if let Some(hold_offset) = item.is_held {
            is_holding = true;
            item_transform.translation = item_transform.translation.lerp(
                player_transform.translation
                    + Vec3::Y * 0.75
                    + Vec3::new(hold_offset.x as f32, 0., hold_offset.y as f32),
                (dt * 15.).min(1.),
            );

            // Attempt to place at the cursor position, assuming there is room.
            if can_place_item {
                let place_at = player_cursor + hold_offset;

                if key.just_pressed(KeyCode::KeyE) {
                    item_transform.translation =
                        Vec3::Y * 0.5 + Vec3::new(place_at.x as f32, 0., place_at.y as f32);
                    item.is_held = None;

                    if let Ok(mut wall) = is_wall.get_mut(item_entity) {
                        wall.enabled = true;
                    }
                }
            }
        }
    }

    let mut set_icon_grab = false;

    struct PickUp {
        cursor_offsets: Vec<IVec2>,
    }

    let mut to_pick_up: Option<PickUp> = None;
    for (_item_entity, item_transform, item) in items.iter_mut() {
        if item_transform.translation.xz().round().as_ivec2() == player_cursor {
            if !is_holding {
                *arbitrary_transform.get_mut(grab_icon.0).unwrap() =
                    Transform::from_translation(item_transform.translation + Vec3::Y * 0.75);
                set_icon_grab = true;
            }

            if !is_holding && key.just_pressed(KeyCode::KeyE) {
                let mut cursor_offsets = item.glued.clone();
                cursor_offsets.push(IVec2::ZERO);
                to_pick_up = Some(PickUp { cursor_offsets });
            }
        }
    }

    if let Some(to_pick_up) = to_pick_up {
        // Pick up all of the items glued to this one.
        for &glue_offset in &to_pick_up.cursor_offsets {
            let glued_item = ground_items.get(&(player_cursor + glue_offset)).unwrap();

            let (_, _, mut item) = items.get_mut(glued_item.entity).unwrap();
            item.is_held = Some(glue_offset);
            if let Ok(mut wall) = is_wall.get_mut(glued_item.entity) {
                // Disable the wall while it is being carried.
                wall.enabled = false;
            }
        }
    }

    if !set_icon_grab {
        let scale = &mut arbitrary_transform.get_mut(grab_icon.0).unwrap().scale;
        *scale = Vec3::ZERO;
    }
    for j in set_point_icon_index..point_icon.0.len() {
        let point_icon_entity = point_icon.0[j];
        arbitrary_transform
            .get_mut(point_icon_entity)
            .unwrap()
            .scale = Vec3::splat(0.);
    }
}
