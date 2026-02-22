use bevy::{input::mouse::MouseButtonInput, prelude::*};
use bevy_embedded_assets::EmbeddedAssetPlugin;

use crate::{forest::ForestPlugin, shop::ShopPlugin};

mod forest;
mod shop;

impl PlayerStats {
    fn get_value(&self, nut_type: &NutType) -> i32 {
        self.base_value
            * match nut_type {
                NutType::Base => 1,
                NutType::Bronze => 10,
                NutType::Silver => 20,
                NutType::Gold => 50,
                NutType::Diamant => 100,
            }
    }
}

use std::sync::atomic::{AtomicUsize, Ordering};

// A global counter that starts at 0
static UPGRADE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone)]
struct UpgradeType {
    id: usize,
    cost: i32,
    title: String,
    value_hint: String,
    cur_up_count: i32,
    max_up_count: i32,
    increase_value: fn(&mut Self, &mut PlayerStats, &mut Money),
}

impl UpgradeType {
    fn new() -> Self {
        Self {
            increase_value: |upgrade, stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    stats.dmg += 2.;
                    upgrade.cost += 1;
                }
            },
            ..Default::default()
        }
    }

    fn rise_up_count(&mut self, money: &mut Money) -> Option<()> {
        if self.cur_up_count >= self.max_up_count {
            return None;
        }
        if money.0 < self.cost {
            return None;
        }
        money.0 -= self.cost;
        println!("Upgraded '{}'", self.title);
        self.cur_up_count += 1;
        Some(())
    }
}

impl Default for UpgradeType {
    fn default() -> Self {
        // fetch_add returns the PREVIOUS value and increments it by 1
        let new_id = UPGRADE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);

        Self {
            id: new_id,
            title: "Undefined".into(),
            value_hint: "+0".into(),
            cost: 1,
            cur_up_count: 0,
            max_up_count: 5,
            increase_value: |upgrade, stats, _money| {
                // Note: Logic here usually depends on the specific upgrade type
                stats.dmg += 2.0;
                upgrade.cost += 1;
            },
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
    dmg: f32,
    laser_length: f32,
    lifes: i32,
    cube_max_life: f32,
    size: f32,
    life: f32,
    dir: Vec2,
    base_value: i32,
    start_nuts: i32,
    nuts_respawn_time: f32,
}

#[derive(Resource, Debug)]
struct UpgradeList(Vec<UpgradeType>);

#[derive(Debug, Resource, Clone)]
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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // init money
    let money = Money(0);
    commands.insert_resource(money);

    // init player
    let player_stats = PlayerStats {
        dmg: 50.,
        laser_length: 500.,
        lifes: 1,
        cube_max_life: 200.,
        size: 16.,
        dir: Vec2::new(0., 0.),
        life: 100.,
        base_value: 1,
        nuts_respawn_time: 100.,
        start_nuts: 2,
    };
    commands.insert_resource(player_stats.clone());

    commands.spawn((
        Text2d::new("Reflect The Laser\nWith You Slippery Ice Cupe\nTo Get The Nuts"),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., 0., 0.),
    ));

    let mut upgrades: Vec<UpgradeType> = vec![];

    // define upgrades
    {
        // damage upgrade
        upgrades.push(UpgradeType {
            title: "Damgae".into(),
            value_hint: "+5".into(),
            cost: 1,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.dmg += 5.;
                    upgrade.cost += 1;
                }
            },
            ..Default::default()
        });
    }

    commands.insert_resource(UpgradeList(upgrades));
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
