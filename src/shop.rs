use bevy::{
    input::{ButtonState, mouse::MouseButtonInput},
    prelude::*,
};

use crate::{GameState, Money, PlayerStats};

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shoping), setup_shop)
            .add_message::<ButtonClickedMessage>()
            .add_message::<MoneyLabelUpdatedMessage>()
            .add_systems(
                Update,
                (
                    check_buttons,
                    read_new_round_button,
                    read_upgrade_button,
                    update_money_label,
                )
                    .chain()
                    .run_if(in_state(GameState::Shoping)),
            )
            .add_systems(OnExit(GameState::Shoping), on_leave);
    }
}

#[derive(Message, Debug)]
struct ButtonClickedMessage(Entity);

#[derive(Message, Debug)]
struct MoneyLabelUpdatedMessage(i32);

#[derive(Component, Debug)]
struct Button(Vec2);

#[derive(Component, Debug)]
struct MoneyLabelComponent;

#[derive(Component, Debug)]
struct NewRound;

#[derive(Component, Debug)]
struct UpgradeComponent {
    times: i32,
    upgrade_type: UpgradeType,
}

#[derive(Debug)]
enum UpgradeType {
    Damage,
    LaserLength,
}

#[derive(Component, Debug)]
struct ShopComponent;

fn setup_shop(mut commands: Commands, money: Res<Money>) {
    commands.spawn((
        Text2d::new("Shop"),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., 350., 0.),
        ShopComponent,
    ));

    let text = format!("Money: {}", money.0);
    commands.spawn((
        Text2d::new(text),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., 320., 0.),
        ShopComponent,
        MoneyLabelComponent,
    ));

    commands.spawn((
        Text2d::new("New Round"),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., -320., 0.),
        Button(Vec2::new(50., 20.)),
        NewRound,
        ShopComponent,
    ));

    commands.spawn((
        UpgradeComponent {
            times: 0,
            upgrade_type: UpgradeType::Damage,
        },
        Text2d::new("Damage"),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., 0., 0.),
        Button(Vec2::new(50., 20.)),
        ShopComponent,
    ));
}

fn check_buttons(
    mut mouse_msg: MessageReader<MouseButtonInput>,
    query: Query<(Entity, &GlobalTransform, &Button)>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mut clicked_writer: MessageWriter<ButtonClickedMessage>,
) {
    for msg in mouse_msg.read() {
        if !(msg.button == MouseButton::Left) {
            continue;
        }

        if let ButtonState::Pressed = msg.state {
            // get the position
            let Some(cursor_pos) = window.cursor_position() else {
                return;
            };
            let (camera, camera_trans) = *camera;
            let Ok(world_pos) = camera.viewport_to_world_2d(camera_trans, cursor_pos) else {
                continue;
            };

            for (entity, trans, button) in &query {
                let pos = trans.translation().truncate();
                let diff = (world_pos - pos).abs();
                if diff.x < button.0.x && diff.y < button.0.y {
                    clicked_writer.write(ButtonClickedMessage(entity));
                }
            }
        }
    }
}

fn read_new_round_button(
    mut reader: MessageReader<ButtonClickedMessage>,
    mut commands: Commands,
    query: Query<&NewRound>,
) {
    for msg in reader.read() {
        if let Ok(_) = query.get(msg.0) {
            commands.set_state(GameState::Playing);
        }
    }
}

fn read_upgrade_button(
    mut reader: MessageReader<ButtonClickedMessage>,
    mut commands: Commands,
    mut query: Query<&mut UpgradeComponent>,
    mut player_stats: ResMut<PlayerStats>,
    mut money: ResMut<Money>,
    mut writer: MessageWriter<MoneyLabelUpdatedMessage>,
) {
    for msg in reader.read() {
        if let Ok(mut upgrade) = query.get_mut(msg.0) {
            if money.0 <= upgrade.times {
                println!("Cannot upgrade");
                return;
            }
            println!("Should cost {}", upgrade.times);
            upgrade.times += 1;
            money.0 -= upgrade.times;
            writer.write(MoneyLabelUpdatedMessage(money.0));
            match upgrade.upgrade_type {
                UpgradeType::Damage => player_stats.dmg += 10.,
                UpgradeType::LaserLength => player_stats.laser_length += 20.,
            }
        }
    }
}

fn update_money_label(
    mut reader: MessageReader<MoneyLabelUpdatedMessage>,
    mut label: Single<&mut Text2d, With<MoneyLabelComponent>>,
) {
    for msg in reader.read() {
        let text = format!("Nuts: {}", msg.0);
        label.0 = text;
    }
}

fn on_leave(query: Query<Entity, With<ShopComponent>>, mut commands: Commands) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}
