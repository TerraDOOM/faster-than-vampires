use rand::prelude::*;
use std::{
    f32::consts::{PI, TAU},
    time::Duration,
};

use bevy::{color, time::Stopwatch};
use bullet::{
    BulletBundle, BulletCommandExt, HomingBullet, NormalBullet, RotatingBullet, StutterBullet,
    Target, WaveBullet,
};

use super::{
    bullet::{DelayedBullet, Velocity},
    *,
};

pub fn enemy_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Touhou), spawn_enemy)
        .insert_resource(EncounterTime {
            time: Stopwatch::new(),
        })
        .add_systems(Update, animate_sprites)
        .add_systems(
            OnEnter(GameState::Touhou),
            |mut time: ResMut<EncounterTime>| time.time.reset(),
        )
        .add_systems(
            FixedUpdate,
            (
                do_random_movement,
                circular_rotating_emitter,
                circular_homing_emitter,
                circular_wave_emitter,
                spray_emitter,
                tentacle_emitter,
                rotating_spray_emitter,
                divisive_emitter,
                flood_emitter,
                advance_encounter_time,
                process_spellcards,
            )
                .in_set(TouhouSets::Gameplay),
        );
}

#[derive(Resource)]
struct EncounterTime {
    time: Stopwatch,
}

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    sprite: Sprite,
    animation: Animation,
    transform: Transform,
    collider: Collider,
    health: Health,
    markers: (EnemyMarker, TouhouMarker, HasEmitters),
}

#[derive(Component, Default)]
pub struct EnemyMarker;

#[derive(Component, Deref, DerefMut, Default)]
pub struct Health(u32);

#[derive(Component, Default)]
struct HasEmitters;

#[derive(Component, Default)]
pub struct CircularAimedEmitter {
    offset: f32,
    count: usize,
}

#[derive(Component, Default)]
pub struct CircularHomingEmitter {
    offset: f32,
    count: usize,
    idx: usize,
}

#[derive(Component, Default)]
pub struct CircularWaveEmitter {
    offset: f32,
    count: usize,
    rotation: f32,
    rotation_speed: f32,
}

#[derive(Component, Default)]
pub struct TentacleEmitter {
    offset: f32,
    count: usize,
}

#[derive(Component, Default)]
pub struct FloodEmitter {
    spray: f32,
}

#[derive(Component, Default)]
pub struct SprayEmitter {
    spray_width: f32,
    firing_time: f32,
    firing_speed: f32,
    count: f32,
}

#[derive(Component, Default)]
pub struct RotatingSprayEmitter {
    spray_width: f32,
    firing_time: f32,
    firing_speed: f32,
    count: f32,
    rotation_speed: f32,
    rotation: f32,
    spray_count: usize,
}

#[derive(Component, Default)]
pub struct DivisiveEmitter {
    columns: u64,
    rows: u64,
}

#[derive(Component, Clone, Default, Debug)]
pub struct Animation {
    transition_time: Timer,
    max_index: usize,
    min_index: usize,
    index: usize,
}

impl Animation {
    pub fn new(time: f32, max_index: usize, min_index: usize) -> Self {
        Animation {
            transition_time: Timer::from_seconds(time, TimerMode::Repeating),
            max_index,
            min_index,
            index: min_index,
        }
    }
}

pub fn animate_sprites(time: Res<Time>, mut sprites: Query<(&mut Sprite, &mut Animation)>) {
    for (mut sprite, mut animation) in &mut sprites {
        let Some(mut atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };

        animation.transition_time.tick(time.delta());
        if animation.transition_time.just_finished() {
            atlas.index = (atlas.index + 1) % (animation.max_index - animation.min_index)
                + animation.min_index;
        }
    }
}

#[derive(Component, Default)]
pub struct Emitter {
    timer: Timer,
}

impl Emitter {
    pub fn new_secs(duration: f32) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(duration), TimerMode::Repeating),
        }
    }
}

#[derive(Component, Default, Deref, DerefMut)]
struct Active(bool);

#[derive(Component)]
struct Spellcard {
    emitters: Vec<Entity>,
    start_time: f32,
    end_time: f32,
}

fn advance_encounter_time(
    time: Res<Time>,
    mut enc_time: ResMut<EncounterTime>,
    spell_cards: Query<&Spellcard>,
) {
    enc_time.time.tick(time.delta());
    let current_time = enc_time.time.elapsed_secs();
    if spell_cards.iter().all(|card| card.end_time < current_time) {
        enc_time.time.reset();
    }
}

fn process_spellcards(
    enc_time: Res<EncounterTime>,
    cards: Query<&Spellcard>,
    mut emitters: Query<&mut Active, With<Emitter>>,
) {
    for mut active in &mut emitters {
        **active = false;
    }
    let current_time = enc_time.time.elapsed_secs();

    for card in &cards {
        if card.start_time < current_time && current_time < card.end_time {
            for ent in &card.emitters {
                let Ok(mut active) = emitters.get_mut(*ent) else {
                    continue;
                };
                **active = true;
            }
        }
    }
}

#[derive(Bundle, Default)]
pub struct EmitterBundle {
    emitter: Emitter,
    bullet_spawner: BulletSpawner,
    transform: Transform,
    active: Active,
}

fn make_emitter(time: f32, bullet_spawner: BulletSpawner) -> EmitterBundle {
    EmitterBundle {
        emitter: Emitter::new_secs(time),
        bullet_spawner,
        ..Default::default()
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct BulletSpawner {
    pub bullet: BulletBundle,
    pub normal: Option<NormalBullet>,
    pub rotation: Option<RotatingBullet>,
    pub stutter: Option<StutterBullet>,
    pub homing: Option<HomingBullet>,
    pub wave: Option<WaveBullet>,
    pub delayed: Option<Box<DelayedBullet>>,
}

impl BulletSpawner {
    pub fn add_components(self, commands: &mut EntityCommands<'_>) {
        if let Some(normal) = self.normal {
            commands.add_bullet(normal);
        }
        if let Some(rotation) = self.rotation {
            commands.add_bullet(rotation);
        }
        if let Some(stutter) = self.stutter {
            commands.add_bullet(stutter);
        }
        if let Some(homing) = self.homing {
            commands.add_bullet(homing);
        }
        if let Some(wave) = self.wave {
            commands.add_bullet(wave);
        }
        if let Some(delayed) = self.delayed {
            commands.add_bullet(*delayed.clone());
        }
    }

    pub fn new(bullet: BulletBundle) -> Self {
        Self {
            bullet,
            ..Default::default()
        }
    }

    pub fn normal(self, velocity: Vec2) -> Self {
        Self {
            normal: Some(NormalBullet { velocity }),
            ..self
        }
    }

    pub fn rotation(self, origin: Vec2, rotation_speed: f32) -> Self {
        Self {
            rotation: Some(RotatingBullet {
                origin,
                rotation_speed,
            }),
            ..self
        }
    }

    pub fn homing(self, seeking_time: f32, rotation_speed: f32, target: bullet::Target) -> Self {
        Self {
            homing: Some(HomingBullet {
                seeking_time,
                rotation_speed,
                target,
            }),
            ..self
        }
    }

    pub fn stutter(self, wait_time: f32, initial_velocity: Vec2, has_started: bool) -> Self {
        Self {
            stutter: Some(StutterBullet {
                wait_time,
                initial_velocity,
                has_started,
            }),
            ..self
        }
    }

    pub fn wave(self, sine_mod: f32, true_velocity: Vec2) -> Self {
        Self {
            wave: Some(WaveBullet {
                sine_mod,
                true_velocity,
            }),
            ..self
        }
    }

    pub fn delayed(self, delayed: DelayedBullet) -> Self {
        Self {
            delayed: Some(Box::new(delayed)),
            ..self
        }
    }
}

#[derive(Component, Clone)]
struct RandomMovement {
    next_move_timer: Timer,
    move_start: Vec2,
    move_end: Vec2,
    move_time: f32,
    time_since_last_move: Stopwatch,
}

impl RandomMovement {
    fn new(start: Vec2, speed: f32) -> Self {
        RandomMovement {
            next_move_timer: Timer::from_seconds(5.0, TimerMode::Repeating),
            move_start: start,
            move_end: start,
            move_time: speed,
            time_since_last_move: Stopwatch::new(),
        }
    }
}

fn do_random_movement(time: Res<Time>, mut query: Query<(&mut Transform, &mut RandomMovement)>) {
    for (mut trans, mut movement) in &mut query {
        let RandomMovement {
            next_move_timer,
            time_since_last_move,
            move_start,
            move_end,
            move_time,
        } = &mut *movement;

        next_move_timer.tick(time.delta());
        time_since_last_move.tick(time.delta());

        let move_float = time_since_last_move.elapsed_secs() / *move_time;

        fn ease(x: f32) -> f32 {
            -((x * PI).cos() - 1.0) / 2.0
        }

        if move_float <= 1.0 {
            let new_pos = move_start.lerp(*move_end, ease(move_float));
            trans.translation = new_pos.extend(trans.translation.z);
        }

        if next_move_timer.finished() {
            let mut rng = rand::rng();
            time_since_last_move.reset();
            next_move_timer.reset();
            next_move_timer.set_duration(Duration::from_secs_f32(
                *move_time + rng.random_range(0.5..10.0),
            ));
            *move_start = *move_end;

            let new_x = rng.random_range(200.0..900.0);
            let new_y = rng.random_range(-500.0..500.0);

            *move_end = Vec2::new(new_x, new_y)
        }
    }
}

fn divisive_emitter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &mut Emitter,
        &BulletSpawner,
        &mut DivisiveEmitter,
        &Active,
    )>,
    player: Single<&Transform, With<PlayerMarker>>,
) {
    let playerpos = player.into_inner();
    for (trans, mut emitter, spawner, circ, active) in &mut query {
        if !**active {
            continue;
        }

        emitter.timer.tick(time.delta());

        let mut bullet = spawner.bullet.clone();

        if emitter.timer.finished() {
            emitter.timer.reset();

            let gap = 1920.0 / (circ.columns as f32 + 1.0);
            for i in 0..circ.columns {
                let mut bullet = bullet.clone();
                bullet.transform.translation +=
                    Vec2::from((-960.0 + (gap * (i + 1) as f32), 600.0)).extend(0.0);

                let mut commands = commands.spawn(bullet);

                if let Some(normal) = spawner.normal {
                    let velocity = Vec2::from((0.0, -1.0)).rotate(normal.velocity);
                    commands.add_bullet(NormalBullet { velocity });
                }
            }
            let gap = 1080.0 / (circ.rows as f32 + 1.0);
            for i in 0..circ.rows {
                let mut bullet = bullet.clone();
                bullet.transform.translation +=
                    Vec2::from((1000.0, -540.0 + (gap * (i + 1) as f32))).extend(0.0);

                let mut commands = commands.spawn(bullet);

                if let Some(normal) = spawner.normal {
                    let velocity = Vec2::from((-1.0, 0.0)).rotate(normal.velocity);
                    commands.add_bullet(NormalBullet { velocity });
                }
            }
        }
    }
}

fn circular_rotating_emitter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &mut Emitter,
        &BulletSpawner,
        &mut CircularAimedEmitter,
        &Active,
    )>,
    player: Single<&Transform, With<PlayerMarker>>,
) {
    let playerpos = player.into_inner();
    for (trans, mut emitter, spawner, circ, active) in &mut query {
        if !**active {
            continue;
        }

        emitter.timer.tick(time.delta());

        let mut bullet = spawner.bullet.clone();

        bullet.transform.translation += trans.translation();

        if emitter.timer.finished() {
            emitter.timer.reset();

            let ang = TAU / circ.count as f32;
            for i in 0..circ.count {
                let mut bullet = bullet.clone();
                let dir = Vec2::from_angle(ang * i as f32);
                bullet.transform.translation += (dir * circ.offset).extend(0.0);
                let player_dir = playerpos.translation.xy() - bullet.transform.translation.xy();

                let mut commands = commands.spawn(bullet);

                if let Some(normal) = spawner.normal {
                    let velocity = dir.rotate(normal.velocity);
                    commands.add_bullet(NormalBullet { velocity });
                }
                if let Some(mut rotating) = spawner.rotation {
                    rotating.origin += trans.translation().xy();
                    commands.add_bullet(rotating);
                }
                if let Some(mut rotating) = spawner.stutter {
                    commands.add_bullet(rotating);
                }
                if let Some(mut rotating) = spawner.homing {
                    commands.add_bullet(rotating);
                }
            }
        }
    }
}

fn tentacle_emitter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &mut Emitter,
        &BulletSpawner,
        &mut TentacleEmitter,
        &Active,
    )>,
    player: Single<&Transform, With<PlayerMarker>>,
) {
    let playerpos = player.into_inner();
    for (trans, mut emitter, spawner, circ, active) in &mut query {
        if !**active {
            continue;
        }

        emitter.timer.tick(time.delta());

        let mut bullet = spawner.bullet.clone();

        bullet.transform.translation += trans.translation();

        if emitter.timer.finished() {
            emitter.timer.reset();

            let ang = TAU / circ.count as f32;
            for i in 0..circ.count {
                let mut bullet = bullet.clone();
                let dir = Vec2::from_angle(ang * i as f32);
                bullet.transform.translation += (dir * circ.offset).extend(0.0);
                let player_dir = playerpos.translation.xy() - bullet.transform.translation.xy();

                let mut commands = commands.spawn(bullet);

                if let Some(normal) = spawner.normal {
                    let velocity = dir.rotate(normal.velocity);
                    commands.add_bullet(NormalBullet { velocity });
                }
                let mut delayed_spawner = BulletSpawner::new(BulletBundle {
                    collider: Collider { radius: 5.0 },
                    sprite: Sprite {
                        image: spawner.bullet.sprite.image.clone(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .homing(2.0, TAU, Target::Player);
                let delayed: DelayedBullet = DelayedBullet {
                    bullet: delayed_spawner,
                    delay: 1.0,
                    deployed: false,
                };
                commands.add_bullet(delayed);
            }
        }
    }
}

fn spray_emitter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &mut Emitter,
        &BulletSpawner,
        &mut SprayEmitter,
        &Active,
    )>,
    player: Single<&Transform, With<PlayerMarker>>,
    mut gizmos: Gizmos,
) {
    let playerpos = player.translation.xy();
    for (trans, mut emitter, spawner, mut spray, active) in &mut query {
        if !**active {
            continue;
        }

        let mut rng = rand::rng();
        emitter.timer.tick(time.delta());

        let mut bullet = spawner.bullet.clone();

        bullet.transform.translation += trans.translation();

        if emitter.timer.elapsed_secs() < spray.firing_time {
            spray.count += (time.delta().as_secs_f32() / spray.firing_speed);
            for _ in 0..(spray.count as u64) {
                let mut bullet = bullet.clone();
                let ang = Vec2::from_angle(
                    rng.random_range((spray.spray_width / -2.0)..=(spray.spray_width / 2.0)),
                );

                let dir = (playerpos - trans.translation().xy()).normalize();

                let mut commands = commands.spawn(bullet);

                if let Some(normal) = spawner.normal {
                    let velocity = ang.rotate(normal.velocity);
                    let velocity = dir.rotate(velocity);
                    commands.add_bullet(NormalBullet { velocity });
                }
                if let Some(rotating) = spawner.rotation {
                    commands.add_bullet(rotating);
                }
                spray.count -= 1.0;
            }
        }

        if emitter.timer.finished() {
            emitter.timer.reset();
        }
    }
}

fn rotating_spray_emitter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &mut Emitter,
        &BulletSpawner,
        &mut RotatingSprayEmitter,
        &Active,
    )>,
    player: Single<&Transform, With<PlayerMarker>>,
    mut gizmos: Gizmos,
) {
    let playerpos = player.translation.xy();
    for (trans, mut emitter, spawner, mut spray, active) in &mut query {
        if !**active {
            continue;
        }

        let mut rng = rand::rng();
        emitter.timer.tick(time.delta());

        let mut bullet = spawner.bullet.clone();

        bullet.transform.translation += trans.translation();
        spray.rotation += spray.rotation_speed * time.delta_secs();

        if emitter.timer.elapsed_secs() < spray.firing_time {
            for i in 0..spray.spray_count {
                spray.count += (time.delta().as_secs_f32() / spray.firing_speed);
                for _ in 0..(spray.count as u64) {
                    let mut bullet = bullet.clone();
                    let ang = Vec2::from_angle(
                        rng.random_range((spray.spray_width / -2.0)..=(spray.spray_width / 2.0))
                            + spray.rotation
                            + (TAU / spray.spray_count as f32 * i as f32),
                    );

                    let mut commands = commands.spawn(bullet);

                    if let Some(normal) = spawner.normal {
                        let velocity = ang.rotate(normal.velocity);
                        commands.add_bullet(NormalBullet { velocity });
                    }
                    spray.count -= 1.0;
                }
            }
        }

        if emitter.timer.finished() {
            emitter.timer.reset();
        }
    }
}

fn flood_emitter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &mut Emitter,
        &BulletSpawner,
        &mut FloodEmitter,
        &Active,
    )>,
    player: Single<&Transform, With<PlayerMarker>>,
    mut gizmos: Gizmos,
) {
    let playerpos = player.translation.xy();
    for (trans, mut emitter, spawner, mut spray, active) in &mut query {
        if !**active {
            continue;
        }

        let mut rng = rand::rng();
        emitter.timer.tick(time.delta());

        let mut bullet = spawner.bullet.clone();

        if emitter.timer.finished() {
            emitter.timer.reset();

            let mut bullet = bullet.clone();
            let ang =
                Vec2::from_angle(rng.random_range((spray.spray / -2.0)..=(spray.spray / 2.0)));
            let placement = rng.random_range(-540.0..=540.0);

            bullet.transform.translation = Vec2::from((920.0, placement)).extend(0.0);

            let mut commands = commands.spawn(bullet);

            if let Some(normal) = spawner.normal {
                let velocity = ang.rotate(normal.velocity);
                commands.add_bullet(NormalBullet { velocity });
            }
        }
    }
}

const BULLET_SIZE: f32 = 15.0;

fn circular_wave_emitter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &mut Emitter,
        &BulletSpawner,
        &mut CircularWaveEmitter,
        &Active,
    )>,
    player: Single<&Transform, With<PlayerMarker>>,
) {
    let playerpos = player.into_inner();
    for (trans, mut emitter, spawner, mut circ, active) in &mut query {
        if !**active {
            continue;
        }

        emitter.timer.tick(time.delta());

        let mut bullet = spawner.bullet.clone();

        bullet.transform.translation += trans.translation();

        if emitter.timer.finished() {
            emitter.timer.reset();

            let ang = TAU / circ.count as f32;
            for i in 0..circ.count {
                let mut bullet = bullet.clone();
                let dir = Vec2::from_angle(ang * i as f32 + circ.rotation);
                bullet.transform.translation += (dir * circ.offset).extend(0.0);
                let player_dir = playerpos.translation.xy() - bullet.transform.translation.xy();

                let mut commands = commands.spawn(bullet);

                if let Some(normal) = spawner.normal {
                    let velocity = dir.rotate(normal.velocity);
                    commands.add_bullet(NormalBullet { velocity });
                }
                if let Some(mut rotating) = spawner.wave {
                    let velocity = dir.rotate(rotating.true_velocity);
                    commands.add_bullet(WaveBullet {
                        sine_mod: rotating.sine_mod,
                        true_velocity: velocity,
                    });
                }
            }

            circ.rotation += circ.rotation_speed;
        }
    }
}

fn circular_homing_emitter(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        &GlobalTransform,
        &mut Emitter,
        &BulletSpawner,
        &mut CircularHomingEmitter,
        &Active,
    )>,
    player: Single<&Transform, With<PlayerMarker>>,
) {
    let playerpos = player.into_inner();
    for (trans, mut emitter, spawner, mut circ, active) in &mut query {
        if !**active {
            continue;
        }

        emitter.timer.tick(time.delta());

        let mut bullet = spawner.bullet.clone();

        bullet.transform.translation += trans.translation();

        if emitter.timer.finished() {
            emitter.timer.reset();

            let ang = TAU / circ.count as f32;
            let mut bullet = bullet.clone();
            let dir = Vec2::from_angle(ang * circ.idx as f32);
            bullet.transform.translation += (dir * circ.offset).extend(0.0);
            let player_dir = playerpos.translation.xy() - bullet.transform.translation.xy();

            let mut commands = commands.spawn(bullet);

            if let Some(normal) = spawner.normal {
                let velocity = dir.rotate(normal.velocity);
                commands.add_bullet(NormalBullet { velocity });
            }
            if let Some(mut rotating) = spawner.homing {
                commands.add_bullet(rotating);
            }

            circ.idx += 1;
            if circ.idx == 4 {
                circ.idx = 0;
            }
        }
    }
}

pub trait EmitterExt {
    fn spawn_spellcard<F>(&mut self, start_time: f32, end_time: f32, f: F) -> &mut Self
    where
        F: FnOnce(&mut SpellcardBuilder<'_, '_>);
}

impl<'a> EmitterExt for EntityCommands<'a> {
    fn spawn_spellcard<F>(&mut self, start_time: f32, end_time: f32, f: F) -> &mut Self
    where
        F: FnOnce(&mut SpellcardBuilder<'_, '_>),
    {
        self.with_children(|child_builder| {
            let mut spellcard_builder = SpellcardBuilder {
                emitters: vec![],
                builder: child_builder,
            };
            f(&mut spellcard_builder);

            let SpellcardBuilder { emitters, builder } = spellcard_builder;
            builder.spawn(Spellcard {
                start_time,
                end_time,
                emitters,
            });
        });

        self
    }
}

pub struct SpellcardBuilder<'a, 'b> {
    emitters: Vec<Entity>,
    builder: &'b mut ChildBuilder<'a>,
}

impl<'a, 'b> SpellcardBuilder<'a, 'b> {
    pub fn emitter<'c>(&'c mut self, emitter: EmitterBundle) -> EntityCommands<'c> {
        let builder = self.builder.spawn(emitter);
        self.emitters.push(builder.id());
        builder
    }
}

pub fn spawn_enemy(mut commands: Commands, assets: Res<TouhouAssets>, params: Res<MissionParams>) {
    let big = Some(Vec2::splat(BULLET_SIZE * 3.0));
    let medium = Some(Vec2::splat(BULLET_SIZE * 2.0));
    let small = Some(Vec2::splat(BULLET_SIZE));

    let bullet =
        |tex: &Handle<Image>, layout: &Handle<TextureAtlasLayout>, size, anim| BulletBundle {
            collider: Collider { radius: 5.0 },
            sprite: Sprite {
                image: tex.clone(),
                custom_size: size,
                texture_atlas: Some(TextureAtlas {
                    layout: layout.clone(),
                    index: 0,
                }),
                ..Default::default()
            },
            animation: Animation::new(0.1, anim, 0),
            ..Default::default()
        };

    let red_girl_bullet = bullet(
        &assets.girl_bullet_sheet,
        &assets.girl_bullet_layout,
        big,
        2,
    );
    let red_girl_bullet_2 = bullet(
        &assets.girl_bullet2_sheet,
        &assets.girl_bullet2_layout,
        big,
        2,
    );
    let lizard_bullet = bullet(
        &assets.lizard_bullet_sheet,
        &assets.lizard_bullet_layout,
        medium,
        3,
    );
    let tentacle_bullet = BulletBundle {
        collider: Collider { radius: 5.0 },
        sprite: Sprite {
            image: assets.bullet1.clone(),
            custom_size: small,
            ..Default::default()
        },
        ..Default::default()
    };
    let moon_girl_bullet = bullet(
        &assets.moongirl_bullet_sheet,
        &assets.moongirl_layout,
        medium,
        5,
    );

    match Enemies::MoonGirl {
        Enemies::RedGirl => {
            commands
                .spawn(EnemyBundle {
                    sprite: Sprite {
                        image: assets.redgirl_sheet.clone(),
                        custom_size: Some(Vec2::splat(150.0)),
                        texture_atlas: Some(TextureAtlas {
                            layout: assets.redgirl_layout.clone(),
                            index: 0,
                        }),
                        ..Default::default()
                    },
                    animation: Animation::new(0.1, 3, 0),
                    transform: Transform::from_xyz(200.0, 0.0, 0.0),
                    collider: Collider { radius: 150.0 },
                    health: Health(2000),
                    ..Default::default()
                })
                .insert(RandomMovement {
                    next_move_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
                    move_start: Vec2::new(200.0, 0.0),
                    move_end: Vec2::new(500.0, 0.0),
                    move_time: 3.0,
                    time_since_last_move: Stopwatch::new(),
                })
                .spawn_spellcard(0.0, 25.0, |builder| {
                    builder
                        .emitter(make_emitter(
                            0.05,
                            BulletSpawner::new(red_girl_bullet.clone())
                                .normal(Vec2::new(4.0, 0.0))
                                .rotation(Vec2::ZERO, 0.0),
                        ))
                        .insert(CircularAimedEmitter {
                            offset: 00.0,
                            count: 8,
                        });
                    builder
                        .emitter(make_emitter(
                            0.25,
                            BulletSpawner::new(red_girl_bullet_2.clone())
                                .normal(Vec2::new(2.0, 0.0))
                                .homing(4.0, TAU / 8.0, Target::Player),
                        ))
                        .insert(CircularHomingEmitter {
                            offset: 150.0,
                            count: 4,
                            idx: 0,
                        });
                })
                .spawn_spellcard(25.0, 45.0, |builder| {
                    builder
                        .emitter(make_emitter(
                            1.5,
                            BulletSpawner::new(red_girl_bullet.clone())
                                .normal(Vec2::new(2.0, 0.0))
                                .rotation(Vec2::ZERO, TAU / 64.0),
                        ))
                        .insert(CircularAimedEmitter {
                            offset: 20.0,
                            count: 48,
                        });
                    builder
                        .emitter(make_emitter(
                            1.5,
                            BulletSpawner::new(red_girl_bullet.clone())
                                .normal(Vec2::new(2.0, 0.0))
                                .rotation(Vec2::ZERO, TAU / -64.0),
                        ))
                        .insert(CircularAimedEmitter {
                            offset: 20.0,
                            count: 48,
                        });
                })
                .spawn_spellcard(45.0, 70.0, |builder| {
                    builder
                        .emitter(make_emitter(
                            0.1,
                            BulletSpawner::new(red_girl_bullet.clone())
                                .normal(Vec2::new(2.0, 0.0))
                                .wave(1.0, Vec2::new(2.0, 0.0)),
                        ))
                        .insert(CircularWaveEmitter {
                            offset: 150.0,
                            count: 6,
                            rotation: 0.0,
                            rotation_speed: 0.1,
                        });
                });
        }
        Enemies::Tentacle => {
            commands
                .spawn(EnemyBundle {
                    sprite: Sprite {
                        image: assets.tentacle.clone(),
                        custom_size: Some(Vec2::splat(150.0)),
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(200.0, 0.0, 0.0),
                    collider: Collider { radius: 150.0 },
                    health: Health(1500),
                    ..Default::default()
                })
                .insert(RandomMovement {
                    next_move_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
                    move_start: Vec2::new(200.0, 0.0),
                    move_end: Vec2::new(500.0, 0.0),
                    move_time: 3.0,
                    time_since_last_move: Stopwatch::new(),
                })
                .spawn_spellcard(0.0, 25.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            5.0,
                            BulletSpawner::new(tentacle_bullet.clone()).normal(Vec2::new(4.0, 0.0)),
                        ))
                        .insert(SprayEmitter {
                            spray_width: TAU,
                            firing_time: 5.0,
                            firing_speed: 0.05,
                            count: 0.0,
                        });
                })
                .spawn_spellcard(25.0, 45.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            0.05,
                            BulletSpawner::new(tentacle_bullet.clone()).normal(Vec2::new(4.0, 0.0)),
                        ))
                        .insert(TentacleEmitter {
                            offset: 100.0,
                            count: 4,
                        });
                })
                .spawn_spellcard(45.0, 70.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            1.4,
                            BulletSpawner::new(tentacle_bullet.clone()).normal(Vec2::new(4.0, 0.0)),
                        ))
                        .insert(SprayEmitter {
                            spray_width: 0.4,
                            firing_time: 1.0,
                            firing_speed: 0.01,
                            count: 0.0,
                        });
                });
        }
        Enemies::Lizard => {
            commands
                .spawn(EnemyBundle {
                    sprite: Sprite {
                        image: assets.lizard_sheet.clone(),
                        custom_size: Some(Vec2::splat(150.0)),
                        texture_atlas: Some(TextureAtlas {
                            layout: assets.lizard_layout.clone(),
                            index: 0,
                        }),
                        ..Default::default()
                    },
                    animation: Animation::new(0.1, 3, 0),
                    transform: Transform::from_xyz(200.0, 0.0, 0.0),
                    collider: Collider { radius: 150.0 },
                    health: Health(2000),
                    ..Default::default()
                })
                .insert(RandomMovement {
                    next_move_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
                    move_start: Vec2::new(200.0, 0.0),
                    move_end: Vec2::new(500.0, 0.0),
                    move_time: 3.0,
                    time_since_last_move: Stopwatch::new(),
                })
                .spawn_spellcard(0.0, 25.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            1.0,
                            BulletSpawner::new(lizard_bullet.clone()).normal(Vec2::new(4.0, 0.0)),
                        ))
                        .insert(SprayEmitter {
                            spray_width: TAU / 4.0,
                            firing_time: 0.5,
                            firing_speed: 0.02,
                            count: 0.0,
                        });
                })
                .spawn_spellcard(25.0, 45.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            1.0,
                            BulletSpawner::new(lizard_bullet.clone()).normal(Vec2::new(4.0, 0.0)),
                        ))
                        .insert(RotatingSprayEmitter {
                            spray_width: TAU / 4.0,
                            firing_time: 1.0,
                            firing_speed: 0.01,
                            rotation_speed: TAU / 8.0,
                            rotation: 0.0,
                            count: 0.0,
                            spray_count: 2,
                        });
                })
                .spawn_spellcard(45.0, 70.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            5.0,
                            BulletSpawner::new(lizard_bullet.clone())
                                .normal(Vec2::new(4.0, 0.0))
                                .rotation(Vec2::from((200.0, 0.0)), TAU / 16.0),
                        ))
                        .insert(SprayEmitter {
                            spray_width: TAU,
                            firing_time: 5.0,
                            firing_speed: 0.01,
                            count: 0.0,
                        });
                });
        }
        Enemies::MoonGirl => {
            commands
                .spawn(EnemyBundle {
                    sprite: Sprite {
                        image: assets.kaguya_sheet.clone(),
                        custom_size: Some(Vec2::splat(100.0)),
                        texture_atlas: Some(TextureAtlas {
                            layout: assets.kaguya_layout.clone(),
                            index: 0,
                        }),
                        ..Default::default()
                    },
                    animation: Animation::new(0.1, 5, 0),
                    transform: Transform::from_xyz(200.0, 0.0, 0.0),
                    collider: Collider { radius: 150.0 },
                    health: Health(5000),
                    ..Default::default()
                })
                .insert(RandomMovement {
                    next_move_timer: Timer::from_seconds(10.0, TimerMode::Repeating),
                    move_start: Vec2::new(200.0, 0.0),
                    move_end: Vec2::new(500.0, 0.0),
                    move_time: 3.0,
                    time_since_last_move: Stopwatch::new(),
                })
                // em1
                .spawn_spellcard(0.0, 25.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            0.05,
                            BulletSpawner::new(moon_girl_bullet.clone())
                                .normal(Vec2::new(5.0, 0.0)),
                        ))
                        .insert(DivisiveEmitter {
                            columns: 19,
                            rows: 11,
                        });
                })
                // em2
                .spawn_spellcard(25.0, 45.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            0.01,
                            BulletSpawner::new(moon_girl_bullet.clone())
                                .normal(Vec2::new(-5.0, 0.0)),
                        ))
                        .insert(FloodEmitter { spray: TAU / 16.0 });
                })
                // em12
                .spawn_spellcard(0.0, 45.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            1.5,
                            BulletSpawner::new(moon_girl_bullet.clone())
                                .normal(Vec2::new(-2.0, 0.0))
                                .rotation(Vec2::ZERO, TAU / 64.0),
                        ))
                        .insert(CircularAimedEmitter {
                            offset: 1120.0,
                            count: 64,
                        });
                })
                // em3
                .spawn_spellcard(45.0, 70.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            1.0,
                            BulletSpawner::new(moon_girl_bullet.clone())
                                .normal(Vec2::new(4.0, 0.0)),
                        ))
                        .insert(RotatingSprayEmitter {
                            spray_width: TAU / 4.0,
                            firing_time: 1.0,
                            firing_speed: 0.01,
                            rotation_speed: TAU / 8.0,
                            rotation: 0.0,
                            count: 0.0,
                            spray_count: 2,
                        });
                })
                //em4
                .spawn_spellcard(70.0, 100.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            0.05,
                            BulletSpawner::new(moon_girl_bullet.clone())
                                .normal(Vec2::new(4.0, 0.0))
                                .rotation(Vec2::ZERO, TAU / 16.0),
                        ))
                        .insert(CircularAimedEmitter {
                            offset: 50.0,
                            count: 24,
                        });
                    parent
                        .emitter(make_emitter(
                            0.05,
                            BulletSpawner::new(moon_girl_bullet.clone())
                                .normal(Vec2::new(4.0, 0.0))
                                .rotation(Vec2::ZERO, TAU / -16.0),
                        ))
                        .insert(CircularAimedEmitter {
                            offset: 50.0,
                            count: 24,
                        });
                })
                // em34
                .spawn_spellcard(45.0, 100.0, |parent| {
                    parent
                        .emitter(make_emitter(
                            4.0,
                            BulletSpawner::new(moon_girl_bullet.clone())
                                .normal(Vec2::new(4.0, 0.0))
                                .stutter(1.0, Vec2::new(4.0, 0.0), false)
                                .homing(3.0, TAU / 3.0, Target::Player),
                        ))
                        .insert(CircularAimedEmitter {
                            offset: 150.0,
                            count: 32,
                        });
                });
        }
    }
}
