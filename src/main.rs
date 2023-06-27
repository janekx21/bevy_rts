mod fps_plugin;
mod lumberjack;
mod unit;
mod util;
use crate::fps_plugin::FpsPlugin;
use crate::lumberjack::*;
use crate::unit::*;
use crate::Selection::Dragging;
use bevy::ecs::query::QueryIter;
use bevy::input::mouse::MouseMotion;
use bevy::math::Mat2;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;
use bevy_tweening::*;
use noisy_bevy::{fbm_simplex_2d, simplex_noise_2d};
use quadtree_rs::{area::AreaBuilder, point::Point, Quadtree};
use std::f32::consts::PI;
use util::random_vec2;

// buildings
#[derive(Component)]
pub struct Barrack;

// resources
#[derive(Component)]
pub struct Tree {
    resource: u32,
}

// ui components
#[derive(Default, Resource)]
pub struct Cursor(Vec2);

#[derive(Component)]
enum Selection {
    None,
    Dragging(Vec2, Vec2),
}
#[derive(Component)]
struct SpawnMenu;

#[derive(Default, Resource)]
struct Stats {
    wood: u32,
}

#[derive(Component)]
struct StatsText;

#[derive(Component)]
struct SpawnButton;

// render components
#[derive(Component)]
struct YSort;

#[derive(Resource)]
pub struct UnitQuadTree(Quadtree<u32, Entity>);

impl Default for UnitQuadTree {
    fn default() -> Self {
        let tree = Quadtree::<u32, Entity>::new(8);
        println!("created tree width={}", tree.width());
        UnitQuadTree(tree)
    }
}

// events
pub struct TreeChopEvent(Entity);
pub struct DepositWoodEvent(u32);
pub struct ApplySelectionEvent {
    start: Vec2,
    end: Vec2,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RTS".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(TweeningPlugin)
        .add_plugin(FpsPlugin)
        .add_startup_system(setup)
        .add_startup_system(setup_ui)
        .add_startup_system(setup_lumberjacks)
        .init_resource::<Cursor>()
        .init_resource::<Stats>()
        .init_resource::<UnitQuadTree>()
        .add_event::<TreeChopEvent>()
        .add_event::<ApplySelectionEvent>()
        .add_event::<DepositWoodEvent>()
        .add_system(cursor_world_position)
        .add_system(keyboard_input)
        .add_system(move_camera)
        .add_system(tree_death)
        .add_system(selection_change)
        .add_system(selection_visual)
        .add_system(unit_push_apart)
        .add_system(selection_visible)
        .add_system(unit_select)
        .add_system(unit_vel)
        .add_system(unit_move)
        .add_system(unit_quad_tree_placement)
        .add_system(lumberjack_animation)
        .add_system(lumberjack_next_action)
        .add_system(lumberjack_move_to_position_action)
        .add_system(button_style)
        .add_system(lumberjack_spawn_button)
        .add_system(spawn_menu_tween)
        .add_system(deposit_wood_stat)
        .add_system(stat_text)
        .add_system(ysort)
        .run();
}

// setups

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // todo move into setup functions
    spawn_camera(&mut commands);
    spawn_world(&asset_server, &mut texture_atlases, &mut commands);
    spawn_baracks(&asset_server, &mut texture_atlases, &mut commands);
    spawn_selection(&asset_server, texture_atlases, &mut commands);
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(100.0)),
                position_type: PositionType::Absolute,
                ..default()
            },
            ..default()
        })
        .insert(SpawnMenu)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        margin: UiRect::all(Val::Auto),
                        padding: UiRect::all(Val::Px(16.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Spawn Worker",
                            TextStyle {
                                font: asset_server.load("fonts/roboto_regular.ttf"),
                                font_size: 32.0,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    });
                })
                .insert(SpawnButton);
        });

    commands
        .spawn(NodeBundle {
            background_color: Color::WHITE.into(),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect::default(),
                padding: UiRect::all(Val::Px(32.0)),
                gap: Size::all(Val::Px(32.)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    background_color: Color::CRIMSON.into(),
                    style: Style {
                        padding: UiRect::all(Val::Px(12.)),
                        ..default()
                    },
                    text: Text::from_section(
                        "stats go here",
                        TextStyle {
                            font: asset_server.load("fonts/roboto_regular.ttf"),
                            font_size: 32.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                })
                .insert(StatsText);
        })
        .with_children(|parent| {
            parent
                .spawn(TextBundle {
                    background_color: Color::CRIMSON.into(),
                    style: Style {
                        //padding: UiRect::all(Val::Px(12.)),
                        border: UiRect::all(Val::Px(8.)),
                        ..default()
                    },
                    text: Text::from_section(
                        "stats go here",
                        TextStyle {
                            font: asset_server.load("fonts/roboto_regular.ttf"),
                            font_size: 16.0,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                })
                .insert(StatsText);
        });
}

fn spawn_selection(
    asset_server: &Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    commands: &mut Commands,
) {
    let texture_handle = asset_server.load("highlighted_boxes.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 5, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(0.0, 0.0, 200.0),
            sprite: TextureAtlasSprite {
                index: 1,
                custom_size: Some(Vec2::ONE),
                color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                ..default()
            },
            ..default()
        })
        .insert(Selection::None);
}

fn spawn_baracks(
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    commands: &mut Commands,
) {
    let texture_handle = asset_server.load("barracks_red.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 5, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for i in (-300..300).step_by(50) {
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                transform: Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
                ..default()
            })
            .insert(YSort)
            .insert(Barrack);
    }
}

fn setup_lumberjacks(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut unitQuadTree: ResMut<UnitQuadTree>,
) {
    let count = 20;
    for x in -count..count {
        for y in -count..count {
            let pos = Vec2::new(x as f32 * 16.0, y as f32 * 16.0);
            lumberjack_spawn(
                &mut commands,
                &asset_server,
                &mut texture_atlases,
                pos + simplex_noise_2d(pos) * 100.,
                &mut unitQuadTree,
            )
        }
    }
}

fn spawn_world(
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
    commands: &mut Commands,
) {
    let texture_handle = asset_server.load("grass.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 5, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for x in 0..100 {
        for y in 0..100 {
            let pos = Vec2::new(x as f32, y as f32) * 16. - Vec2::ONE * 16. * 50.;

            let height = fbm_simplex_2d(pos * 0.003, 8, 2.0, 0.5) / 2.;

            commands.spawn(SpriteSheetBundle {
                texture_atlas: texture_atlas_handle.clone(),
                sprite: TextureAtlasSprite {
                    //custom_size: Some(Vec2::new(1000.0, 1000.0)),
                    index: ((height / 2. + 0.5) * 5.).floor() as usize,
                    flip_x: rand::random::<bool>(),
                    flip_y: rand::random::<bool>(),
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(0.0)).with_rotation(
                    Quat::from_axis_angle(Vec3::Z, rand::random::<f32>().round() * PI * 0.5),
                ),
                ..default()
            });

            if height >= 0.1 && height <= 0.3 {
                if rand::random::<i32>() % 8 == 0 {
                    spawn_tree(pos, commands, asset_server, texture_atlases);
                }
            }
        }
    }

    for i in (0..360).step_by(15) {
        let rotation = Mat2::from_angle(i as f32 * PI / 180.0);
        let pos = rotation.mul_vec2(Vec2::X * 16.0 * 12.0);
        spawn_tree(pos, commands, &asset_server, texture_atlases);
    }
}

fn spawn_camera(commands: &mut Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.5;
    commands.spawn(camera);
}

fn spawn_tree(
    pos: Vec2,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("trees.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(16.0, 16.0), 4, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform::from_translation(pos.extend(1.0)),
            sprite: TextureAtlasSprite {
                index: 1,
                ..default()
            },
            ..default()
        })
        .insert(YSort)
        .insert(Tree { resource: 100 });
}

// systems

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn button_style(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut Style),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mut style) in interaction_query.iter_mut() {
        *color = match *interaction {
            Interaction::Clicked => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        };
        style.border = match *interaction {
            Interaction::Hovered => UiRect::all(Val::Px(2.0)),
            _ => UiRect::default(),
        };
    }
}

fn lumberjack_spawn_button(
    query: Query<&Interaction, (Changed<Interaction>, With<SpawnButton>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut unitQuadTree: ResMut<UnitQuadTree>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {
            for _ in 0..20 {
                lumberjack_spawn(
                    &mut commands,
                    &asset_server,
                    &mut texture_atlases,
                    Vec2::new(0.0, 0.0) + random_vec2() * 10.0,
                    &mut unitQuadTree,
                )
            }
        };
    }
}

fn spawn_menu_tween(
    mut query: Query<&mut Style, With<SpawnMenu>>,
    time: Res<Time>,
    windows: Query<&Window>,
    mut var: Local<f32>,
) {
    let win = windows.single();
    let is_hidden = if let Some(position) = win.cursor_position() {
        position.y < 100.0
    } else {
        true
    };
    *var += time.delta_seconds() * if is_hidden { -1.0 } else { 1.0 } / 0.2;
    *var = var.clamp(0.0, 1.0);

    for mut style in query.iter_mut() {
        style.position.bottom = Val::Px(ease_in_out_cubic(*var) * -100.0);
    }
}

fn ease_in_out_cubic(x: f32) -> f32 {
    if x < 0.5 {
        4.0 * x * x * x
    } else {
        1.0 - f32::powf(-2.0 * x + 2.0, 3.0) / 2.0
    }
}

fn deposit_wood_stat(mut deposit_wood: EventReader<DepositWoodEvent>, mut stats: ResMut<Stats>) {
    for event in deposit_wood.iter() {
        stats.wood += event.0
    }
}

fn stat_text(mut query: Query<&mut Text, With<StatsText>>, stats: Res<Stats>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("wood = {}", stats.wood)
    }
}

fn keyboard_input(keys: Res<Input<KeyCode>>) {
    if keys.any_just_pressed([KeyCode::Space]) {
        println!("key got pressed");
    }
}

fn move_camera(
    mut query: Query<&mut Transform, With<Camera2d>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut motion_evr: EventReader<MouseMotion>,
    buttons: Res<Input<MouseButton>>,
) {
    let mut dir_keyboard = Vec2::ZERO;

    if keys.pressed(KeyCode::Left) {
        dir_keyboard -= Vec2::X
    }
    if keys.pressed(KeyCode::Right) {
        dir_keyboard += Vec2::X
    }
    if keys.pressed(KeyCode::Up) {
        dir_keyboard += Vec2::Y
    }
    if keys.pressed(KeyCode::Down) {
        dir_keyboard -= Vec2::Y
    }
    let move_keyboard = dir_keyboard.clamp_length_max(1.0) * 200.0 * time.delta_seconds();

    // todo move to own system
    let dir_mouse = motion_evr
        .iter()
        .map(|e| e.delta)
        .fold(Vec2::ZERO, |x, y| x + y)
        * 0.6 // slow down a little
        * if buttons.pressed(MouseButton::Right) {
            Vec2::new(-1.0, 1.0)
        } else {
            Vec2::ZERO
        };

    let mut camera_transform = query.single_mut();
    camera_transform.translation += (move_keyboard + dir_mouse).round().extend(0.0);
}

fn tree_death(
    mut query: Query<(Entity, &mut Tree)>,
    mut tree_chop_event: EventReader<TreeChopEvent>,
    mut commands: Commands,
) {
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

fn cursor_world_position(
    // need to get window dimensions
    windows: Query<&Window>,
    // query to get camera transform
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut cursor: ResMut<Cursor>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = camera_query.single();

    // get the window that the camera is displaying to (or the primary window)
    let window = match camera.target {
        RenderTarget::Window(window_ref) => match window_ref {
            WindowRef::Entity(e) => windows.get(e).unwrap(),
            WindowRef::Primary => windows.single(),
        },
        _ => windows.single(),
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = window.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(window.width() as f32, window.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        // eprintln!("World coords: {}/{}", world_pos.x, world_pos.y);
        cursor.0 = world_pos;
    }
}

fn selection_change(
    mut query: Query<&mut Selection>,
    cursor: Res<Cursor>,
    input: Res<Input<MouseButton>>,
    mut apply_selection: EventWriter<ApplySelectionEvent>,
) {
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
                apply_selection.send(ApplySelectionEvent { start, end });
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

fn ysort(mut query: Query<&mut Transform, With<YSort>>) {
    for mut transform in query.iter_mut() {
        transform.translation.z = 200.0 - transform.translation.y * 0.0001;
    }
}
