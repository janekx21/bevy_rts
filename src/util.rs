use bevy::ecs::query::ReadOnlyWorldQuery;

use crate::{Entity, QueryIter, Transform, Vec2};

pub fn find_nearest<F: ReadOnlyWorldQuery>(
    transform_query: QueryIter<(Entity, &Transform), F>,
    worker_pos: Vec2,
) -> Option<(Entity, Vec2)> {
    transform_query.fold(None, |acc_option, (entity, transform)| {
        let target_pos = transform.translation.truncate();
        Some(if let Some(acc) = acc_option {
            if Vec2::distance(worker_pos, target_pos) < Vec2::distance(worker_pos, acc.1) {
                (entity, target_pos)
            } else {
                acc
            }
        } else {
            (entity, target_pos)
        })
    })
}

pub fn random_vec2() -> Vec2 {
    let x = rand::random::<f32>() * 2.0 - 1.0;
    let y = rand::random::<f32>() * 2.0 - 1.0;
    Vec2::new(x, y)
}
