use bevy::{asset::AssetPath, prelude::*};

use crate::{
    unit::{SelectedMark, SelectionBox, Unit},
    util::{add_texture_atlas, load_image},
    Cull2D, Cursor, SpriteSheets, YSort,
};
pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSoldierEvent>()
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

pub fn soldier_spawn(
    mut spawn_event: EventReader<SpawnSoldierEvent>,
    mut commands: Commands,
    sprite_sheets: Res<SpriteSheets>,
) {
    for event in spawn_event.iter() {
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: sprite_sheets.swordsman_red.clone(),
                transform: Transform::from_translation(event.0.extend(1.0)),
                ..default()
            })
            .insert(Name::new("Soldier"))
            .insert(YSort)
            .insert(Cull2D)
            .insert(Unit::default())
            .insert(Soldier::default())
            .with_children(|builder| {
                builder
                    .spawn(SpriteSheetBundle {
                        texture_atlas: sprite_sheets.box_selector.clone(),
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
