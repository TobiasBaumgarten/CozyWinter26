use bevy::{input::mouse::MouseButtonInput, prelude::*};
use bevy_embedded_assets::EmbeddedAssetPlugin;

use crate::{define_upgrades::get_upgrades, forest::ForestPlugin, shop::ShopPlugin};

mod define_upgrades;
mod forest;
mod shop;

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

impl PlayerStats {
    fn get_value(&self, nut_type: &NutType) -> i32 {
        self.base_nut_value
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
    // fn new() -> Self {
    //     Self {
    //         increase_value: |upgrade, stats, money| {
    //             if upgrade.rise_up_count(money).is_some() {
    //                 stats.dmg += 2.;
    //                 upgrade.cost += 1;
    //             }
    //         },
    //         ..Default::default()
    //     }
    // }

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
    cube_max_life: f32,
    size: f32,
    nut_base_life: f32,
    dir: Vec2,
    base_nut_value: i32,
    respawn_nuts: i32,
    start_nuts: i32,
    nuts_respawn_time: f32,
}

#[derive(Resource, Debug)]
struct UpgradeList(Vec<UpgradeType>);

#[derive(Debug, Resource, Clone)]
struct Money(i32);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // init money
    let money = Money(0);
    commands.insert_resource(money);

    // init player
    let player_stats = PlayerStats {
        dmg: 50.,
        laser_length: 500.,
        cube_max_life: 200.,
        size: 16.,
        dir: Vec2::new(0., 0.),
        nut_base_life: 100.,
        base_nut_value: 1,
        nuts_respawn_time: 5.,
        respawn_nuts: 1,
        start_nuts: 0,
    };
    commands.insert_resource(player_stats.clone());

    commands.spawn((
        Text2d::new("Reflect The Laser\nWith You Slippery Ice Cupe\nTo Get The Nuts"),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., 0., 0.),
    ));

    let upgrades = get_upgrades();
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
