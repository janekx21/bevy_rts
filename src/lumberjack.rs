use crate::unit::{SelectedMark, SelectionBox, Unit};
use crate::util::find_nearest;
use crate::{Barrack, Cull2D, Cursor, DepositWoodEvent, Tree, TreeChopEvent, YSort};
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Lumberjack {
    action: Action,
    wood: u32,
    animation_timer: f32,
}

#[derive(Default)]
pub enum Action {
    #[default]
    Idle,
    MoveToPosition(Vec2),
    CollectResource(Entity),
    DepositResource(Entity),
    Chop(f32),
}

pub fn lumberjack_spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    pos: Vec2,
) {
    let texture_handle = asset_server.load("farmer_red.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 5, 12, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let box_selector = {
        let texture_handle = asset_server.load("box_selector.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 2, 1, None, None);
        texture_atlases.add(texture_atlas)
    };

    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        })
        .insert(YSort)
        .insert(Cull2D)
        .insert(Unit::default())
        .insert(Lumberjack::default())
        .with_children(|builder| {
            builder
                .spawn(SpriteSheetBundle {
                    texture_atlas: box_selector,
                    visibility: Visibility::Hidden,
                    ..default()
                })
                .insert(SelectionBox);
        });
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

        (*sprite).index = match worker.action {
            Action::Chop(timer) => {
                let animation_frame = (timer.clamp(0.0, 1.0) * 3.0).floor() as usize;
                40 + direction + animation_frame
            }
            _ if unit.vel.length() > 20.0 => {
                direction + (frame % 4 + 1) + if worker.wood > 0 { 20 } else { 0 }
            }
            _ => direction + if worker.wood > 0 { 20 } else { 0 },
        };
    }
}

pub fn lumberjack_next_action(
    mut query: Query<(&mut Lumberjack, &mut Unit, &Transform)>,
    barrack_query: Query<(Entity, &Transform), (With<Barrack>, Without<Unit>)>,
    tree_query: Query<(Entity, &Transform), (With<Tree>, Without<Unit>)>,
    mut tree_chop_event: EventWriter<TreeChopEvent>,
    entity_query: Query<Entity>,
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
                    worker.action = find_nearest(tree_query.iter(), pos)
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
                if let Ok((tree_entity, tree_transform)) = tree_query.get(target) {
                    // move towards tree
                    let target_pos = tree_transform.translation.truncate();
                    unit.target_direction = (target_pos - pos).normalize();
                    if Vec2::distance_squared(target_pos, pos) < 10.0 * 10.0 {
                        tree_chop_event.send(TreeChopEvent(tree_entity));
                        worker.action = Action::Chop(1.0);
                    }
                } else if entity_query.get(target).is_err() {
                    worker.action = Action::Idle;
                    worker.animation_timer = 0.0;
                }
            }
            Action::DepositResource(target) => {
                if let Ok(barrack_transform) = barrack_query.get_component::<Transform>(target) {
                    // move towards barrack
                    let target_pos = barrack_transform.translation.truncate();
                    unit.target_direction = (target_pos - pos).normalize();
                    if Vec2::distance_squared(target_pos, pos) < 20.0 * 20.0 {
                        // found target
                        worker.wood = 0;
                        worker.action = Action::Idle;
                        worker.animation_timer = 0.0;
                        deposit_wood.send(DepositWoodEvent(1))
                    }
                }
            }
            Action::Chop(timeout) => {
                unit.target_direction = Vec2::ZERO;
                if timeout > 0.0 {
                    worker.action = Action::Chop(timeout - time.delta_seconds() * (8.0 / 3.0));
                } else {
                    worker.wood += 1;
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
