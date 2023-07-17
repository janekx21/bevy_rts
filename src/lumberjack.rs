use crate::unit::{SelectedMark, SelectionBox, Unit};
use crate::util::{find_nearest, nearest_entity};
use crate::{Barrack, Cull2D, Cursor, DepositWoodEvent, SpriteSheets, Tree, TreeChopEvent, YSort};
use bevy::prelude::*;

pub struct LumberjackPlugin;

impl Plugin for LumberjackPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnLumberjackEvent>()
            .add_system(lumberjack_spawning)
            .add_system(lumberjack_animation)
            .add_system(lumberjack_next_action)
            .add_system(lumberjack_move_to_position_action);
    }
}

#[derive(Component, Default, Reflect)]
pub struct Lumberjack {
    action: Action,
    wood: u32,
    animation_timer: f32,
}

#[derive(Default, Reflect)]
pub enum Action {
    #[default]
    Idle,
    MoveToPosition(Vec2),
    CollectResource(Entity),
    DepositResource(Entity),
    Chop {
        timeout: f32,
        target: Entity,
    },
}

pub struct SpawnLumberjackEvent(pub Vec2);

pub fn lumberjack_spawning(
    mut commands: Commands,
    mut events: EventReader<SpawnLumberjackEvent>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for event in events.iter() {
        let pos = event.0;
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: sprite_sheets.farmer_red.clone(),
                transform: Transform::from_translation(pos.extend(1.0)),
                ..default()
            })
            .insert(Name::new("Luberjack"))
            .insert(YSort)
            .insert(Cull2D)
            .insert(Unit::default())
            .insert(Lumberjack::default())
            .with_children(|builder| {
                builder
                    .spawn(SpriteSheetBundle {
                        texture_atlas: sprite_sheets.box_selector.clone(),
                        visibility: Visibility::Hidden,
                        ..default()
                    })
                    .insert(SelectionBox);
            });
    }
}

pub fn lumberjack_animation(mut query: Query<(&Unit, &Lumberjack, &mut TextureAtlasSprite)>) {
    for (unit, worker, mut sprite) in query.iter_mut() {
        let frame = (worker.animation_timer * 8.0).round() as usize;
        let direction = match unit.last_direction {
            Vec2 { x, y } if x > y && -x > y => 0,    // down
            Vec2 { x, y } if x > -y && -x > -y => 5,  // up
            Vec2 { x, y } if x > y && x > -y => 10,   // right
            Vec2 { x, y } if -x > y && -x > -y => 15, // left
            _ => 0,                                   // no direction -> down
        };

        sprite.index = match worker.action {
            Action::Chop { timeout, target: _ } => {
                let animation_frame = (timeout.clamp(0.0, 1.0) * 3.0).floor() as usize;
                40 + direction + animation_frame
            }
            _ if unit.vel.length() > 20.0 => {
                direction + (frame % 4 + 1) + if worker.wood > 0 { 20 } else { 0 }
            }
            _ => direction + if worker.wood > 0 { 20 } else { 0 },
        };
    }
}

const BARACK_SIZE: f32 = 20.0;

pub fn lumberjack_next_action(
    mut query: Query<(&mut Lumberjack, &mut Unit, &Transform)>,
    barrack_query: Query<(Entity, &Transform), (With<Barrack>, Without<Unit>)>,
    tree_query: Query<(Entity, &Transform, &Tree), Without<Unit>>,
    mut tree_chop_event: EventWriter<TreeChopEvent>,
    _entity_query: Query<Entity>,
    mut deposit_wood: EventWriter<DepositWoodEvent>,
    time: Res<Time>,
) {
    for (mut worker, mut unit, transform) in query.iter_mut() {
        worker.animation_timer += time.delta_seconds();
        let pos = transform.translation.truncate();
        match worker.action {
            Action::Idle => {
                unit.target_direction = Vec2::ZERO;
                if worker.wood >= 5 {
                    // can carry 5 wood
                    worker.action = find_nearest(barrack_query.iter(), pos)
                        .map(|f| f.0)
                        .map_or(Action::Idle, Action::DepositResource);
                } else {
                    worker.action = tree_query
                        .iter()
                        .fold(None, |acc, (a, b, _c)| {
                            nearest_entity(acc, pos, (a, b.translation.truncate()))
                        })
                        .map(|f| f.0)
                        .map_or(Action::Idle, Action::CollectResource);
                }
            }
            Action::MoveToPosition(target_pos) => {
                let delta = target_pos - pos;
                unit.target_direction = delta;
                if delta.length_squared() < 25.0 * 25.0 {
                    worker.action = Action::Idle;
                    worker.animation_timer = 0.0;
                }
            }
            Action::CollectResource(target) => {
                match tree_query.get(target) {
                    Ok((tree_entity, tree_transform, tree)) if tree.resource >= 0 => {
                        // move towards tree
                        let target_pos = tree_transform.translation.truncate();
                        unit.target_direction = (target_pos - pos).normalize();
                        if Vec2::distance_squared(target_pos, pos) < 10.0 * 10.0 {
                            worker.action = Action::Chop {
                                timeout: 1.0,
                                target: tree_entity,
                            };
                        }
                    }
                    _ => worker.action = Action::Idle,
                }
            }
            Action::DepositResource(target) => {
                match barrack_query.get_component::<Transform>(target) {
                    Ok(barrack_transform) => {
                        // move towards barrack
                        let target_pos = barrack_transform.translation.truncate();
                        unit.target_direction = (target_pos - pos).normalize();
                        if Vec2::distance_squared(target_pos, pos) < BARACK_SIZE * BARACK_SIZE {
                            // found target
                            worker.wood = 0;
                            worker.action = Action::Idle;
                            worker.animation_timer = 0.0;
                            deposit_wood.send(DepositWoodEvent(1))
                        }
                    }
                    _ => worker.action = Action::Idle,
                }
            }
            Action::Chop { timeout, target } => {
                unit.target_direction = Vec2::ZERO;
                if timeout > 0.0 {
                    worker.action = Action::Chop {
                        timeout: timeout - time.delta_seconds() * (8.0 / 3.0),
                        target,
                    };
                } else {
                    if let Ok((tree_entity, _tree_transform, tree)) = tree_query.get(target) {
                        if tree.resource >= 0 {
                            tree_chop_event.send(TreeChopEvent(tree_entity));
                            worker.wood += 1;
                        }
                    }
                    worker.action = Action::Idle;
                    worker.animation_timer = 0.0;
                }
            }
        }
    }
}

pub fn lumberjack_move_to_position_action(
    mut query: Query<(&Unit, &mut Lumberjack), With<SelectedMark>>,
    input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
) {
    if input.just_pressed(MouseButton::Right) {
        for (_, mut worker) in query.iter_mut() {
            worker.action = Action::MoveToPosition(cursor.0);
        }
    }
}
