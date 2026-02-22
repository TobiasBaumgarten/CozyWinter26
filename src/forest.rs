use std::{f32::consts::PI, time::Duration};

use bevy::{
    math::{VectorSpace, ops::sin},
    prelude::*,
};
use rand::RngExt;

use crate::{GameState, Money, NutType, PlayerStats};

const HALF_SIZE_CUBE: f32 = 16.;
const GRAVITY: Vec2 = Vec2::new(0., 70.);
const HALF_SIZE_SPAWN_FRAME: Vec2 = Vec2::new(300., 200.);

pub struct ForestPlugin;

impl Plugin for ForestPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SpawnNutMessage>()
            .add_message::<DeadPlayerMessage>()
            .add_systems(OnEnter(GameState::Playing), setup_forest)
            .add_systems(
                Update,
                (
                    update_cube,
                    collide_laser_cube,
                    draw_laser,
                    update_respawn_nuts,
                    spawn_nuts,
                    handle_sprite_state_nut,
                    handle_sprite_state_player,
                    handle_falling,
                    handle_dead_cubes,
                    on_dead,
                    check_end_timer,
                )
                    .run_if(in_state(GameState::Playing))
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
}

#[derive(Debug, Component)]
struct Nut;

#[derive(Debug, Component)]
struct Falling(Timer, Vec2);

#[derive(Debug, Component)]
struct EndScreenTimer(Timer);

#[derive(Debug, Component)]
struct RespawnNutsTimer(Timer);

#[derive(Debug, Component)]
struct PlayerCube {
    available_cubes: i32,
}
#[derive(Resource)]
struct HitCubeSound(Entity);

#[derive(Resource)]
struct HitNutSound(Entity);

#[derive(Resource)]
struct ReleaseNutSound(Entity);

#[derive(Debug, Component)]
struct IceAnimation;

#[derive(Debug, Message)]
pub struct SpawnNutMessage(Option<Vec2>);

#[derive(Debug, Message)]
pub struct DeadPlayerMessage;

#[derive(Debug, Clone, Resource)]
struct AnimationAtlasLayout(Handle<TextureAtlasLayout>);

#[derive(Debug, Resource)]
pub struct LaserPoints {
    pub source_start: Vec2,
    pub source_dir: Vec2,
    pub list: Vec<(Vec2, Vec2)>,
}

fn setup_forest(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut writer: MessageWriter<SpawnNutMessage>,
    player_stats: Res<PlayerStats>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    // background sprite
    {
        commands.spawn((
            DespawnOnExit(GameState::Playing),
            Sprite::from_image(asset_server.load("embedded://bg.png")),
            Transform::from_xyz(0., 0., -1.),
        ));
    }

    // init laser
    {
        let start = Vec2::new(-250., -200.);
        let direction = Vec2::new(1., 0.);
        let points = LaserPoints {
            source_start: start,
            source_dir: direction,
            list: vec![(start, start + direction * player_stats.laser_length)],
        };
        commands.insert_resource(points);
    }

    // create and insert atlas
    let atlas = TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None);
    let layout = texture_atlases.add(atlas);
    commands.insert_resource(AnimationAtlasLayout(layout.clone()));

    // init cube
    {
        let cube = asset_server.load("embedded://player_cube_sheet.png");

        commands.spawn((
            Sprite {
                image: cube.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: layout.clone(),
                    index: 0,
                }),
                ..Default::default()
            },
            Transform {
                rotation: Quat::from_rotation_z(30. / 180. * PI),
                translation: Vec3::new(0., 0., 0.),
                ..Default::default()
            },
            PlayerCube { available_cubes: 1 },
            Cube {
                size: HALF_SIZE_CUBE,
                life: player_stats.cube_max_life,
            },
            IceAnimation,
            DespawnOnExit(GameState::Playing),
        ));
    }

    // spawn the fist nut - this will spawn every time on the same position
    writer.write(SpawnNutMessage(Some(Vec2::ZERO)));

    // spawn beginning nuts
    {
        println!("Player start nuts -> {}", player_stats.start_nuts);
        for _ in 0..player_stats.start_nuts {
            writer.write(SpawnNutMessage(None));
        }
    }

    // spawn nuts timer
    {
        commands.spawn((
            DespawnOnExit(GameState::Playing),
            RespawnNutsTimer(Timer::from_seconds(
                player_stats.nuts_respawn_time,
                TimerMode::Repeating,
            )),
        ));
    }

    // spawn sound
    {
        // cube hit sound
        let sound_entity = commands
            .spawn((
                AudioPlayer::new(asset_server.load("embedded://cube_hit.wav")),
                PlaybackSettings::LOOP.paused(),
                DespawnOnExit(GameState::Playing),
            ))
            .id();

        commands.insert_resource(HitCubeSound(sound_entity));

        // nut hit sound
        let sound_entity = commands
            .spawn((
                AudioPlayer::new(asset_server.load("embedded://nut_hit.wav")),
                PlaybackSettings::LOOP.paused(),
                DespawnOnExit(GameState::Playing),
            ))
            .id();

        commands.insert_resource(HitNutSound(sound_entity));
    }
}

fn spawn_nuts(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut reader: MessageReader<SpawnNutMessage>,
    player_stats: Res<PlayerStats>,
    atlas_layout: Res<AnimationAtlasLayout>,
) {
    let nut: Handle<Image> = asset_server.load("embedded://nut.png");
    let ice: Handle<Image> = asset_server.load("embedded://ice_nut_sheet.png");
    // TODO: Set position
    // TODO: resize sprite

    for new_nut_pos in reader.read() {
        // TODO Random with a chance NutType
        let base_nut = NutType::Base;
        let pos = match new_nut_pos.0 {
            Some(v) => v,
            None => {
                let mut rng = rand::rng();
                let x = rng.random_range(-HALF_SIZE_SPAWN_FRAME.x..HALF_SIZE_SPAWN_FRAME.x);
                let y = rng.random_range(-HALF_SIZE_SPAWN_FRAME.y..HALF_SIZE_SPAWN_FRAME.y);
                Vec2::new(x, y)
            }
        };

        commands
            .spawn((
                Sprite::from_image(nut.clone()),
                Cube {
                    size: player_stats.size,
                    life: player_stats.nut_base_life,
                },
                Transform {
                    rotation: Quat::from_rotation_z(player_stats.dir.to_angle()),
                    translation: pos.extend(0.),
                    ..Default::default()
                },
                base_nut,
                DespawnOnExit(GameState::Playing),
            ))
            .with_child((
                IceAnimation,
                Sprite {
                    image: ice.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: atlas_layout.0.clone(),
                        index: 0,
                    }),
                    ..Default::default()
                },
                DespawnOnExit(GameState::Playing),
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

fn update_respawn_nuts(
    mut timer: Single<&mut RespawnNutsTimer>,
    mut writer: MessageWriter<SpawnNutMessage>,
    time: Res<Time>,
    stats: Res<PlayerStats>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() {
        for _ in 0..stats.respawn_nuts {
            writer.write(SpawnNutMessage(None));
        }
    }
}

fn collide_laser_cube(
    mut points: ResMut<LaserPoints>,
    mut cubes: Query<(
        &mut Cube,
        &Transform,
        Option<&mut PlayerCube>,
        Option<&NutType>,
    )>,
    time: Res<Time>,
    player_stats: Res<PlayerStats>,
    cube_hit_sound: ResMut<HitCubeSound>,
    nut_hit_sound: ResMut<HitNutSound>,
    audio_sinks: Query<&AudioSink>,
) {
    // sound pause default
    {
        if let Ok(sink) = audio_sinks.get(cube_hit_sound.0) {
            sink.pause();
        }
        if let Ok(sink) = audio_sinks.get(nut_hit_sound.0) {
            sink.pause();
        }
    }

    let start = points.source_start;
    let mut ray_start = start;
    let mut ray_dir = points.source_dir;
    let mut remaining = player_stats.laser_length;

    // Reset to single full line initially
    points.list = vec![(start, start + ray_dir * remaining)];

    // First pass: find PlayerCube and reflect
    for (mut cube, trans, is_player, is_nut_type) in cubes.iter_mut() {
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

            if is_nut_type.is_some() {
                // play nut hit sound
                if let Ok(sink) = audio_sinks.get(cube_hit_sound.0) {
                    sink.play();
                }
            }

            // handle player
            // TODO but change this later so that the laser can further s
            if is_player.is_some() {
                // play hit sound
                if let Ok(sink) = audio_sinks.get(cube_hit_sound.0) {
                    sink.play();
                }
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
    player_stats: Res<PlayerStats>,
    mut writer: MessageWriter<DeadPlayerMessage>,
) {
    for (entity, cube, player, nut_type) in query {
        if cube.life > 0. {
            continue;
        }

        // when a nut has zero life
        if let Some(nut_type) = nut_type {
            commands.entity(entity).remove::<Cube>();
            money.0 += player_stats.get_value(nut_type);

            commands.entity(entity).insert(Falling(
                Timer::new(Duration::new(1, 0), TimerMode::Once),
                Vec2::new(0., 0.),
            ));

            println!("Nuts: {}", money.0);
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
                color: Color::srgb(1., 0.5, 0.5),
                custom_size: Some(Vec2::new(length, thickness)),
                ..Default::default()
            },
            Transform {
                translation: center.extend(0.),
                rotation: Quat::from_rotation_z(angle),
                ..Default::default()
            },
            Laser,
            DespawnOnExit(GameState::Playing),
        ));
    }
}

fn handle_sprite_state_nut(
    query: Query<(&ChildOf, &mut Sprite), With<IceAnimation>>,
    parent_query: Query<&Cube>,
    player_stats: Res<PlayerStats>,
) {
    for (child_of, mut sprite) in query {
        let Ok(cube) = parent_query.get(child_of.parent()) else {
            continue;
        };
        let state_part_value = player_stats.nut_base_life as usize / 4;
        let state = (player_stats.nut_base_life - cube.life) as usize / (state_part_value);
        let state = state.clamp(0, 4);
        if let Some(_atlas) = &mut sprite.texture_atlas {
            _atlas.index = state;
            // println!("Atlas index: {}", state);
        };
    }
}

fn handle_sprite_state_player(
    single: Single<(&mut Sprite, &Cube), With<PlayerCube>>,
    player_stats: Res<PlayerStats>,
) {
    let (mut sprite, cube) = single.into_inner();
    let state_part_value = player_stats.cube_max_life as usize / 4;
    let state = (player_stats.cube_max_life - cube.life) as usize / (state_part_value);
    let state = state.clamp(0, 4);
    if let Some(_atlas) = &mut sprite.texture_atlas {
        _atlas.index = state;
    }
}

fn handle_falling(
    query: Query<(Entity, &mut Transform, &mut Falling)>,
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

fn on_dead(
    mut reader: MessageReader<DeadPlayerMessage>,
    mut commands: Commands,
    money: Res<Money>,
) {
    for _ in reader.read() {
        let timer = Timer::new(Duration::from_secs_f32(1.5), TimerMode::Once);
        let text = format!("Game Over\nYou Have {} Nuts", money.0);

        commands.spawn((
            EndScreenTimer(timer),
            Text2d::new(text),
            TextLayout::new(Justify::Center, LineBreak::NoWrap),
            Transform::from_xyz(0., 0., 0.),
            DespawnOnExit(GameState::Playing),
        ));
    }
}

fn check_end_timer(
    mut single: Single<&mut EndScreenTimer>,
    time: Res<Time>,
    mut commands: Commands,
) {
    single.0.tick(time.delta());

    if single.0.is_finished() {
        commands.set_state(GameState::Shoping);
    }
}

// TODO Some kind of feedback by hitting the nut

// Respawn timer
