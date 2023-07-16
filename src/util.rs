use bevy::{asset::AssetPath, ecs::query::ReadOnlyWorldQuery, prelude::*};

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

pub fn nearest_entity(
    acc: Option<(Entity, Vec2)>,
    source_pos: Vec2,
    target: (Entity, Vec2),
) -> Option<(Entity, Vec2)> {
    let (entity, target_pos) = target;
    Some(if let Some(acc) = acc {
        if Vec2::distance(source_pos, target_pos) < Vec2::distance(source_pos, acc.1) {
            (entity, target_pos)
        } else {
            acc
        }
    } else {
        (entity, target_pos)
    })
}

pub fn random_vec2() -> Vec2 {
    let x = rand::random::<f32>() * 2.0 - 1.0;
    let y = rand::random::<f32>() * 2.0 - 1.0;
    Vec2::new(x, y)
}

pub fn load_image<'a, P: Into<AssetPath<'a>>>(world: &mut World, path: P) -> Handle<Image> {
    let asset_server = world
        .get_resource::<AssetServer>()
        .expect("an asset server resource");
    asset_server.load(path)
}

pub fn add_texture_atlas(world: &mut World, texture_atlas: TextureAtlas) -> Handle<TextureAtlas> {
    let mut texture_atlases = world
        .get_resource_mut::<Assets<TextureAtlas>>()
        .expect("asset server resource");
    texture_atlases.add(texture_atlas)
}

pub fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - f32::powf(-2.0 * x + 2.0, 3.0) / 2.0
    }
}
