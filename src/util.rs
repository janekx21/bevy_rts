use crate::{Entity, EntityFetch, FilterFetch, QueryIter, ReadFetch, Transform, Vec2, WorldQuery};

pub fn find_nearest<F: WorldQuery> (transform_query: QueryIter<(Entity, &Transform), (EntityFetch, ReadFetch<Transform>), F>, worker_pos: Vec2) -> Option<(Entity, Vec2)>  where F::Fetch: FilterFetch{
    transform_query.fold(None, |acc_option, (e, t)| Some(if let Some(acc) = acc_option {
        if Vec2::distance(worker_pos, t.translation.truncate()) < Vec2::distance(worker_pos, acc.1)
        {
            (e, t.translation.truncate())
        } else {
            acc
        }
    } else {
        (e, t.translation.truncate())
    }))
}

pub fn random_vec2() -> Vec2 {
    let x = rand::random::<f32>() * 2.0 - 1.0;
    let y = rand::random::<f32>() * 2.0 - 1.0;
    Vec2::new(x,y)
}
