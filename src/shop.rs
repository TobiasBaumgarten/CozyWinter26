use bevy::prelude::*;

use crate::{GameState, Money, PlayerStats, UpgradeList};

const UPGRADE_BUTTON_SIZE: Vec2 = Vec2::new(150., 80.);
const UPGRADE_FIELD_MARGIN: Vec2 = Vec2::new(100., 50.);

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Shoping), setup_shop)
            .add_message::<ButtonClickedMessage>()
            .add_message::<MoneyLabelUpdatedMessage>()
            .add_message::<SpawnUpgradeButtonMessage>()
            .add_message::<ChangedUpgradeState>()
            .add_systems(
                Update,
                (
                    check_buttons,
                    read_upgrade_button,
                    spawn_upgrade,
                    update_money_label,
                    update_changed_upgrade_ui,
                    update_available_upgrade_money,
                    read_new_round_button,
                )
                    .chain()
                    .run_if(in_state(GameState::Shoping)),
            );
    }
}

#[derive(Message, Debug)]
struct ButtonClickedMessage(Entity);

#[derive(Message, Debug)]
struct MoneyLabelUpdatedMessage(i32);

#[derive(Message, Debug)]
struct SpawnUpgradeButtonMessage {
    upgrade_id: usize,
}

#[derive(Message, Debug)]
struct ChangedUpgradeState {
    upgrade_id: usize,
}

#[derive(Component, Debug)]
struct UpgradeNodeParent;

#[derive(Component, Debug)]
struct MoneyLabelComponent;

#[derive(Component, Debug)]
struct NewRound;

#[derive(Component, Debug)]
struct UpgradeComponent(usize);

#[derive(Component, Debug)]
struct UpgradeCostLabel;

#[derive(Component, Debug)]
struct UpgradeUsesLabel;

#[derive(Component, Debug)]
struct UpgradeNode;

fn setup_shop(
    mut commands: Commands,
    money: Res<Money>,
    upgrades: Res<UpgradeList>,
    mut spawn_writer: MessageWriter<SpawnUpgradeButtonMessage>,
    mut money_writer: MessageWriter<MoneyLabelUpdatedMessage>,
) {
    commands.spawn((
        Text2d::new("Shop"),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., 350., 0.),
        DespawnOnExit(GameState::Shoping),
    ));

    let text = format!("Nuts: {}", money.0);
    commands.spawn((
        Text2d::new(text),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        Transform::from_xyz(0., 320., 0.),
        DespawnOnExit(GameState::Shoping),
        MoneyLabelComponent,
    ));

    money_writer.write(MoneyLabelUpdatedMessage(money.0));

    // new round button
    commands
        .spawn((
            DespawnOnExit(GameState::Shoping),
            Button,
            Node {
                width: Val::Px(UPGRADE_BUTTON_SIZE.x),
                height: Val::Px(50.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.),
                margin: UiRect::AUTO,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.2, 0.2)),
            NewRound,
        ))
        .with_child(Text::new("New Round"));

    commands.spawn((
        UpgradeNodeParent,
        DespawnOnExit(GameState::Shoping),
        Node {
            width: Val::Px(900.),
            height: Val::Px(400.),
            margin: UiRect::AUTO,
            top: Val::Px(UPGRADE_FIELD_MARGIN.y),
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row, // side by side
            flex_wrap: FlexWrap::Wrap,          // wrap to next line when full
            align_content: AlignContent::Start,
            row_gap: Val::Px(25.),    // space between rows
            column_gap: Val::Px(25.), // space between columns
            ..Default::default()
        },
    ));

    // spawn every update node
    for upgrade in upgrades.0.iter() {
        spawn_writer.write(SpawnUpgradeButtonMessage {
            upgrade_id: upgrade.id,
        });
    }
}

fn spawn_upgrade(
    mut reader: MessageReader<SpawnUpgradeButtonMessage>,
    mut commands: Commands,
    upgrades: Res<UpgradeList>,
    parent: Single<Entity, With<UpgradeNodeParent>>,
) {
    for upgrade in reader.read() {
        let id = upgrade.upgrade_id;
        let upgrade = match upgrades.0.iter().find(|u| u.id == id) {
            Some(val) => val,
            None => return,
        };

        commands.entity(*parent).with_children(|parent| {
            parent
                .spawn((
                    DespawnOnExit(GameState::Shoping),
                    UpgradeComponent(id),
                    UpgradeNode,
                    Button,
                    Node {
                        width: Val::Px(UPGRADE_BUTTON_SIZE.x),
                        height: Val::Px(UPGRADE_BUTTON_SIZE.y),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::End,
                        padding: UiRect::all(Val::Px(10.)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                ))
                .with_children(|parent_field| {
                    // draw the cost
                    parent_field.spawn((
                        UpgradeComponent(id),
                        UpgradeCostLabel,
                        Text::new(format!("{}", upgrade.cost)),
                        TextFont::from_font_size(14.),
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(2.0),
                            left: Val::Px(2.0),
                            ..default()
                        },
                    ));
                    // draw the available upgrades
                    parent_field.spawn((
                        UpgradeComponent(id),
                        UpgradeUsesLabel,
                        Text::new(format!("{}/{}", upgrade.cur_up_count, upgrade.max_up_count)),
                        TextFont::from_font_size(14.),
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(2.0),
                            right: Val::Px(2.0),
                            ..default()
                        },
                    ));

                    // draw title
                    parent_field.spawn((
                        Text::new(format!("{}\n{}", upgrade.title, upgrade.value_hint)),
                        TextFont::from_font_size(18.),
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                    ));
                });
        });
    }
}

fn check_buttons(
    query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut clicked_writer: MessageWriter<ButtonClickedMessage>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for (entity, interaction) in &query {
        if *interaction == Interaction::Pressed {
            let click_sound = asset_server.load("embedded://button.wav");
            commands.spawn((AudioPlayer::new(click_sound), PlaybackSettings::DESPAWN));
            clicked_writer.write(ButtonClickedMessage(entity));
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
            println!("NExt Round");
            commands.set_state(GameState::Playing);
        }
    }
}

fn read_upgrade_button(
    mut reader: MessageReader<ButtonClickedMessage>,
    mut query: Query<&UpgradeComponent, With<UpgradeNode>>,
    mut player_stats: ResMut<PlayerStats>,
    mut money: ResMut<Money>,
    mut money_writer: MessageWriter<MoneyLabelUpdatedMessage>,
    mut upgrade_list: ResMut<UpgradeList>,
    mut changed_upgrade_writer: MessageWriter<ChangedUpgradeState>,
) {
    for msg in reader.read() {
        if let Ok(upgrade_comp) = query.get_mut(msg.0) {
            println!("Action! id:-");
            if let Some(upgrade) = upgrade_list.0.iter_mut().find(|u| u.id == upgrade_comp.0) {
                let stats = &mut *player_stats;
                let mon = &mut *money;
                (upgrade.increase_value)(upgrade, stats, mon);
                money_writer.write(MoneyLabelUpdatedMessage(mon.0));
                changed_upgrade_writer.write(ChangedUpgradeState {
                    upgrade_id: upgrade.id,
                });
            } else {
                println!("Cannot upgrade id: {}", upgrade_comp.0);
            }
        }
    }
}

fn update_changed_upgrade_ui(
    mut changed_upgrade_reader: MessageReader<ChangedUpgradeState>,
    mut query: Query<(
        &UpgradeComponent,
        &mut Text,
        Option<&UpgradeCostLabel>,
        Option<&UpgradeUsesLabel>,
    )>,
    upgrade_list: Res<UpgradeList>,
) {
    for change in changed_upgrade_reader.read() {
        for (up_comp, mut text, is_cost, is_uses) in query.iter_mut() {
            if up_comp.0 != change.upgrade_id {
                continue;
            }

            let upgrade = match upgrade_list.0.iter().find(|u| u.id == change.upgrade_id) {
                Some(v) => v,
                None => continue,
            };

            if is_cost.is_some() {
                text.0 = format!("{}", upgrade.cost);
            }

            if is_uses.is_some() {
                text.0 = format!("{}/{}", upgrade.cur_up_count, upgrade.max_up_count);
            }
        }
    }
}

fn update_available_upgrade_money(
    mut reader: MessageReader<MoneyLabelUpdatedMessage>,
    mut query: Query<(&UpgradeComponent, &mut BackgroundColor), With<UpgradeNode>>,
    upgrade_list: Res<UpgradeList>,
) {
    for money in reader.read() {
        for (up_comp, mut back) in query.iter_mut() {
            let upgrade = match upgrade_list.0.iter().find(|u| u.id == up_comp.0) {
                Some(v) => v,
                None => continue,
            };

            if upgrade.cur_up_count >= upgrade.max_up_count {
                back.0 = Color::srgb(0.1, 0.1, 0.1);
            } else if money.0 >= upgrade.cost {
                back.0 = Color::srgb(0.2, 0.3, 0.2);
            } else {
                back.0 = Color::srgb(0.3, 0.2, 0.2);
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
