use crate::{ApplySelectionEvent, UnitQuadTree};
use bevy::prelude::*;
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
                            let delta = (b.translation - a.translation).truncate() * 0.08;
                            let l = delta.length_squared();
                            if l < 1.0 && l > 0.01 {
                                let push = delta.normalize() * (1.0 - delta.length_squared());
                                a_unit.vel -= 800.0 * time.delta_seconds() * push;
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

fn pos_to_region(unit_pos: Vec2) -> quadtree_rs::area::Area<u32> {
    let region = AreaBuilder::default()
        .anchor(pos_to_point(unit_pos) - Point { x: 1, y: 1 })
        .dimensions((3, 3))
        .build()
        .unwrap();
    region
}
