use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct FpsPlugin;

#[derive(Component)]
struct FpsMarker;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_startup_system(plugin_init)
            .add_system(text_change);
    }
}
fn plugin_init(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexStart,
                position_type: PositionType::Absolute,
                position: Rect {
                    right: Val::Px(10.0),
                    top: Val::Px(10.0),
                    ..default()
                },
                ..default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "fps = ?",
                TextStyle {
                    font: asset_server.load("fonts/roboto_regular.ttf"),
                    font_size: 32.0,
                    color: Color::WHITE,
                },
                default()
            ),
            ..default()
        }).insert(FpsMarker);
}

fn text_change(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsMarker>>) {
    let mut fps = 0.0;
    if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps_avg) = fps_diagnostic.average() {
            fps = fps_avg;
        }
    }
    query.single_mut().sections[0].value = format!("fps = {:.1}", fps)
}

