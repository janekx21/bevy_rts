mod fps_plugin;
mod worker;
mod util;

use bevy::prelude::*;
use std::f32::consts::PI;
use bevy::ecs::query::{EntityFetch, FilterFetch, QueryIter, ReadFetch, WorldQuery};
use bevy::math::{Mat2, Vec3Swizzles};
use bevy::render::camera::{Camera2d, RenderTarget};
use crate::fps_plugin::FpsPlugin;
use crate::Selection::Dragging;
use crate::worker::{move_workers, spawn_worker, Worker, worker_selecter, worker_selection, move_worker_todo};

#[derive(Component)]
pub struct Barrack;

#[derive(Component)]
pub struct Tree {
    resource: u32,
}

pub struct TreeChop(Entity);

pub struct ApplySelection {
    start: Vec2,
    end: Vec2,
}

#[derive(Default)]
struct Cursor(Vec2);

#[derive(Component)]
enum Selection {
    None,
    Dragging(Vec2, Vec2),
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Bevy RTS".to_string(),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FpsPlugin)
        .add_startup_system(setup)
        .init_resource::<Cursor>()
        .add_system(my_cursor_system)
        .add_system(keyboard_input)
        .add_system(move_workers)
        .add_system(move_camera)
        .add_system(push_apart)
        .add_system(tree_death)
        .add_event::<TreeChop>()
        .add_event::<ApplySelection>()
        .add_system(selection_change)
        .add_system(selection_visual)
        .add_system(worker_selection)
        .add_system(worker_selecter)
        .add_system(move_worker_todo)
        .run();
}

fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scale = 0.5;
    commands.spawn_bundle(camera);

    commands.spawn_bundle(UiCameraBundle::default());

    let texture_handle = asset_server.load("grass.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 5, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn_bundle(SpriteSheetBundle {
        texture_atlas: texture_atlas_handle,
        sprite: TextureAtlasSprite { custom_size: Some(Vec2::new(1000.0, 1000.0)), index: 1, ..default() }
        ,
        ..default()
    });


    /*
    let texture_handle = asset_server.load("farmer_red.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 5, 12);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let box_selector = {
        let texture_handle = asset_server.load("box_selector.png");
        let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 2, 1);
        texture_atlases.add(texture_atlas)
    };
     */

    let count = 4;
    for x in -count..count {
        for y in -count..count {
            /*
            let selector = commands.spawn_bundle(
                SpriteSheetBundle {
                    texture_atlas: box_selector.clone(),
                    ..default()
                }
            ).insert(SelectionBox).id();

            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(x as f32 * 16.0, y as f32 * 16.0, 1.0)),
                    ..default()
                })
                .insert(Worker{target: None, wood: 0, is_selected: false})
                .add_child(selector);
             */

            spawn_worker(&mut commands, &asset_server, &mut texture_atlases, Vec2::new(x as f32 * 16.0, y as f32 * 16.0))
        }
    }

    let texture_handle = asset_server.load("barracks_red.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 5);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for i in (-300..300).step_by(600) {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
                ..default()
            })
            .insert(Barrack);
    }

    let texture_handle = asset_server.load("trees.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for i in (0..360).step_by(15) {
        let rotation = Mat2::from_angle(i as f32 * PI / 180.0);
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(rotation.mul_vec2(Vec2::X * 16.0 * 12.0).extend(0.0)),
                sprite: TextureAtlasSprite { index: 1, ..default() },
                ..default()
            })
            .insert(Tree { resource: 100 });
    }

    let texture_handle = asset_server.load("highlighted_boxes.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 5, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.spawn_bundle(SpriteSheetBundle {
        texture_atlas: texture_atlas_handle.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 2.0),
        sprite: TextureAtlasSprite { index: 1, custom_size: Some(Vec2::ONE), color: Color::rgba(1.0, 1.0, 1.0, 0.5), ..default() },
        ..default()
    }).insert(Selection::None);
}

fn keyboard_input(keys: Res<Input<KeyCode>>) {
    if keys.any_just_pressed([KeyCode::Space]) {
        println!("key got pressed");
    }
}


fn move_camera(mut query: Query<&mut Transform, With<Camera2d>>, keys: Res<Input<KeyCode>>, time: Res<Time>) {
    let mut dir = Vec2::ZERO;

    if keys.pressed(KeyCode::Left) {
        dir -= Vec2::X
    }
    if keys.pressed(KeyCode::Right) {
        dir += Vec2::X
    }
    if keys.pressed(KeyCode::Up) {
        dir += Vec2::Y
    }
    if keys.pressed(KeyCode::Down) {
        dir -= Vec2::Y
    }
    let mut camera_transform = query.single_mut();
    camera_transform.translation += (dir.clamp_length_max(1.0) * 100.0 * time.delta_seconds()).extend(0.0);
}

fn tree_death(mut query: Query<(Entity, &mut Tree)>, mut tree_chop_event: EventReader<TreeChop>, mut commands: Commands) {
    for event in tree_chop_event.iter() {
        if let Ok(mut tree) = query.get_component_mut::<Tree>(event.0) {
            if tree.resource == 0 {
                commands.entity(event.0).despawn();
            } else {
                tree.resource -= 1;
            }
        }
    }
}

fn push_apart(mut query: Query<&mut Transform, With<Worker>>) {
    let mut combinations = query.iter_combinations_mut();
    while let Some([mut a, mut b]) = combinations.fetch_next() {
        let delta = b.translation - a.translation;
        if delta.length() < 12.0 {
            let push = delta.normalize() * 0.8;
            a.translation -= push;
            b.translation += push;
        }
    }
}

fn my_cursor_system(
    // need to get window dimensions
    window_resource: Res<Windows>,
    // query to get camera transform
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut cursor: ResMut<Cursor>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = camera_query.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = if let RenderTarget::Window(id) = camera.target {
        window_resource.get(id).unwrap()
    } else {
        window_resource.get_primary().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = window.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        // eprintln!("World coords: {}/{}", world_pos.x, world_pos.y);
        cursor.0 = world_pos;
    }
}


fn selection_change(mut query: Query<&mut Selection>, cursor: Res<Cursor>, input: Res<Input<MouseButton>>, mut apply_selection: EventWriter<ApplySelection>) {
    let mut selection = query.single_mut();

    match *selection {
        Selection::None => {
            if input.pressed(MouseButton::Left) {
                *selection = Dragging(cursor.0, cursor.0)
            }
        }
        Dragging(start, end) => {
            *selection = if input.pressed(MouseButton::Left) {
                Dragging(start, cursor.0)
            } else {
                apply_selection.send(ApplySelection { start, end });
                Selection::None
            }
        }
    }
}

fn selection_visual(mut query: Query<(&mut Transform, &mut TextureAtlasSprite, &Selection)>) {
    let (mut transform, mut sprite, selection) = query.single_mut();

    transform.scale = Vec3::ZERO;

    if let Dragging(start, end) = *selection {
        let center = (start + end) * 0.5;
        let size = Vec2::abs(start - end);

        transform.translation = center.extend(2.0);
        transform.scale = size.extend(1.0);
        sprite.index = 1;
    }
}
