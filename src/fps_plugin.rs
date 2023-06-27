use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct FpsPlugin;

#[derive(Component)]
struct FpsMarker;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            .add_startup_system(plugin_init)
            .add_system(text_change);
    }
}
fn plugin_init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            text: Text::from_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "fps = ?",
                TextStyle {
                    font: asset_server.load("fonts/roboto_regular.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
            ),
            ..default()
        })
        .insert(FpsMarker);
}

fn text_change(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsMarker>>) {
    let mut fps = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps_diagnostic| fps_diagnostic.average())
        .unwrap_or(0.0);

    // println!("fps={:.1}", fps);
    query.single_mut().sections[0].value = format!("fps = {:.1}", fps)
}
