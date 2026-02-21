use bevy::{
    ecs::relationship::RelationshipSourceCollection, input::mouse::MouseButtonInput, prelude::*,
};
use bevy_embedded_assets::EmbeddedAssetPlugin;

use crate::{forest::ForestPlugin, shop::ShopPlugin};

mod forest;
mod shop;

#[derive(Debug, Resource)]
struct NutStats {
    size: UpgradeType<f32>,
    life: UpgradeType<f32>,
    dir: UpgradeType<Vec2>,
    base_value: UpgradeType<i32>,
    start_nuts: UpgradeType<i32>,
    nuts_respawn_time: UpgradeType<f32>,
}

#[derive(Debug, Clone)]
struct UpgradeType<T> {
    value: T,
    cur_count: i32,
    max_count: i32,
    increase_value: fn(i32) -> i32,
}

impl<T> UpgradeType<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            cur_count: 0,
            max_count: 5,
            increase_value: |v| v + 1,
        }
    }
}

impl NutStats {
    fn get_value(&self, nut_type: &NutType) -> i32 {
        self.base_value.value
            * match nut_type {
                NutType::Base => 1,
                NutType::Bronze => 10,
                NutType::Silver => 20,
                NutType::Gold => 50,
                NutType::Diamant => 100,
            }
    }
}

#[derive(States, Debug, Eq, PartialEq, Hash, Clone)]
pub enum GameState {
    Start,
    Playing,
    Shoping,
}

#[allow(unused)]
#[derive(Debug, Component)]
enum NutType {
    Base,
    Bronze,
    Silver,
    Gold,
    Diamant,
}

#[derive(Debug, Resource, Clone)]
struct PlayerStats {
    dmg: UpgradeType<f32>,
    laser_length: UpgradeType<f32>,
    lifes: UpgradeType<i32>,
    cube_max_life: UpgradeType<f32>,
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
        .add_plugins((ForestPlugin, ShopPlugin))
        .insert_state(GameState::Start)
        .add_systems(Update, (start_game).run_if(in_state(GameState::Start)))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // init money
    commands.insert_resource(Money(0));

    // init player
    let player_stats = PlayerStats {
        dmg: UpgradeType::new(50.),
        laser_length: UpgradeType::new(500.),
        lifes: UpgradeType::new(1),
        cube_max_life: UpgradeType::new(200.),
    };
    commands.insert_resource(player_stats.clone());

    // init nut stats
    {
        let nut_stats = NutStats {
            size: UpgradeType::new(16.),
            dir: UpgradeType::new(Vec2::new(0., 0.)),
            life: UpgradeType::new(100.),
            base_value: UpgradeType::new(1),
            nuts_respawn_time: UpgradeType::new(100.),
            start_nuts: UpgradeType::new(2),
        };

        commands.insert_resource(nut_stats);
    }

    commands.spawn((
        Text2d::new("Reflect The Laser\nWith You Slippery Ice Cupe\nTo Get The Nuts"),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., 0., 0.),
    ));
}

fn start_game(
    mut mouse_msg: MessageReader<MouseButtonInput>,
    mut commands: Commands,
    query: Query<Entity, With<Text2d>>,
) {
    for _ in mouse_msg.read() {
        for entity in query {
            commands.entity(entity).despawn();
        }
        commands.set_state(GameState::Playing);
    }
}

// TODO Resouce that saves all the upgrade states
