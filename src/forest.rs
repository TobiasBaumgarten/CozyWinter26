use std::{f32::consts::PI, time::Duration};

use bevy::{math::ops::sin, prelude::*};

use crate::{Money, NutStats, PlayerStats, setup};

const HALF_SIZE_CUBE: f32 = 16.;
const GRAVITY: Vec2 = Vec2::new(0., 70.);

pub struct ForestPlugin;

impl Plugin for ForestPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnNutMessage>()
            .add_message::<DeadPlayerMessage>()
            .add_systems(Startup, setup_forest.after(setup))
            .add_systems(
                Update,
                (
                    update_cube,
                    collide_laser_cube,
                    draw_laser,
                    spawn_nuts,
                    handle_falling,
                    handle_dead_cubes,
                )
                    .chain(),
            );
    }
}

#[derive(Debug, Component)]
struct Laser;

#[derive(Debug, Component)]
struct Cube {
    size: f32,
    life: f32,
    dir: Vec2,
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

#[derive(Debug, Component)]
struct Falling(Timer, Vec2);

#[derive(Debug, Component)]
struct PlayerCube {
    available_cubes: i32,
}

#[derive(Debug, Message)]
pub struct SpawnNutMessage;

#[derive(Debug, Message)]
pub struct DeadPlayerMessage;

#[derive(Debug, Resource)]
pub struct LaserPoints {
    pub source_start: Vec2,
    pub source_dir: Vec2,
    pub list: Vec<(Vec2, Vec2)>,
    pub length: f32,
}

fn setup_forest(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut writer: MessageWriter<SpawnNutMessage>,
    player_stats: Res<PlayerStats>,
) {
    // init laser
    {
        let start = Vec2::new(-250., -200.);
        let direction = Vec2::new(1., 0.);
        let points = LaserPoints {
            source_start: start,
            source_dir: direction,
            list: vec![(start, start + direction * player_stats.laser_length)],
            length: 500.,
        };
        commands.insert_resource(points);
    }

    // init cube
    {
        let cube = asset_server.load("embedded://ice_cube.png");

        commands.spawn((
            Sprite::from_image(cube),
            Transform {
                rotation: Quat::from_rotation_z(30. / 180. * PI),
                translation: Vec3::new(0., 0., 0.),
                ..Default::default()
            },
            PlayerCube { available_cubes: 1 },
            Cube {
                size: HALF_SIZE_CUBE,
                dir: Vec2::new(1., 0.),
                life: 300.,
            },
        ));
    }

    writer.write(SpawnNutMessage);
}

fn spawn_nuts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut reader: MessageReader<SpawnNutMessage>,
    nut_stats: Res<NutStats>,
) {
    let nut: Handle<Image> = asset_server.load("embedded://frozen_nut.png");
    // TODO: Set position
    // TODO: resize sprite

    for _ in reader.read() {
        // TODO Random with a chance NutType
        let base_nut = NutType::Base;

        commands.spawn((
            base_nut,
            Cube {
                size: nut_stats.size,
                life: nut_stats.life,
                dir: nut_stats.dir,
            },
            Sprite::from_image(nut.clone()),
            Transform {
                rotation: Quat::from_rotation_z(nut_stats.dir.to_angle()),
                translation: Vec3::new(0., 0., 0.),
                ..Default::default()
            },
        ));
    }
}

fn update_cube(
    mut cube: Single<&mut Transform, With<PlayerCube>>,
    mut cursor_event: MessageReader<CursorMoved>,
    camera: Single<(&Camera, &GlobalTransform)>,
    time: Res<Time>,
) {
    let (camera, camera_trans) = *camera;
    for cursor_moved in cursor_event.read() {
        let window_mouse_pos = cursor_moved.position;

        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_trans, window_mouse_pos) {
            cube.translation = world_pos.extend(0.);
            cube.rotate(Quat::from_rotation_z(sin(time.delta_secs())));
        }
    }
}

fn collide_laser_cube(
    mut points: ResMut<LaserPoints>,
    mut cubes: Query<(&mut Cube, &Transform, Option<&mut PlayerCube>)>,
    time: Res<Time>,
    player_stats: Res<PlayerStats>,
) {
    let start = points.source_start;
    let mut ray_start = start;
    let mut ray_dir = points.source_dir;
    let mut remaining = player_stats.laser_length;

    // Reset to single full line initially
    points.list = vec![(start, start + ray_dir * remaining)];

    // First pass: find PlayerCube and reflect
    for (mut cube, trans, is_player) in cubes.iter_mut() {
        let cube_matrix = trans.to_matrix();
        let world_to_local = cube_matrix.inverse();

        let ray_end = ray_start + ray_dir * remaining;
        let local_start = world_to_local
            .transform_point3(ray_start.extend(0.0))
            .truncate();
        let local_end = world_to_local
            .transform_point3(ray_end.extend(0.0))
            .truncate();

        if let Some((hit_pos_local, local_normal)) =
            ray_rect_intersection(local_start, local_end, HALF_SIZE_CUBE)
        {
            let hit_pos_world = cube_matrix
                .transform_point3(hit_pos_local.extend(0.0))
                .truncate();

            cube.life -= time.delta_secs() * player_stats.dmg;

            // handle player
            // TODO but change this later so that the laser can further s
            if is_player.is_some() {
                // Reflect the ray
                let world_normal = (trans.rotation * local_normal.extend(0.0))
                    .truncate()
                    .normalize();
                let reflect_dir = ray_dir - 2.0 * ray_dir.dot(world_normal) * world_normal;

                remaining -= (ray_start - hit_pos_world).length();
                let reflect_end = hit_pos_world + reflect_dir * remaining;

                points.list = vec![(start, hit_pos_world), (hit_pos_world, reflect_end)];

                ray_start = hit_pos_world;
                ray_dir = reflect_dir;
            } else {
                // Nut: stop the ray here
                points.list.last_mut().unwrap().1 = hit_pos_world;
                break;
            }
        }
    }
}

fn handle_dead_cubes(
    query: Query<(Entity, &Cube, Option<&mut PlayerCube>, Option<&NutType>)>,
    mut commands: Commands,
    mut money: ResMut<Money>,
    nut_stats: Res<NutStats>,
    mut writer: MessageWriter<DeadPlayerMessage>,
) {
    for (entity, cube, player, nut_type) in query {
        if cube.life > 0. {
            continue;
        }

        // when a nut has zero life
        if let Some(nut_type) = nut_type {
            commands.entity(entity).remove::<Cube>();
            let base = nut_stats.base_value;
            money.0 += base
                * match nut_type {
                    NutType::Base => 1,
                    NutType::Bronze => 10,
                    NutType::Silver => 20,
                    NutType::Gold => 50,
                    NutType::Diamant => 100,
                };

            commands
                .entity(entity)
                .insert(Falling(Timer::new(Duration::new(1, 0), TimerMode::Once),Vec2::new(0.,0.)));

            println!("Money: {}", money.0);
            continue; 
        }

        // when the player has zero life
        if let Some(mut _player) = player {
            _player.available_cubes -= 1;
            if _player.available_cubes < 1 {
                // when the player has zero cubes left
                writer.write(DeadPlayerMessage);
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Returns the (intersection_point, normal) if the line hits the cube
fn ray_rect_intersection(p0: Vec2, p1: Vec2, size: f32) -> Option<(Vec2, Vec2)> {
    let d = p1 - p0;
    let mut t_near = -f32::INFINITY;
    let mut t_far = f32::INFINITY;
    let mut normal = Vec2::ZERO;

    for i in 0..2 {
        if d[i].abs() < f32::EPSILON {
            if p0[i].abs() > size {
                return None;
            }
        } else {
            let mut t1 = (-size - p0[i]) / d[i];
            let mut t2 = (size - p0[i]) / d[i];
            let mut n = if d[i] > 0.0 {
                Vec2::new(
                    if i == 0 { -1.0 } else { 0.0 },
                    if i == 1 { -1.0 } else { 0.0 },
                )
            } else {
                Vec2::new(
                    if i == 0 { 1.0 } else { 0.0 },
                    if i == 1 { 1.0 } else { 0.0 },
                )
            };

            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
                n *= -1.0;
            }
            if t1 > t_near {
                t_near = t1;
                normal = n;
            }
            t_far = t_far.min(t2);
        }
    }

    if t_near <= t_far && t_near >= 0.0 && t_near <= 1.0 {
        Some((p0 + d * t_near, normal))
    } else {
        None
    }
}

/// Draw the laser points
fn draw_laser(
    mut commands: Commands,
    lines: Res<LaserPoints>,
    old_lines: Query<Entity, With<Laser>>,
) {
    // Despawn ALL old lines FIRST, outside the segment loop
    for line in &old_lines {
        commands.entity(line).despawn();
    }

    for (start, end) in lines.list.iter() {
        let thickness = 5.0;
        let dir = end - start;
        let center = start + (dir / 2.0);
        let length = dir.length();
        let angle = dir.y.atan2(dir.x);

        commands.spawn((
            Sprite {
                color: Color::srgb(1., 1., 1.),
                custom_size: Some(Vec2::new(length, thickness)),
                ..Default::default()
            },
            Transform {
                translation: center.extend(0.),
                rotation: Quat::from_rotation_z(angle),
                ..Default::default()
            },
            Laser,
        ));
    }
}

fn handle_falling(
    mut query: Query<(Entity, &mut Transform, &mut Falling)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut trans, mut falling) in query {
        falling.0.tick(time.delta());

        if falling.0.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        falling.1 += GRAVITY * time.delta_secs();
        trans.translation -= falling.1.extend(0.);
    }
}

// TODO Some kind of feedback by hitting the nut
// TODO Change texture as state of nut

// TODO Dead Screen where we show the money

// TODO Switch to Upgrade Menu

// Respawn timer
