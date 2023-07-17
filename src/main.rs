mod fps_plugin;
mod lumberjack;
mod soldier;
mod unit;
mod util;
use crate::fps_plugin::FpsPlugin;
use crate::lumberjack::*;
use crate::unit::*;
use crate::Selection::Dragging;
use bevy::ecs::query::QueryIter;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::utils::HashMap;
use bevy::window::PresentMode;
use bevy::window::WindowMode;
use bevy::window::WindowRef;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_tweening::*;
use noisy_bevy::{fbm_simplex_2d, simplex_noise_2d};
use quadtree_rs::Quadtree;
use soldier::SoldierPlugin;
use soldier::SpawnSoldierEvent;
use std::f32::consts::PI;
use util::add_texture_atlas;
use util::ease_in_out_cubic;
use util::load_image;
use util::random_vec2;

// buildings
#[derive(Component)]
pub struct Barrack;

// resources
#[derive(Component)]
pub struct Tree {
    resource: i32,
}

// ui components
#[derive(Default, Resource, Deref)]
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
#[derive(Component)]
struct Cull2D;

#[derive(Resource, Deref, DerefMut)]
pub struct UnitQuadTree(Quadtree<u32, Entity>);

#[derive(Resource)]
pub struct SpriteSheets {
    swordsman_red: Handle<TextureAtlas>,
    box_selector: Handle<TextureAtlas>,
    highlighted_boxes: Handle<TextureAtlas>,
    trees: Handle<TextureAtlas>,
    farmer_red: Handle<TextureAtlas>,
    grass_deco: Handle<TextureAtlas>,
    barracks_red: Handle<TextureAtlas>,
}

// events
pub struct TreeChopEvent(Entity);
pub struct DepositWoodEvent(u32);
pub struct ApplySelectionEvent {
    start: Vec2,
    end: Vec2,
}
#[derive(Deref)]
struct TreeSpawnEvent {
    pos: Vec2,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "RTS".into(),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(TweeningPlugin)
        .add_plugin(FpsPlugin)
        // .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(SoldierPlugin)
        .add_plugin(UnitPlugin)
        .add_plugin(LumberjackPlugin)
        .add_startup_system(setup)
        .add_startup_system(setup_ui)
        .add_startup_system(setup_lumberjacks)
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_world)
        .add_startup_system(spawn_baracks)
        .add_startup_system(spawn_selection)
        .init_resource::<SpriteSheets>()
        .init_resource::<Cursor>()
        .init_resource::<Stats>()
        .init_resource::<UnitQuadTree>()
        .add_event::<TreeChopEvent>()
        .add_event::<ApplySelectionEvent>()
        .add_event::<DepositWoodEvent>()
        .add_event::<TreeSpawnEvent>()
        .register_type::<Lumberjack>()
        .add_system(cursor_world_position)
        .add_system(keyboard_input)
        .add_system(move_camera)
        .add_system(tree_death)
        .add_system(selection_change)
        .add_system(selection_visual)
        .add_system(button_style)
        .add_system(lumberjack_spawn_button)
        .add_system(spawn_menu_tween)
        .add_system(deposit_wood_stat)
        .add_system(stat_text)
        .add_system(ysort)
        .add_system(tree_spawning.after(spawn_world))
        .add_system(camera_view_check)
        .add_system(fullscreen_toggle)
        .run();
}

impl Default for UnitQuadTree {
    fn default() -> Self {
        UnitQuadTree(Quadtree::<u32, Entity>::new(8))
    }
}

impl FromWorld for SpriteSheets {
    fn from_world(world: &mut World) -> Self {
        SpriteSheets {
            swordsman_red: {
                let texture_atlas = TextureAtlas::from_grid(
                    load_image(world, "swordsman_red.png"),
                    Vec2::new(16.0, 16.0),
                    5,
                    12,
                    None,
                    None,
                );
                add_texture_atlas(world, texture_atlas)
            },
            farmer_red: {
                let texture_atlas = TextureAtlas::from_grid(
                    load_image(world, "farmer_red.png"),
                    Vec2::new(16.0, 16.0),
                    5,
                    12,
                    None,
                    None,
                );
                add_texture_atlas(world, texture_atlas)
            },
            box_selector: {
                let texture_atlas = TextureAtlas::from_grid(
                    load_image(world, "box_selector.png"),
                    Vec2::new(16.0, 16.0),
                    2,
                    1,
                    None,
                    None,
                );
                add_texture_atlas(world, texture_atlas)
            },
            highlighted_boxes: {
                let texture_atlas = TextureAtlas::from_grid(
                    load_image(world, "highlighted_boxes.png"),
                    Vec2::new(16.0, 16.0),
                    5,
                    1,
                    None,
                    None,
                );
                add_texture_atlas(world, texture_atlas)
            },
            trees: {
                let texture_atlas = TextureAtlas::from_grid(
                    load_image(world, "trees.png"),
                    Vec2::new(16.0, 16.0),
                    4,
                    1,
                    None,
                    None,
                );
                add_texture_atlas(world, texture_atlas)
            },
            grass_deco: {
                let texture_atlas = TextureAtlas::from_grid(
                    load_image(world, "ground/grass_deco.png"),
                    Vec2::new(16.0, 16.0),
                    2,
                    2,
                    None,
                    None,
                );
                add_texture_atlas(world, texture_atlas)
            },
            barracks_red: {
                let texture_atlas = TextureAtlas::from_grid(
                    load_image(world, "barracks_red.png"),
                    Vec2::new(16.0, 16.0),
                    4,
                    5,
                    None,
                    None,
                );
                add_texture_atlas(world, texture_atlas)
            },
        }
    }
}

// setups

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // todo move into setup functions
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
        });
}

fn spawn_selection(mut commands: Commands, sprite_sheets: Res<SpriteSheets>) {
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: sprite_sheets.highlighted_boxes.clone(),
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

fn spawn_baracks(mut commands: Commands, sprite_sheets: Res<SpriteSheets>) {
    for i in (-300..300).step_by(50) {
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: sprite_sheets.barracks_red.clone(),
                transform: Transform::from_translation(Vec3::new(i as f32, 0.0, 0.0)),
                ..default()
            })
            .insert(YSort)
            .insert(Cull2D)
            .insert(Barrack);
    }
}

fn setup_lumberjacks(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut unit_quad_tree: ResMut<UnitQuadTree>,
    mut events: EventWriter<SpawnLumberjackEvent>,
) {
    let count = 10;
    for x in -count..count {
        for y in -count..count {
            let pos = Vec2::new(x as f32 * 16.0, y as f32 * 16.0);
            let pos = pos + simplex_noise_2d(pos) * 100.0;
            events.send(SpawnLumberjackEvent(pos));
        }
    }
}

fn spawn_world(
    mut commands: Commands,
    mut spawn_tree_events: EventWriter<TreeSpawnEvent>,
    sprite_sheets: Res<SpriteSheets>,
) {
    for x in 0..100 {
        for y in 0..100 {
            let pos = Vec2::new(x as f32, y as f32) * 16. - Vec2::ONE * 16. * 50.;
            let height = fbm_simplex_2d(pos * 0.003, 8, 2.0, 0.5) / 2.;
            let height_norm = height / 2. + 0.5;
            commands
                .spawn(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::ONE * 16.0),
                        color: Color::hsl(
                            76.0 - height_norm * 24.0,
                            0.6 - height_norm * 0.1,
                            0.6 + height_norm * 0.15,
                        ),
                        ..default()
                    },
                    transform: Transform::from_translation(pos.extend(0.0)),
                    ..default()
                })
                .insert(Name::new("Ground"))
                .insert(Cull2D)
                .with_children(|builder| {
                    if rand::random::<u8>() % 5 == 0 {
                        builder.spawn(SpriteSheetBundle {
                            texture_atlas: sprite_sheets.grass_deco.clone(),
                            sprite: TextureAtlasSprite {
                                index: rand::random::<usize>() % 4,
                                flip_x: rand::random(),
                                ..default()
                            },
                            transform: Transform::from_translation(
                                (random_vec2() * 6.0).round().extend(0.0),
                            ),
                            ..default()
                        });
                    }
                });

            if height >= 0.1 && height <= 0.3 {
                if rand::random::<i32>() % 8 == 0 {
                    let pos = pos + (random_vec2() * 8.0).round();
                    spawn_tree_events.send(TreeSpawnEvent { pos })
                }
            }
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 0.5;
    commands.spawn(camera);
}

fn tree_spawning(
    mut events: EventReader<TreeSpawnEvent>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for TreeSpawnEvent { pos } in events.iter() {
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: sprite_sheets.trees.clone(),
                transform: Transform::from_translation(pos.extend(1.0)),
                sprite: TextureAtlasSprite {
                    index: 1 + rand::random::<usize>() % 3,
                    ..default()
                },
                ..default()
            })
            .insert(Name::new("Tree"))
            .insert(YSort)
            .insert(Cull2D)
            .insert(Name::new("Tree"))
            .insert(Tree { resource: 100 });
    }
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
    mut event: EventWriter<SpawnSoldierEvent>,
) {
    for interaction in query.iter() {
        if *interaction == Interaction::Clicked {
            for _ in 0..20 {
                let pos = Vec2::new(0.0, 0.0) + random_vec2() * 100.0;
                event.send(SpawnSoldierEvent(pos));
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

fn fullscreen_toggle(keys: Res<Input<KeyCode>>, mut windows: Query<&mut Window>) {
    if keys.just_pressed(KeyCode::F11) {
        let mut window = windows.single_mut();
        window.mode = if window.mode == WindowMode::Windowed {
            WindowMode::BorderlessFullscreen
        } else {
            WindowMode::Windowed
        };
    }
}

// fn inspector_toggle(keys: Res<Input<KeyCode>>) {
//     if keys.just_pressed(KeyCode::F11) {
//         let mut window = windows.single_mut();
//         window.mode = if window.mode == WindowMode::Windowed {
//             WindowMode::BorderlessFullscreen
//         } else {
//             WindowMode::Windowed
//         };
//     }
// }

fn tree_death(
    mut query: Query<(Entity, &mut Tree)>,
    mut tree_chop_event: EventReader<TreeChopEvent>,
    mut commands: Commands,
) {
    for event in tree_chop_event.iter() {
        if let Ok(mut tree) = query.get_component_mut::<Tree>(event.0) {
            if tree.resource <= 0 {
                if let Some(e) = commands.get_entity(event.0) {
                    e.despawn_recursive();
                }
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
            WindowRef::Entity(e) => windows.get(e).expect("a window from an entity"),
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
fn camera_view_check(
    camera_query: Query<(&Transform, &OrthographicProjection), With<Camera>>,
    mut visible_query: Query<
        (&GlobalTransform, &mut Visibility, &Handle<TextureAtlas>),
        With<Cull2D>,
    >,
    texute_atlases: Res<Assets<TextureAtlas>>,
    mut size_cache: Local<HashMap<Handle<TextureAtlas>, f32>>,
) {
    for (camera_transform, projection) in camera_query.iter() {
        let camera_pos = camera_transform.translation.truncate();
        for (transform, mut visible, tex) in visible_query.iter_mut() {
            let size = *(*size_cache).entry(tex.clone()).or_insert_with(|| {
                texute_atlases
                    .get(tex)
                    .expect("valid texture")
                    .textures
                    .iter()
                    .map(Rect::size)
                    .map(Vec2::max_element)
                    .map(f32::round)
                    .map(|f| f as i32)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .max()
                    .unwrap_or_default() as f32
            });

            let mut rect = projection.area.inset(size);
            rect.min += camera_pos;
            rect.max += camera_pos;
            let pos = transform.translation().truncate();
            *visible = if rect.contains(pos) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
