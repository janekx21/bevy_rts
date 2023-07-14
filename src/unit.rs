use crate::{selection_change, ApplySelectionEvent, UnitQuadTree};
use bevy::prelude::*;
use quadtree_rs::{area::AreaBuilder, point::Point};

pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(unit_push_apart)
            .add_system(unit_select)
            .add_system(unit_vel)
            .add_system(unit_move)
            .add_system(unit_quad_tree_placement)
            .add_system(selection_added)
            .add_system(selection_removed.after(selection_change));
    }
}

#[derive(Component, Default)]
pub struct Unit {
    pub vel: Vec2,
    pub target_direction: Vec2,
    pub last_direction: Vec2,
    pub point: Option<(Point<u32>, u64)>,
    pub hp: f32,
}

#[derive(Component)]
pub struct SelectionBox;

#[derive(Component)]
pub struct SelectedMark;

fn unit_quad_tree_placement(
    mut query: Query<(&Transform, &mut Unit, Entity)>,
    mut unit_quad_tree: ResMut<UnitQuadTree>,
) {
    for (transform, mut unit, entity) in query.iter_mut() {
        let unit_pos = transform.translation.truncate();
        let point_pos = pos_to_point(unit_pos);
        if let Some((current_point, current_handle)) = unit.point {
            if current_point != point_pos {
                unit.point = unit_quad_tree
                    .insert_pt(point_pos, entity)
                    .map(|handle| (point_pos, handle));
                if unit.point.is_some() {
                    unit_quad_tree.delete_by_handle(current_handle);
                }
            }
        } else {
            unit.point = unit_quad_tree
                .insert_pt(point_pos, entity)
                .map(|handle| (point_pos, handle));
        }
    }
}

fn unit_vel(mut query: Query<(&mut Transform, &Unit)>, time: Res<Time>) {
    query.par_iter_mut().for_each_mut(|(mut transform, unit)| {
        transform.translation += unit.vel.extend(0.0) * time.delta_seconds();
    });
}

const MAX_SPEED: f32 = 60.0;
const ACCELLERATION: f32 = 200.0;
const MOVEMENT_DEAD_ZONE: f32 = 2.0;

fn unit_move(mut query: Query<&mut Unit>, time: Res<Time>) {
    query.par_iter_mut().for_each_mut(|mut unit| {
        let target = unit.target_direction.clamp_length_max(1.0) * MAX_SPEED; // max speed
        let delta = target - unit.vel;
        unit.vel += delta.clamp_length_max(time.delta_seconds() * ACCELLERATION); // accell
        if unit.vel.length_squared() > MOVEMENT_DEAD_ZONE * MOVEMENT_DEAD_ZONE {
            unit.last_direction = unit.vel.normalize();
        }
    });
}

fn unit_select(
    mut apply_selection: EventReader<ApplySelectionEvent>,
    mut query: Query<(&Transform, Entity), With<Unit>>,
    mut commands: Commands,
) {
    for event in apply_selection.iter() {
        let rect = Rect::from_corners(event.start, event.end);

        for (transform, entity) in query.iter() {
            let point = transform.translation.truncate();
            let mut entity = commands.entity(entity);
            if rect.contains(point) {
                entity.insert(SelectedMark);
            } else {
                entity.remove::<SelectedMark>();
            }
        }
    }
}

fn selection_added(
    unit_query: Query<&Children, (With<Unit>, Added<SelectedMark>)>,
    mut child_query: Query<&mut Visibility, With<SelectionBox>>,
) {
    for children in unit_query.iter() {
        for child in children.iter() {
            let mut vis = child_query.get_mut(*child).expect("valid child");
            *vis = Visibility::Inherited;
        }
    }
}

fn selection_removed(
    mut removed: RemovedComponents<SelectedMark>,
    unit_query: Query<&Children, With<Unit>>,
    mut child_query: Query<&mut Visibility, With<SelectionBox>>,
) {
    // `RemovedComponents<T>::iter()` returns an interator with the `Entity`s that had their
    // `Component` `T` (in this case `MyComponent`) removed at some point earlier during the frame.
    for entity in removed.iter() {
        let children = unit_query.get(entity).expect("children");
        for child in children.iter() {
            let mut vis = child_query.get_mut(*child).expect("a valid child");
            *vis = Visibility::Hidden;
        }
    }
}

const UNIT_SIZE: f32 = 12.0;
const PUSH_APART_FORCE: f32 = 800.0;

fn unit_push_apart(
    transform_query: Query<(&Transform, Entity), With<Unit>>,
    mut unit_query: Query<(&mut Unit, Entity)>,
    unit_quad_tree: Res<UnitQuadTree>,
    time: Res<Time>,
) {
    let quad_tree = &unit_quad_tree.0;
    unit_query.par_iter_mut().for_each_mut(|(mut a_unit, ae)| {
        match transform_query.get(ae) {
            Ok((a, _)) => {
                let region = pos_to_region(a.translation.truncate());

                let mut quad_tree_query = quad_tree.query_strict(region);
                while let Some(entry) = quad_tree_query.next() {
                    match transform_query.get(*entry.value_ref()) {
                        Ok((b, be)) if ae != be => {
                            let delta = (b.translation - a.translation).truncate() / UNIT_SIZE; // todo how big is a unit?
                            let l = delta.length_squared();
                            if l < 1.0 && l > 0.01 {
                                let push = delta.normalize() * (1.0 - l);
                                a_unit.vel -= PUSH_APART_FORCE * time.delta_seconds() * push;
                            }
                        }
                        Err(_) | Ok(_) => { /* Do nothing */ }
                    }
                }
            }
            Err(_) => { /* Do nothing */ }
        }
    })
}

fn pos_to_point(unit_pos: Vec2) -> Point<u32> {
    let pos = (unit_pos + (Vec2::ONE * 128.0)).round();
    Point {
        x: pos.x as u32,
        y: pos.y as u32,
    }
}

fn pos_to_region(unit_pos: Vec2) -> quadtree_rs::area::Area<u32> {
    AreaBuilder::default()
        .anchor(pos_to_point(unit_pos) - Point { x: 1, y: 1 })
        .dimensions((3, 3))
        .build()
        .expect("valid region")
}
