use std::collections::VecDeque;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::player::{Player, Wall, Water};

#[derive(Component)]
pub struct Blueprint;

#[derive(Component)]
pub struct Door;

pub struct BlueprintPlugin;

impl Plugin for BlueprintPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_blueprints_system)
            .add_systems(
                Update,
                (
                    find_active_blueprint_system,
                    process_blueprint_system,
                    show_blueprint_ui_system,
                )
                    .chain(),
            );
    }
}

pub struct BlueprintLine {
    pub container_entity: Entity,
    pub text_entity: Entity,
}

#[derive(Resource)]
pub struct BlueprintUi {
    pub container_entity: Entity,
    pub lines: Vec<BlueprintLine>,
}

pub fn setup_blueprints_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/delius/Delius-Regular.ttf");
    commands.spawn((
        Camera2d,
        Camera {
            order: 10,
            ..default()
        },
    ));

    let root_uinode = commands
        .spawn(Node {
            width: percent(100),
            height: percent(100),
            ..default()
        })
        .id();

    let mut blueprint_uis: Vec<BlueprintLine> = Vec::new();

    let left_column = commands
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            width: px(600),
            margin: UiRect::axes(px(15), px(5)),
            row_gap: px(10),
            ..default()
        },))
        .with_children(|builder| {
            for _ in 0..8 {
                let mut text_entity = None;
                let container_entity = builder
                    .spawn((
                        Node {
                            padding: UiRect::axes(px(15), px(15)),
                            ..default()
                        },
                        Visibility::Inherited,
                        BackgroundColor(Color::linear_rgba(0.05, 0.05, 0.15, 0.6)),
                    ))
                    .with_children(|builder| {
                        text_entity = Some(
                            builder
                                .spawn((
                                    Text::new("This is\nmultiline text"),
                                    TextColor(Color::linear_rgb(1., 1., 1.)),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 25.0,
                                        ..default()
                                    },
                                ))
                                .id(),
                        );
                    })
                    .id();

                blueprint_uis.push(BlueprintLine {
                    container_entity,
                    text_entity: text_entity.unwrap(),
                });
            }
        })
        .id();

    commands.entity(root_uinode).add_children(&[left_column]);

    commands.insert_resource(BlueprintUi {
        lines: blueprint_uis,
        container_entity: left_column,
    });
    commands.insert_resource(ActiveBlueprint {
        active_blueprint: None,
    });
}

#[derive(Resource)]
pub struct ActiveBlueprint {
    pub active_blueprint: Option<BlueprintInfo>,
}

#[derive(PartialEq, Eq)]
pub struct BlueprintInfo {
    blueprint_location: IVec2,
    blueprint_entity: Entity,
}

pub fn find_active_blueprint_system(
    player: Query<&Transform, With<Player>>,
    blueprints: Query<(Entity, &Transform), With<Blueprint>>,
    mut active_blueprint: ResMut<ActiveBlueprint>,
) {
    let Ok(player) = player.single() else {
        return;
    };

    let mut target_info = None;

    for (blueprint_entity, blueprint_transform) in blueprints.iter() {
        if blueprint_transform
            .translation
            .xz()
            .distance(player.translation.xz())
            < 2.4
        {
            target_info = Some(BlueprintInfo {
                blueprint_location: blueprint_transform.translation.xz().round().as_ivec2(),
                blueprint_entity,
            });
        }
    }

    if active_blueprint.active_blueprint != target_info {
        active_blueprint.active_blueprint = target_info;
    }
}

pub fn process_blueprint_system(
    active_blueprint: Res<ActiveBlueprint>,
    q_water: Query<(&Transform, &Water)>,
    q_wall: Query<(&Transform, &Wall)>,
    q_door: Query<(&Transform, &Door)>,
    mut gizmos: Gizmos,
) {
    let Some(active_blueprint) = active_blueprint.active_blueprint.as_ref() else {
        return;
    };

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    enum GridType {
        Wall,
        Water,
        Door,
    }

    let mut grid: HashMap<IVec2, GridType> = HashMap::new();
    fn round(t: &Transform) -> IVec2 {
        t.translation.xz().round().as_ivec2()
    }
    for (t, _water) in q_water.iter() {
        grid.insert(round(&t), GridType::Water);
    }
    for (t, wall) in q_wall.iter() {
        if wall.enabled {
            grid.insert(round(&t), GridType::Wall);
        }
    }
    for (t, _door) in q_door.iter() {
        grid.insert(round(&t), GridType::Door);
    }

    let mut reachable_queue: VecDeque<IVec2> = VecDeque::new();
    let mut reachable_from: HashMap<IVec2, IVec2> = HashMap::new();

    reachable_from.insert(
        active_blueprint.blueprint_location,
        active_blueprint.blueprint_location,
    );
    reachable_queue.push_back(active_blueprint.blueprint_location);

    let mut bounded_by_water = false;
    let mut has_door = false;
    let mut too_big = false;

    let mut bad_pos: Option<IVec2> = None;

    while let Some(current) = reachable_queue.pop_front() {
        if reachable_from.len() > 300 {
            bad_pos = Some(current);
            too_big = true;
            break;
        }
        for dir in [IVec2::X, IVec2::Y, IVec2::NEG_X, IVec2::NEG_Y] {
            let neighbor = current + dir;
            if reachable_from.contains_key(&neighbor) {
                continue;
            }
            reachable_from.insert(neighbor, current);
            let neighbor_cell = grid.get(&neighbor).copied();
            if neighbor_cell == Some(GridType::Water) {
                bad_pos = Some(neighbor);
                bounded_by_water = true;
                break;
            }
            if neighbor_cell == Some(GridType::Door) {
                // TODO: make sure it goes outside
                has_door = true;
                continue;
            }
            if neighbor_cell == Some(GridType::Wall) {
                continue;
            }

            reachable_queue.push_back(neighbor);
        }

        if bad_pos.is_some() {
            break;
        }
    }

    let good = has_door && !bounded_by_water && !too_big;

    let mut bad_path: Vec<IVec2> = Vec::new();
    while let Some(path_pos) = bad_pos {
        if path_pos == active_blueprint.blueprint_location {
            break;
        }
        bad_path.push(path_pos);
        bad_pos = reachable_from.get(&path_pos).copied();
    }

    if good {
        let color = Color::linear_rgb(0., 0., 1.);
        for p in reachable_from.keys() {
            let p = Vec3::new(p.x as f32, 0., p.y as f32);
            gizmos.line(p, p + Vec3::Y * 6., color);
        }
    } else {
        let color = Color::linear_rgb(1., 0., 0.);
        for p in &bad_path {
            let p = Vec3::new(p.x as f32, 0., p.y as f32);
            gizmos.line(p, p + Vec3::Y * 6., color);
        }
    }
}

pub fn show_blueprint_ui_system(
    time: Res<Time>,
    active_blueprint: Res<ActiveBlueprint>,
    ui: Res<BlueprintUi>,
    mut nodes: Query<&mut Node>,
) {
    let dt = time.delta_secs();
    // let Some(active_blueprint) = active_blueprint.active_blueprint.as_ref() else {
    //     return;
    // };
    let is_visible = active_blueprint.active_blueprint.is_some();

    let target_margin = if is_visible { 10. } else { -800. };

    let mut container = nodes.get_mut(ui.container_entity).unwrap();

    container.margin.left = match container.margin.left {
        Val::Px(value) => Val::Px(value.lerp(target_margin, (10. * dt).min(1.))),
        _ => Val::Px(target_margin),
    };
}
