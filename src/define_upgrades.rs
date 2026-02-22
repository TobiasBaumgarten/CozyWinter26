use crate::UpgradeType;

pub fn get_upgrades() -> Vec<UpgradeType> {
    let mut upgrades: Vec<UpgradeType> = vec![];

    // define upgrades
    {
        // damage upgrade
        upgrades.push(UpgradeType {
            title: "Damage".into(),
            value_hint: "+ 5".into(),
            cost: 1,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.dmg += 5.;
                }
            },
            ..Default::default()
        });

        // second damage upgrade
        upgrades.push(UpgradeType {
            title: "Damage".into(),
            value_hint: "+ 20".into(),
            cost: 5,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.dmg += 20.;
                    upgrade.cost += 2;
                }
            },
            ..Default::default()
        });

        // laser length upgrade
        upgrades.push(UpgradeType {
            title: "Laser Length".into(),
            value_hint: "+ 25".into(),
            cost: 1,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.laser_length += 25.;
                    upgrade.cost += 1;
                }
            },
            ..Default::default()
        });

        // max life upgrade
        upgrades.push(UpgradeType {
            title: "Max Life".into(),
            value_hint: "+ 20".into(),
            max_up_count: 30,
            cost: 1,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.cube_max_life += 20.;
                    upgrade.cost += upgrade.cur_up_count / 10;
                }
            },
            ..Default::default()
        });

        // dopple health
        upgrades.push(UpgradeType {
            title: "Max Life".into(),
            value_hint: "x 2".into(),
            max_up_count: 5,
            cost: 15,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.cube_max_life *= 2.;
                    upgrade.cost += 10;
                }
            },
            ..Default::default()
        });

        // start nuts upgrade
        upgrades.push(UpgradeType {
            title: "Start Nuts".into(),
            value_hint: "+ 1".into(),
            cost: 2,
            max_up_count: 10,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.start_nuts += 1;
                    upgrade.cost += 2 + upgrade.cur_up_count;
                }
            },
            ..Default::default()
        });

        // respawn time upgrade
        upgrades.push(UpgradeType {
            title: "Respawn Time".into(),
            value_hint: "- 0.2".into(),
            cost: 2,
            max_up_count: 15,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.nuts_respawn_time -= 0.2;
                    upgrade.cost += 2 + upgrade.cur_up_count * 2;
                }
            },
            ..Default::default()
        });

        // respawn nuts upgrade
        upgrades.push(UpgradeType {
            title: "Respawn Nuts".into(),
            value_hint: "+ 1".into(),
            cost: 2,
            max_up_count: 15,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.respawn_nuts += 1;
                    upgrade.cost += 2 + upgrade.cur_up_count * 2;
                }
            },
            ..Default::default()
        });

        // nut helth reduce
        upgrades.push(UpgradeType {
            title: "Nut Life".into(),
            value_hint: "- 5".into(),
            cost: 3,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.nut_base_life -= 5.;
                    upgrade.cost += 1;
                }
            },
            ..Default::default()
        });

        // nut worth
        upgrades.push(UpgradeType {
            title: "Nut Worth".into(),
            value_hint: "+ 1".into(),
            max_up_count: 15,
            cost: 3,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.base_nut_value += 1;
                    upgrade.cost += 2;
                }
            },
            ..Default::default()
        });

                // nut worth
        upgrades.push(UpgradeType {
            title: "Nut Worth".into(),
            value_hint: "x 2".into(),
            max_up_count: 5,
            cost: 20,
            increase_value: |upgrade, player_stats, money| {
                if upgrade.rise_up_count(money).is_some() {
                    player_stats.base_nut_value *= 2;
                    upgrade.cost += 10;
                }
            },
            ..Default::default()
        });

        // TODO
        //
    }

    upgrades
}
