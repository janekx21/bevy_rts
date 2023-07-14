use bevy::{asset::AssetPath, prelude::*};

use crate::{
    unit::{SelectedMark, SelectionBox, Unit},
    Cull2D, Cursor, YSort,
};
pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoldierResource>()
            .add_event::<SpawnSoldierEvent>()
            .add_system(soldier_spawn)
            .add_system(move_to_position_action)
            .add_system(next_action);
    }
}

#[derive(Component, Default)]
pub struct Soldier {
    action: SoldierAction,
    weapon_timeout: f32,
    weapon: Option<Weapon>,
    armor: Option<Armor>,
}

#[derive(Default, PartialEq)]
pub enum SoldierAction {
    #[default]
    Idle,
    MoveToPosition {
        target: Vec2,
        attack_move: bool,
    },
    Attack(Entity),
}

pub enum Weapon {
    Sword,
    Axe,
    Spear,
    Bow,
    Sling,
    Crossbow,
}

pub enum Armor {
    Leather,
    Chain,
    Plate,
}

pub struct SpawnSoldierEvent(pub Vec2);

#[derive(Resource)]
pub struct SoldierResource {
    swordsman_red: Handle<TextureAtlas>,
    box_selector: Handle<TextureAtlas>,
}

impl FromWorld for SoldierResource {
    fn from_world(world: &mut World) -> Self {
        let swordsman_red = {
            let texture_atlas = TextureAtlas::from_grid(
                load_image(world, "swordsman_red.png"),
                Vec2::new(16.0, 16.0),
                5,
                12,
                None,
                None,
            );

            add_texture_atlas(world, texture_atlas)
        };

        let box_selector = {
            let texture_atlas = TextureAtlas::from_grid(
                load_image(world, "box_selector.png"),
                Vec2::new(16.0, 16.0),
                2,
                1,
                None,
                None,
            );

            add_texture_atlas(world, texture_atlas)
        };

        SoldierResource {
            swordsman_red,
            box_selector,
        }
    }
}

fn load_image<'a, P: Into<AssetPath<'a>>>(world: &mut World, path: P) -> Handle<Image> {
    let asset_server = world
        .get_resource::<AssetServer>()
        .expect("an asset server resource");
    asset_server.load(path)
}

fn add_texture_atlas(world: &mut World, texture_atlas: TextureAtlas) -> Handle<TextureAtlas> {
    let mut texture_atlases = world
        .get_resource_mut::<Assets<TextureAtlas>>()
        .expect("asset server resource");
    texture_atlases.add(texture_atlas)
}

pub fn soldier_spawn(
    mut spawn_event: EventReader<SpawnSoldierEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    soldier_resource: Res<SoldierResource>,
) {
    for event in spawn_event.iter() {
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: soldier_resource.swordsman_red.clone(),
                transform: Transform::from_translation(event.0.extend(1.0)),
                ..default()
            })
            .insert(YSort)
            .insert(Cull2D)
            .insert(Unit::default())
            .insert(Soldier::default())
            .with_children(|builder| {
                builder
                    .spawn(SpriteSheetBundle {
                        texture_atlas: soldier_resource.box_selector.clone(),
                        visibility: Visibility::Hidden,
                        ..default()
                    })
                    .insert(SelectionBox);
            });
    }
}

pub fn move_to_position_action(
    mut query: Query<(&Unit, &mut Soldier), With<SelectedMark>>,
    input: Res<Input<MouseButton>>,
    cursor: Res<Cursor>,
) {
    if input.just_pressed(MouseButton::Right) {
        for (_, mut worker) in query.iter_mut() {
            worker.action = SoldierAction::MoveToPosition {
                target: **cursor,
                attack_move: false,
            };
        }
    }
}

pub fn next_action(mut query: Query<(&mut Soldier, &mut Unit, &Transform)>, time: Res<Time>) {
    for (mut soldier, mut unit, transform) in query.iter_mut() {
        let pos = transform.translation.truncate();
        match soldier.action {
            SoldierAction::Idle => {
                unit.target_direction = Vec2::ZERO;
            }
            SoldierAction::MoveToPosition {
                target,
                attack_move,
            } => {
                let delta = target - pos;
                unit.target_direction = delta;

                if delta.length_squared() < 25.0 * 25.0 {
                    soldier.action = SoldierAction::Idle;
                }
            }
            _ => {
                soldier.action = SoldierAction::Idle;
            }
        }
    }
}

pub fn calculate_damage(weapon: Weapon) -> f32 {
    match weapon {
        Weapon::Sword => 3.0,
        Weapon::Axe => 4.0,
        Weapon::Spear => 2.0,
        Weapon::Bow => 2.0,
        Weapon::Sling => 1.0,
        Weapon::Crossbow => 3.0,
    }
}

pub fn calculate_range(weapon: Weapon) -> f32 {
    match weapon {
        Weapon::Sword => 15.0,
        Weapon::Axe => 15.0,
        Weapon::Spear => 50.0,
        Weapon::Bow => 1000.0,
        Weapon::Sling => 500.0,
        Weapon::Crossbow => 1500.0,
    }
}
