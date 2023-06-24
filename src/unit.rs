use bevy::prelude::*;

use crate::ApplySelectionEvent;

#[derive(Component, Default)]
pub struct Unit {
    pub is_selected: bool,
    pub vel: Vec2,
    pub target_direction: Vec2,
}

#[derive(Component)]
pub struct SelectionBox;

pub fn unit_vel(mut query: Query<(&mut Transform, &Unit)>, time: Res<Time>) {
    for (mut transform, unit) in query.iter_mut() {
        transform.translation += unit.vel.extend(0.0) * time.delta_seconds();
    }
}

pub fn unit_move(mut query: Query<&mut Unit>, time: Res<Time>) {
    for mut unit in query.iter_mut() {
        let target = unit.target_direction.clamp_length_max(1.0) * 60.; // max speed
        let delta = target - unit.vel;
        unit.vel += delta.clamp_length_max(time.delta_seconds() * 200.0); // accell
    }
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

pub fn unit_push_apart(mut query: Query<&mut Transform, With<Unit>>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([mut a, mut b]) = combinations.fetch_next() {
        let delta = (b.translation - a.translation) / 12.0;
        if delta.length() < 1.0 {
            let push = delta.normalize() * (1. - delta.length()) * 2.0;
            a.translation -= push;
            b.translation += push;
        }
    }
}
