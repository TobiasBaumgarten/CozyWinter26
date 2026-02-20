use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;

use crate::forest::ForestPlugin;

mod forest;

#[derive(Debug, Resource)]
struct NutStats {
    size: f32,
    life: f32,
    dir: Vec2,
    base_value: i32,
}

#[derive(Debug, Resource, Clone)]
struct PlayerStats {
    dmg: f32,
    laser_length: f32,
}

#[derive(Debug, Resource)]
struct Money(i32);

fn main() {
    let window = WindowPlugin {
        primary_window: Some(Window {
            title: "Cozy Winter Game by Beside Central".into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    App::new()
        .add_plugins((EmbeddedAssetPlugin::default(), DefaultPlugins.set(window)))
        .add_systems(Startup, setup)
        .add_plugins(ForestPlugin)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // init money
    commands.insert_resource(Money(0));

    // init player
    let player_stats = PlayerStats {
        dmg: 50.,
        laser_length: 500.,
    };
    commands.insert_resource(player_stats.clone());


    // init nut stats
    {
        let nut_stats = NutStats {
            size: 32.,
            dir: Vec2::new(0., 0.),
            life: 100.,
            base_value: 1,
        };

        commands.insert_resource(nut_stats);
    }
}
