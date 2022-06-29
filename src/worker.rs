use bevy::prelude::*;
use crate::{ApplySelection, Barrack, Tree, TreeChop};
use crate::util::{find_nearest, random_vec2};

#[derive(Component)]
pub struct Worker {
    target: Option<Entity>,
    wood: u32,
    is_selected: bool,
}

#[derive(Component)]
pub struct SelectionBox;

pub fn worker_selecter(mut apply_selection: EventReader<ApplySelection>, mut query: Query<(&Transform, &mut Worker)>) {
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

pub fn worker_selection(mut child_query: Query<(&Parent, &mut Visibility), With<SelectionBox>>, parent_query: Query<&Worker>) {
    for (par, mut vis) in child_query.iter_mut() {
        if let Ok(parent) = parent_query.get(par.0) {
            vis.is_visible = parent.is_selected;
        }
    }
}

pub fn spawn_worker(mut commands: &mut Commands,
                    asset_server: &Res<AssetServer>,
                    mut texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
                    pos: Vec2
) {
    let texture_handle = asset_server.load("farmer_red.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 5, 12);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let box_selector = {
        let texture_handle = asset_server.load("box_selector.png");
        let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 2, 1);
        texture_atlases.add(texture_atlas)
    };

    let selector = commands.spawn_bundle(
        SpriteSheetBundle {
            texture_atlas: box_selector.clone(),
            ..default()
        }
    ).insert(SelectionBox).id();

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_translation(pos.extend(1.0)),
            ..default()
        })
        .insert(Worker{target: None, wood: 0, is_selected: false})
        .add_child(selector);
}

pub fn move_workers(
    mut worker_query: Query<(&mut Worker, &mut Transform)>,
    barrack_query: Query<(Entity, &Transform), (With<Barrack>, Without<Worker>)>,
    tree_query: Query<(Entity, &Transform), (With<Tree>, Without<Worker>)>,
    mut tree_chop_event: EventWriter<TreeChop>,
    entity_query: Query<Entity>
) {
    for (mut worker, mut transform) in worker_query.iter_mut() {
        let worker_pos = transform.translation.truncate();
        if let Some(target) = worker.target {
            let mut delta = Vec2::ZERO;
            if let Ok(barrack_transform) = barrack_query.get_component::<Transform>(target) {
                // move towards barrack
                delta = barrack_transform.translation.truncate() - worker_pos;
                if delta.length() < 10.0 {
                    // found target
                    worker.target = None;
                    worker.wood = 0;
                }
            }
            if let Ok((tree_entity, tree_transform)) = tree_query.get(target) {
                // move towards tree
                delta = tree_transform.translation.truncate() - worker_pos;
                if delta.length() < 10.0 {
                    worker.wood = 10;
                    tree_chop_event.send(TreeChop(tree_entity));
                    worker.target = None;
                }
            }
            transform.translation += (random_vec2() * 0.1 + delta.clamp_length_max(1.0)).extend(0.0);
            if let Err(_) = entity_query.get(target) {
                worker.target = None
            }
        } else {
            // no target

            if worker.wood > 0 {
                worker.target = find_nearest(barrack_query.iter().into(), worker_pos).map(|f|f.0);
            } else {
                worker.target = find_nearest(tree_query.iter().into(), worker_pos).map(|f|f.0);
            }
        }
    }
}
