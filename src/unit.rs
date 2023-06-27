use crate::{ApplySelectionEvent, UnitQuadTree};
use bevy::{ecs::query::BatchingStrategy, prelude::*};
use quadtree_rs::{area::AreaBuilder, point::Point};

#[derive(Component, Default)]
pub struct Unit {
    pub is_selected: bool,
    pub vel: Vec2,
    pub target_direction: Vec2,
    pub point: Option<(Point<u32>, u64)>,
}

#[derive(Component)]
pub struct SelectionBox;

pub fn unit_quad_tree_placement(
    mut query: Query<(&Transform, &mut Unit, Entity)>,
    mut unit_quad_tree: ResMut<UnitQuadTree>,
) {
    for (transform, mut unit, entity) in query.iter_mut() {
        let unit_pos = transform.translation.truncate();
        let point_pos = pos_to_point(unit_pos);
        if let Some((current_point, current_handle)) = unit.point {
            if current_point != point_pos {
                // println!("move {:?} -> {:?}", current_point, point_pos);
                // needs movement
                // println!("must be moved");
                unit_quad_tree.0.delete_by_handle(current_handle);
                let handle = unit_quad_tree
                    .0
                    .insert_pt(point_pos, entity)
                    .expect("valid handle");
                unit.point = Some((point_pos, handle));
            }
        } else {
            // needs insert
            let handle = unit_quad_tree
                .0
                .insert_pt(point_pos, entity)
                .expect("valid handle");
            unit.point = Some((point_pos, handle));
            // println!("not in tree, got inserted");
        }
    }
}

fn pos_to_point(unit_pos: Vec2) -> Point<u32> {
    let pos = ((unit_pos / 8.0) + (Vec2::ONE * 128.0)).round();
    Point {
        x: pos.x as u32,
        y: pos.y as u32,
    }
}

pub fn unit_vel(mut query: Query<(&mut Transform, &Unit)>, time: Res<Time>) {
    query.par_iter_mut().for_each_mut(|(mut transform, unit)| {
        transform.translation += unit.vel.extend(0.0) * time.delta_seconds();
    });
}

pub fn unit_move(mut query: Query<&mut Unit>, time: Res<Time>) {
    query.par_iter_mut().for_each_mut(|mut unit| {
        let target = unit.target_direction.clamp_length_max(1.0) * 60.; // max speed
        let delta = target - unit.vel;
        unit.vel += delta.clamp_length_max(time.delta_seconds() * 200.0); // accell
    });
}

pub fn unit_select(
    mut apply_selection: EventReader<ApplySelectionEvent>,
    mut query: Query<(&Transform, &mut Unit)>,
) {
    for event in apply_selection.iter() {
        let min = Vec2::min(event.start, event.end);
        let max = Vec2::max(event.start, event.end);

        for (transform, mut unit) in query.iter_mut() {
            let p = transform.translation.truncate();
            let inside = p.x > min.x && p.x < max.x && p.y > min.y && p.y < max.y;
            unit.is_selected = inside;
        }
    }
}

pub fn selection_visible(
    mut child_query: Query<(&Parent, &mut Visibility), With<SelectionBox>>,
    parent_query: Query<&Unit>,
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

pub fn unit_push_apart(
    query: Query<(&Transform, Entity), With<Unit>>,
    mut unitq: Query<&mut Unit>,
    unit_quad_tree: Res<UnitQuadTree>,
    time: Res<Time>,
) {
    for (a, ae) in query.iter() {
        let region = AreaBuilder::default()
            .anchor(pos_to_point(a.translation.truncate()) - Point { x: 1, y: 1 })
            .dimensions((3, 3))
            .build()
            .unwrap();
        let mut other_query = unit_quad_tree.0.query(region);
        while let Some(entry) = other_query.next() {
            let b_entity = entry.value_ref();
            let (b, be) = query.get(*b_entity).expect("valid enitiy");
            if ae == be {
                continue; // skip myself
            }
            let delta = (b.translation - a.translation).truncate() / 12.0;
            if delta.length() < 1.0 {
                let push = (delta * 100.0).clamp_length_max(1.0) * (1. - delta.length()) * 2.0;
                //a.translation -= push;
                let mut unit = unitq.get_mut(ae).expect("valid entity");
                unit.vel -= push * 800.0 * time.delta_seconds();
                // dont do this b.translation += push;
            }
        }
    }
    // let mut combinations = query.iter_combinations_mut();
    // while let Some([mut a, mut b]) = combinations.fetch_next() {}
}
