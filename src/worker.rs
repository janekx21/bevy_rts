use crate::util::{find_nearest, random_vec2};
use crate::worker::Action::{CollectResource, DepositResource, Idle, MoveToPosition};
use crate::{ApplySelection, Barrack, Cursor, DepositWood, Tree, TreeChop};
use bevy::prelude::*;

pub enum Action {
    Idle,
    MoveToPosition(Vec2),
    CollectResource(Entity),
    DepositResource(Entity),
}

#[derive(Component)]
pub struct Worker {
    action: Action,
    wood: u32,
    is_selected: bool,
    next_move: Vec2,
}

#[derive(Component)]
pub struct SelectionBox;

pub fn worker_select(
    mut apply_selection: EventReader<ApplySelection>,
    mut query: Query<(&Transform, &mut Worker)>,
) {
    for event in apply_selection.iter() {
        let min = Vec2::min(event.start, event.end);
        let max = Vec2::max(event.start, event.end);

        for (transform, mut worker) in query.iter_mut() {
            let p = transform.translation.truncate();
            let inside = p.x > min.x && p.x < max.x && p.y > min.y && p.y < max.y;
            worker.is_selected = inside;
        }
    }
}

pub fn worker_selection_box_visible(
    mut child_query: Query<(&Parent, &mut Visibility), With<SelectionBox>>,
    parent_query: Query<&Worker>,
) {
    for (par, mut vis) in child_query.iter_mut() {
        if let Ok(parent) = parent_query.get(par.get()) {
            *vis = if parent.is_selected {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn worker_spawn(
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

    let selector = commands
        .spawn(SpriteSheetBundle {
            texture_atlas: box_selector,
            ..default()
        })
        .insert(SelectionBox)
        .id();

    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        })
        .insert(Worker {
            action: Idle,
            wood: 0,
            is_selected: false,
            next_move: Vec2::ZERO,
        })
        .add_child(selector);
}

pub fn worker_next_action(
    mut worker_query: Query<(&mut Worker, &Transform)>,
    barrack_query: Query<(Entity, &Transform), (With<Barrack>, Without<Worker>)>,
    tree_query: Query<(Entity, &Transform), (With<Tree>, Without<Worker>)>,
    mut tree_chop_event: EventWriter<TreeChop>,
    entity_query: Query<Entity>,
    mut deposit_wood: EventWriter<DepositWood>,
) {
    for (mut worker, transform) in worker_query.iter_mut() {
        let worker_pos = transform.translation.truncate();
        match worker.action {
            Idle => {
                worker.next_move = Vec2::ZERO;
                if worker.wood > 0 {
                    worker.action = find_nearest(barrack_query.iter(), worker_pos)
                        .map(|f| f.0)
                        .map_or(Idle, DepositResource);
                } else {
                    worker.action = find_nearest(tree_query.iter(), worker_pos)
                        .map(|f| f.0)
                        .map_or(Idle, CollectResource);
                }
            }
            Action::MoveToPosition(pos) => {
                worker.next_move = pos - worker_pos;
            }
            CollectResource(target) => {
                if let Ok((tree_entity, tree_transform)) = tree_query.get(target) {
                    // move towards tree
                    let target_pos = tree_transform.translation.truncate();
                    worker.next_move = (target_pos - worker_pos).normalize();
                    if Vec2::distance_squared(target_pos, worker_pos) < 10.0 * 10.0 {
                        tree_chop_event.send(TreeChop(tree_entity));
                        worker.wood = 1;
                        worker.action = Idle;
                    }
                } else if entity_query.get(target).is_err() {
                    worker.action = Idle;
                }
            }
            DepositResource(target) => {
                if let Ok(barrack_transform) = barrack_query.get_component::<Transform>(target) {
                    // move towards barrack
                    let target_pos = barrack_transform.translation.truncate();
                    worker.next_move = (target_pos - worker_pos).normalize();
                    if Vec2::distance_squared(target_pos, worker_pos) < 20.0 * 20.0 {
                        // found target
                        worker.wood = 0;
                        worker.action = Idle;
                        deposit_wood.send(DepositWood(1))
                    }
                }
            }
        }
    }
}

pub fn worker_move(mut query: Query<(&mut Transform, &Worker)>, time: Res<Time>) {
    for (mut transform, worker) in query.iter_mut() {
        transform.translation += (random_vec2() * 0.1 + worker.next_move.clamp_length_max(1.0))
            .extend(0.0)
            * time.delta_seconds()
            * 60.0;
    }
}

pub fn move_to_position(
    mut query: Query<&mut Worker>,
    input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
) {
    if input.just_pressed(MouseButton::Right) {
        for mut worker in query.iter_mut().filter(|w| w.is_selected) {
            worker.action = MoveToPosition(cursor.0);
        }
    }
}
