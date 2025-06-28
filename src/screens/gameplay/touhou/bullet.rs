use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
    time::Duration,
};

use bevy::{
    color::palettes::css::{BLUE, RED},
    ecs::query::QueryFilter,
    input::common_conditions::input_pressed,
    time::Stopwatch,
};
use enemy::{Animation, BulletSpawner, EnemyMarker, Health};

use super::*;

#[derive(QueryFilter)]
pub struct PlayerBullets {
    marker: With<BulletMarker>,
    cond: With<PlayerBullet>,
}
#[derive(QueryFilter)]
struct EnemyBullets {
    marker: With<BulletMarker>,
    cond: Without<PlayerBullet>,
}
type Bullets = With<BulletMarker>;

pub fn bullet_plugin(app: &mut App) {
    app.add_event::<BulletHit>()
        .add_event::<PlayerHit>()
        .add_event::<EnemyHit>()
        .add_systems(
            FixedUpdate,
            (
                (
                    move_normal_bullets,
                    move_rotating_bullets,
                    move_homing_bullets,
                    move_wave_bullets,
                    move_stutter_bullets,
                    resolve_delayed_bullets,
                )
                    .chain(),
                check_enemy_bullets,
                check_bullet_bullet,
                check_player_bullets,
                despawn_bullets,
                fire_weapons.run_if(input_pressed(KeyCode::KeyZ)),
                tick_bullets,
            )
                .run_if(in_state(GameState::Touhou)),
        )
        .add_systems(
            FixedPreUpdate,
            set_alt_fire.run_if(in_state(GameState::Touhou)),
        )
        .add_systems(
            FixedPostUpdate,
            (bullet_bullet_hit, process_player_hits, process_enemy_hits)
                .run_if(in_state(GameState::Touhou)),
        );
}

fn make_machinegun(assets: &TouhouAssets) -> Weapon {
    Weapon {
        timer: Timer::new(Duration::from_secs_f32(0.05), TimerMode::Repeating),
        ammo_cost: 1,
        bullet: BulletSpawner::new(BulletBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0)
                .with_rotation(Quat::from_rotation_z(PI / 2.0)),
            collider: Collider { radius: 6.0 },
            sprite: Sprite {
                image: assets.bullet1.clone(),
                ..Default::default()
            },
            ..Default::default()
        })
        .normal(Vec2::new(20.0, 0.0)),
        salted: true,
        damage: 2,
        phasing: false,
    }
}

fn make_rocketlauncher(assets: &TouhouAssets) -> Weapon {
    let bundle = BulletBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        collider: Collider { radius: 20.0 },
        sprite: Sprite {
            image: assets.rocket.clone(),
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..Default::default()
        },
        ..Default::default()
    };

    Weapon {
        timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating),
        ammo_cost: 100,
        bullet: BulletSpawner::new(bundle.clone())
            .normal(Vec2 { x: 10.0, y: 0.0 })
            .delayed(DelayedBullet {
                bullet: BulletSpawner::new(bundle)
                    .normal(Vec2::new(10.0, 0.0))
                    .homing(60.0, TAU / 2.0, Target::Enemy),
                delay: 0.3,
                deployed: false,
            }),
        salted: false,
        damage: 50,
        phasing: false,
    }
}

#[derive(Component)]
pub struct Weapon {
    timer: Timer,
    ammo_cost: u32,
    bullet: BulletSpawner,
    salted: bool,
    phasing: bool,
    damage: u32,
}

#[derive(Clone, Debug)]
pub(crate) enum BulletType {
    Normal(NormalBullet),
    Rotating(RotatingBullet),
    Homing(HomingBullet),
    Stutter(StutterBullet),
    Wave(WaveBullet),
    Delayed(DelayedBullet),
}

pub fn config_loadout(
    mission_params: Res<MissionParams>,
    mut commands: Commands,
    assets: Res<TouhouAssets>,
    player: Single<(Entity, &mut Speed, &mut Ammo, &mut Life, &mut Collider), With<PlayerMarker>>,
) {
    let loadout = &mission_params.loadout;
    let (ent, mut speed, mut ammo, mut life, mut collider) = player.into_inner();
    let assets = &*assets;

    let mut weapons = vec![];
    let mut alt_weapons = vec![];

    let mut salted = false;
    let mut alt_salted = false;

    let mut phasing = false;
    let mut alt_phasing = false;

    let mut ammo_multiplier = 1.0;
    let mut damage_multiplier = 1.0;

    for &(tech, alt) in loadout {
        let mut weapon_vec =
            |alt, weapon| if alt { &mut alt_weapons } else { &mut weapons }.push(weapon);

        match tech {
            Tech::MachineGun => weapon_vec(alt, make_machinegun(assets)),
            Tech::AmmoStockpile => **ammo += 1000,
            Tech::HeavyBody => {
                ammo_multiplier += 0.5;
                **life += 3;
                collider.radius += 5.0
            }
            Tech::Rocket => {
                weapon_vec(alt, make_rocketlauncher(assets));
            }
            Tech::MagicBullet => {
                if alt {
                    alt_salted = true;
                } else {
                    salted = true;
                }
            }
            Tech::EngineT1 => **speed *= 2.0,
            Tech::EngineT2 => {
                **speed *= 4.0;
                damage_multiplier += 0.5;
            }
            Tech::MachineGunT2 => {
                weapon_vec(alt, make_machinegun(assets));
                weapon_vec(alt, make_machinegun(assets));
            }
            Tech::Phase => {
                if alt {
                    alt_phasing = true;
                } else {
                    phasing = true;
                }
            }
            _ => {}
        }
    }

    **ammo = (**ammo as f32 * ammo_multiplier) as u32;

    commands.entity(ent).with_children(|player| {
        for mut weapon in weapons {
            weapon.salted = salted;
            weapon.phasing = phasing;
            weapon.damage = (weapon.damage as f32 * damage_multiplier) as u32;
            player.spawn(weapon);
        }
        for mut weapon in alt_weapons {
            weapon.salted = alt_salted;
            weapon.phasing = alt_phasing;
            weapon.damage = (weapon.damage as f32 * damage_multiplier) as u32;
            player.spawn(weapon).insert(AltFire);
        }
    });
}

impl Weapon {
    fn spawn_bullet(&mut self, commands: &mut Commands, player_pos: Vec2) {
        self.timer.reset();

        let bullet = BulletBundle {
            transform: Transform {
                translation: player_pos.extend(0.0) + self.bullet.bullet.transform.translation,
                ..self.bullet.bullet.transform
            },
            ..self.bullet.bullet.clone()
        };

        let mut ent = commands.spawn(bullet);

        ent.insert(PlayerBullet {
            damage: self.damage,
        })
        .insert_if(Salted, || self.salted)
        .insert_if(Phasing, || self.phasing);

        self.bullet.clone().add_components(&mut ent);
    }
}

fn set_alt_fire(
    mut commands: Commands,
    player: PlayerQ<Entity>,
    mut weapons: Query<&mut Weapon>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut player = commands.entity(*player);

    if input.just_pressed(KeyCode::ShiftLeft) || input.just_released(KeyCode::ShiftLeft) {
        for mut weapon in &mut weapons {
            weapon.timer.reset();
        }
    }
    if input.pressed(KeyCode::ShiftLeft) {
        player.insert(AltFire);
    } else {
        player.remove::<AltFire>();
    }
}

fn fire_weapons(
    time: Res<Time>,
    mut commands: Commands,
    mut weapons: Query<(&mut Weapon, Option<&AltFire>)>,
    player: PlayerQ<(&Transform, &mut Ammo, Option<&AltFire>)>,
) {
    let (trans, mut ammo, alt) = player.into_inner();
    let ammo: &mut u32 = &mut **ammo;

    let pos = trans.translation.xy();
    let alt_fire = alt.is_some();
    let (mut weapon_count, mut alt_weapon_count) = (0, 0);

    for (weapon, is_alt) in &mut weapons {
        if is_alt.is_some() != alt_fire {
            alt_weapon_count += 1;
        } else {
            weapon_count += 1;
        }
    }

    let mut weapon_idx = 0;

    for (mut weapon, is_alt) in &mut weapons {
        // we are in the wrong weapon group
        if is_alt.is_some() != alt_fire {
            continue;
        } else {
            weapon_idx += 1;
        }

        weapon.timer.tick(time.delta());

        if weapon.timer.just_finished() {
            weapon.timer.reset();

            if *ammo < weapon.ammo_cost {
                continue;
            }

            *ammo = ammo.saturating_sub(weapon.ammo_cost);

            weapon.spawn_bullet(
                &mut commands,
                pos - Vec2 {
                    x: 0.0,
                    y: (((weapon_idx - 1) * 50) - (25 * (weapon_count - 1))) as f32,
                },
            )
        }
    }
}

#[derive(Component, Default, Clone, Debug)]
pub struct BulletMarker;

#[derive(Bundle, Default, Clone, Debug)]
pub struct BulletBundle {
    pub transform: Transform,
    pub collider: Collider,
    pub sprite: Sprite,
    pub velocity: Velocity,
    pub lifetime: Lifetime,
    pub animation: Animation,
    pub markers: (BulletMarker, TouhouMarker),
}

#[derive(Component, Default, Clone, Debug)]
pub struct Lifetime(Stopwatch);

#[derive(Component, Deref, DerefMut, Default)]
pub struct PlayerBullet {
    damage: u32,
}

#[derive(Component, Clone, Copy, Default, Debug)]
pub struct NormalBullet {
    pub velocity: Vec2,
}

#[derive(Component, Clone, Copy, Default, Debug)]
pub struct HomingBullet {
    pub rotation_speed: f32,
    pub seeking_time: f32,
    pub target: Target,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum Target {
    #[default]
    Enemy,
    Player,
}

#[derive(Component, Clone, Copy, Default, Debug)]
pub struct StutterBullet {
    pub wait_time: f32,
    pub initial_velocity: Vec2,
    pub has_started: bool,
}

#[derive(Component, Clone, Copy, Default, Debug)]
pub struct WaveBullet {
    pub true_velocity: Vec2,
    pub sine_mod: f32,
}

#[derive(Component, Clone, Default, Debug)]
pub struct DelayedBullet {
    pub bullet: BulletSpawner,
    pub delay: f32,
    pub deployed: bool,
}

#[derive(Debug, Copy, Clone, Component)]
pub struct RotatingBullet {
    pub origin: Vec2,
    // rotation speed in radians/s
    pub rotation_speed: f32,
}

fn circle(t: &Transform, c: &Collider) -> Circle {
    super::Circle::new(c.radius, t.translation.xy())
}

#[derive(Component, Default)]
pub struct Salted;

#[derive(Component, Default)]
pub struct Phasing;

#[derive(Event)]
pub struct PlayerHit(Entity);

#[derive(Debug, Clone, Component, Default)]
pub struct Velocity {
    velocity: Vec2,
}

#[derive(Component)]
pub struct AltFire;

#[derive(Event)]
pub struct BulletHit {
    player: Entity,
    enemy: Entity,
}

#[derive(Event, Copy, Clone)]
struct EnemyHit {
    bullet: Entity,
    enemy: Entity,
}

pub fn process_enemy_hits(
    mut commands: Commands,
    mut hits: EventReader<EnemyHit>,
    player_bullets: Query<(&PlayerBullet, Option<&Phasing>)>,
    mut enemies: Query<&mut Health, With<EnemyMarker>>,
) {
    for &EnemyHit { enemy, bullet } in hits.read() {
        let Ok((damage, phasing)) = player_bullets.get(bullet) else {
            continue;
        };

        let Ok(mut enemy_health) = enemies.get_mut(enemy) else {
            continue;
        };

        **enemy_health =
            enemy_health.saturating_sub(**damage * if phasing.is_some() { 1 } else { 2 });

        commands.entity(bullet).try_despawn();
    }
}

pub fn process_player_hits(
    mut commands: Commands,
    mut hits: EventReader<PlayerHit>,
    mut bullet_hits: EventReader<BulletHit>,
    salted_bullets: Query<Entity, (PlayerBullets, With<Salted>)>,
    player: Option<PlayerQ<(&mut Life, Option<&Invulnerability>)>>,
) {
    let Some((mut life, immortal)) = player.map(|p| p.into_inner()) else {
        return;
    };

    let already_collided_bullets: HashMap<_, _> = bullet_hits
        .read()
        .map(|hit| (hit.enemy, hit.player))
        .collect();

    let mut life_lost = false;
    for PlayerHit(ent) in hits.read() {
        // if something else already hit this bullet
        if let Some(&player_bullet) = already_collided_bullets.get(&ent) {
            // ... and if that something else was salted...
            if salted_bullets.contains(player_bullet) {
                continue;
            }
        }

        // try to get the bullet entity, but if it has already despawned, continue
        let Some(mut bullet) = commands.get_entity(*ent) else {
            continue;
        };

        bullet.try_despawn();

        if life_lost || immortal.is_some() {
            continue;
        }

        life.0 = life.0.saturating_sub(1);
        life_lost = true;
    }
}

fn bullet_bullet_hit(
    mut commands: Commands,
    mut hits: EventReader<BulletHit>,
    player_bullets: Query<(&NormalBullet, Option<&Salted>), PlayerBullets>,
    enemy_bullets: Query<&NormalBullet, EnemyBullets>,
) {
    for BulletHit { player, enemy } in hits.read() {
        let Ok((p, salted)) = player_bullets.get(*player) else {
            continue;
        };
        let Ok(e) = enemy_bullets.get(*enemy) else {
            continue;
        };

        commands.entity(*player).despawn();
        if salted.is_some() {
            commands.entity(*enemy).despawn();
        }
    }
}

fn check_player_bullets(
    mut hits: EventWriter<EnemyHit>,
    player_bullets: Query<(Entity, &Transform, &Collider), PlayerBullets>,
    enemies: Query<(Entity, &Transform, &Collider), With<enemy::EnemyMarker>>,
) {
    for (bullet, b_trans, b_coll) in &player_bullets {
        let b_circle = b_coll.to_circle(b_trans.translation.xy());
        for (enemy, e_trans, e_coll) in &enemies {
            let e_circle = e_coll.to_circle(e_trans.translation.xy());

            if b_circle.hits(e_circle) {
                hits.send(EnemyHit { enemy, bullet });
            }
        }
    }
}

fn check_bullet_bullet(
    mut hits: EventWriter<BulletHit>,
    player_bullets: Query<(Entity, &Transform, &Collider), (PlayerBullets, Without<Phasing>)>,
    enemy_bullets: Query<(Entity, &Transform, &Collider), EnemyBullets>,
) {
    for (p, p_trans, p_coll) in &player_bullets {
        let player_circle = circle(p_trans, p_coll);

        for (e, e_trans, e_coll) in &enemy_bullets {
            let enemy_circle = circle(e_trans, e_coll);

            if player_circle.hits(enemy_circle) {
                hits.send(BulletHit {
                    player: p,
                    enemy: e,
                });
            }
        }
    }
}

fn check_enemy_bullets(
    player: PlayerQ<(&Transform, &Collider)>,
    bullet_query: Query<(Entity, &Transform, &Collider), EnemyBullets>,
    mut hit_writer: EventWriter<PlayerHit>,
) {
    let player_circle = {
        let (trans, coll) = player.into_inner();
        circle(trans, coll)
    };

    for (ent, trans, coll) in &bullet_query {
        let bullet_circle = circle(trans, coll);

        if bullet_circle.hits(player_circle) {
            hit_writer.send(PlayerHit(ent));
        }
    }
}

fn despawn_bullets(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), EnemyBullets>,
) {
    for (entity, transform) in &bullet_query {
        if !Rect::new(-1500.0, -1500.0, 1500.0, 1500.0).contains(transform.translation.xy()) {
            commands.entity(entity).despawn()
        }
    }
}

fn move_normal_bullets(mut bullet_query: Query<(&NormalBullet, &mut Transform)>) {
    for (bullet, mut transform) in &mut bullet_query {
        transform.translation += bullet.velocity.extend(0.0);
    }
}

fn move_rotating_bullets(
    time: Res<Time>,
    mut bullet_query: Query<(&RotatingBullet, Option<&mut NormalBullet>, &mut Transform)>,
    mut gizmos: Gizmos,
) {
    for (bullet, normal, mut trans) in &mut bullet_query {
        let prev_pos = trans.translation.xy();

        let pos_mod = prev_pos - bullet.origin;

        gizmos.cross_2d(Isometry2d::from_translation(bullet.origin), 20.0, RED);

        let angle = bullet.rotation_speed * time.delta_secs();

        let rotated = Vec2::from_angle(angle).rotate(pos_mod);

        let new_pos = rotated + bullet.origin;

        if let Some(mut normal) = normal {
            normal.velocity = Vec2::from_angle(angle).rotate(normal.velocity);
        }

        trans.translation = new_pos.extend(0.0);
    }
}

fn move_homing_bullets(
    time: Res<Time>,
    mut bullet_query: Query<
        (
            &mut Sprite,
            &HomingBullet,
            &mut NormalBullet,
            &mut Velocity,
            &Lifetime,
            &mut Transform,
        ),
        With<BulletMarker>,
    >,
    player: Option<Single<&Transform, (PlayerFilter, Without<BulletMarker>)>>,
    enemy: Option<Single<&Transform, (With<EnemyMarker>, Without<BulletMarker>)>>,
    mut gizmos: Gizmos,
) {
    let enemy = enemy.map(|e| e.translation);
    let player = player.map(|p| p.translation);

    for (mut sprite, bullet, mut normal, mut velocity, lifetime, mut trans) in &mut bullet_query {
        if bullet.target == Target::Enemy && enemy.is_none() {
            log::error!("can't find enemy");
        }

        let target_pos = match (bullet.target, player, enemy) {
            (Target::Player, Some(player), _) => player,
            (Target::Enemy, _, Some(enemy)) => enemy,
            _ => continue,
        }
        .xy();

        if lifetime.0.elapsed_secs() <= bullet.seeking_time
            && normal.velocity.length().abs() >= 0.01
        {
            let angle =
                (target_pos - trans.translation.xy()).normalize() * normal.velocity.length();

            let rotation = bullet.rotation_speed * time.delta_secs();

            gizmos.arrow_2d(trans.translation.xy(), trans.translation.xy() + angle, BLUE);

            normal.velocity = normal.velocity.rotate_towards(angle, rotation);
            trans.rotation = Quat::from_rotation_z(normal.velocity.to_angle());
        }
    }
}

fn move_stutter_bullets(
    mut bullet_query: Query<(
        &mut StutterBullet,
        &mut NormalBullet,
        &Lifetime,
        &mut Transform,
    )>,
) {
    for (mut bullet, mut velocity, lifetime, mut trans) in &mut bullet_query {
        if lifetime.0.elapsed_secs() < bullet.wait_time {
            velocity.velocity = Vec2::ZERO;
        } else if !bullet.has_started {
            velocity.velocity = bullet.initial_velocity;
            bullet.has_started = true;
        }
    }
}

fn resolve_delayed_bullets(
    mut commands: Commands,
    mut bullet_query: Query<(
        Entity,
        &mut DelayedBullet,
        &mut NormalBullet,
        &Lifetime,
        &mut Transform,
    )>,
) {
    for (entity, mut bullet, mut velocity, lifetime, mut trans) in &mut bullet_query {
        if lifetime.0.elapsed_secs() >= bullet.delay && !bullet.deployed {
            bullet.deployed = true;
            bullet
                .bullet
                .clone()
                .add_components(&mut commands.entity(entity));
        }
    }
}

fn move_wave_bullets(
    mut bullet_query: Query<(
        &WaveBullet,
        &mut Velocity,
        &Lifetime,
        &mut Transform,
        &mut NormalBullet,
    )>,
) {
    for (bullet, mut velocity, lifetime, mut trans, mut normal) in &mut bullet_query {
        normal.velocity =
            bullet.true_velocity * ((lifetime.0.elapsed_secs() * bullet.sine_mod).sin() + 1.0);
    }
}

fn tick_bullets(time: Res<Time>, mut bullets: Query<&mut Lifetime, Bullets>) {
    for mut watch in &mut bullets {
        watch.0.tick(time.delta());
    }
}

pub trait BulletCommandExt {
    fn add_bullet<T: AsBulletKind>(&mut self, kind: T) -> &mut Self;
}

pub trait AsBulletKind {
    fn as_bullet_type(self) -> BulletType;
}

impl AsBulletKind for RotatingBullet {
    fn as_bullet_type(self) -> BulletType {
        BulletType::Rotating(self)
    }
}

impl AsBulletKind for NormalBullet {
    fn as_bullet_type(self) -> BulletType {
        BulletType::Normal(self)
    }
}

impl AsBulletKind for HomingBullet {
    fn as_bullet_type(self) -> BulletType {
        BulletType::Homing(self)
    }
}

impl AsBulletKind for StutterBullet {
    fn as_bullet_type(self) -> BulletType {
        BulletType::Stutter(self)
    }
}

impl AsBulletKind for WaveBullet {
    fn as_bullet_type(self) -> BulletType {
        BulletType::Wave(self)
    }
}

impl AsBulletKind for DelayedBullet {
    fn as_bullet_type(self) -> BulletType {
        BulletType::Delayed(self)
    }
}

impl AsBulletKind for BulletType {
    fn as_bullet_type(self) -> BulletType {
        self
    }
}

impl BulletCommandExt for EntityCommands<'_> {
    fn add_bullet<T: AsBulletKind>(&mut self, kind: T) -> &mut Self {
        let kind = kind.as_bullet_type();

        match kind {
            BulletType::Normal(normal) => self.insert(normal),
            BulletType::Rotating(rotating) => self.insert(rotating),
            BulletType::Homing(homing) => self.insert(homing),
            BulletType::Stutter(stutter) => self.insert(stutter),
            BulletType::Wave(wave) => self.insert(wave),
            BulletType::Delayed(delayed) => self.insert(delayed),
        }
    }
}
