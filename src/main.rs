use std::f32::consts::PI;
use std::ops::MulAssign;
use bevy::diagnostic::{Diagnostic, Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::ecs::query::{EntityFetch, FilterFetch, QueryIter, ReadFetch, WorldQuery};
use bevy::input::keyboard::KeyboardInput;
use bevy::math::{const_quat, Mat2, Vec3Swizzles};
use bevy::prelude::*;
use bevy::prelude::StartupStage::PreStartup;
use bevy::reflect::ReflectRef::Tuple;
use bevy::render::camera::Camera2d;
use bevy::utils::tracing::instrument::WithSubscriber;

#[derive(Component)]
struct Worker{
    target: Option<Entity>,
    wood: u32
}

#[derive(Component)]
struct Barrack;

#[derive(Component)]
struct Tree{
    resource: u32
}

struct TreeChop(Entity);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(keyboard_input)
        .add_system(move_workers)
        .add_system(move_camera)
        .add_system(push_apart)
        .add_system(tree_death)
        .add_event::<TreeChop>()
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_system(text_change)
        // .add_system(animate_worker)
        .run();
}

fn text_change(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    let mut fps = 0.0;
    if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_avg) = fps_diagnostic.average() {
            fps = fps_avg;
        }
    }
    query.single_mut().sections[0].value = format!("fps = {:.1}", fps)
}

fn setup(mut commands: Commands,
         asset_server: Res<AssetServer>,
         mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection.scale = 0.5;
    commands.spawn_bundle(camera);

    commands.spawn_bundle(UiCameraBundle::default());







    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                position_type: PositionType::Relative,
                position: Rect {
                    bottom: Val::Px(10.0),
                    right: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "hello\nbevy!",
                TextStyle {
                    font: asset_server.load("fonts/roboto_regular.ttf"),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    ..default()
                },
            ),
            ..default()
        });






    let texture_handle = asset_server.load("farmer_red.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 5, 12);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for x in -1..2 {
        for y in -1..2 {
            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle.clone(),
                    transform: Transform::from_translation(Vec3::new(x as f32 * 16.0, y as f32 * 16.0, 1.0)),
                    ..default()
                })
                .insert(Worker{target: None, wood: 0});
        }
    }

    let texture_handle = asset_server.load("barracks_red.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 5);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for i in (-300..300).step_by(600) {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(i as f32, 0.0,0.0)),
                ..default()
            })
            .insert(Barrack);
    }

    let texture_handle = asset_server.load("trees.png");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 1);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for i in (0..360).step_by(15) {
        let rotation = Mat2::from_angle(i as f32 * PI / 180.0 );
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(rotation.mul_vec2(Vec2::X * 16.0 * 12.0).extend(0.0)),
                sprite: TextureAtlasSprite{index: 1, ..default()},
                ..default()
            })
            .insert(Tree{resource: 100});
    }
}

fn keyboard_input(keys: Res<Input<KeyCode>>) {
    if keys.any_just_pressed([KeyCode::Space]) {
        println!("key got pressed");
    }
}

fn move_workers(
    mut worker_query: Query<(&mut Worker, &mut Transform)>,
    barrack_query: Query<(Entity, &Transform), (With<Barrack>, Without<Worker>)>,
    tree_query: Query<(Entity, &Transform), (With<Tree>, Without<Worker>)>,
    mut tree_chop_event: EventWriter<TreeChop>,
    entity_query: Query<Entity>
) {
    for (mut worker, mut transform) in worker_query.iter_mut() {
        let worker_pos = transform.translation.xy();
        if let Some(target) = worker.target {
            let mut delta = Vec2::ZERO;
            if let Ok(barrack_transform) = barrack_query.get_component::<Transform>(target) {
                // move towards barrack
                delta = barrack_transform.translation.xy() - worker_pos;
                if delta.length() < 10.0 {
                    // found target
                    worker.target = None;
                    worker.wood = 0;
                }
            }
            if let Ok((tree_entity, tree_transform)) = tree_query.get(target) {
                // move towards tree
                delta = tree_transform.translation.xy() - worker_pos;
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
            if tree.resource <= 0 {
                commands.entity(event.0).despawn();
            } else {
                tree.resource-=1;
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

fn find_nearest<F: WorldQuery> (transform_query: QueryIter<(Entity, &Transform), (EntityFetch, ReadFetch<Transform>), F>, worker_pos: Vec2) -> Option<(Entity, Vec2)>  where F::Fetch: FilterFetch{
    transform_query.fold(None, |acc_option, (e, t)| Some(if let Some(acc) = acc_option {
        if Vec2::distance(worker_pos, t.translation.xy()) < Vec2::distance(worker_pos, acc.1)
        {
            (e, t.translation.xy())
        } else {
            acc
        }
    } else {
        (e, t.translation.xy())
    }))
}

fn random_vec2() -> Vec2 {
    let x = rand::random::<f32>() * 2.0 - 1.0;
    let y = rand::random::<f32>() * 2.0 - 1.0;
    Vec2::new(x,y)
}


/*
fn animate_worker(
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&Worker, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (_, mut sprite, texture_atlas_handle) in query.iter_mut() {
        let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
        sprite.index = (sprite.index + 1) % texture_atlas.len();
    }
}
 */
