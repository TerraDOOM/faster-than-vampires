use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, screens::Screen};

use super::{
    animation::AnimatedSprite,
    combat::{Damage, Health},
    player::Player,
    GameplayLogic,
};

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ShipType {
    Flagship,
    EmpireGoon,
    PirateShip,
    Asteroid,
    Rammer,
}

#[derive(Component, Debug, Clone)]
pub struct Ship {
    pub shiptype: ShipType,
    pub position: Vec2,
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<EntityAssets>();
    app.load_resource::<EntityAssets>();

    app.add_systems(
        FixedUpdate,
        (
            process_goon_ai,
            process_rammer_ai,
            process_flagship_ai,
            cont_damage_update,
            evil_cont_damage_update,
            (update_shooter_behavior, run_aiming_shooters, moving_shooter).chain(),
        )
            .in_set(GameplayLogic),
    );
}

#[derive(Component)]
pub struct Enemy;
pub fn gen_enemy(ship: Ship, assets: &EntityAssets, init_velocity: Vec2) -> impl Bundle {
    let pos = ship.position;

    gen_enemy_trans(
        ship,
        assets,
        init_velocity,
        Transform::from_xyz(pos.x, pos.y, 0.0),
    )
}

pub fn gen_enemy_trans(
    ship: Ship,
    assets: &EntityAssets,
    init_velocity: Vec2,
    transform: Transform,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs

    (
        Enemy,
        if ship.shiptype == ShipType::Flagship {
            (
                Sprite {
                    image: assets.flagship.clone(),
                    custom_size: Some(Vec2 { x: 384.0, y: 384.0 }),
                    ..default()
                },
                Collider::circle(128.0),
            )
        } else {
            (
                Sprite {
                    image: match ship.shiptype {
                        ShipType::EmpireGoon => assets.empire_goon.clone(),
                        ShipType::PirateShip => assets.pirate_ship.clone(),
                        ShipType::Asteroid => assets.asteroid.clone(),
                        ShipType::Rammer => assets.ramming_ship.clone(),
                        _ => assets.empire_goon.clone(),
                    },
                    custom_size: Some(Vec2 { x: 32.0, y: 32.0 }),
                    ..default()
                },
                Collider::circle(16.0),
            )
        },
        transform,
        RigidBody::Dynamic,
        LinearVelocity(init_velocity),
    )
}

#[derive(Component, Debug)]
pub struct GoonAI;
pub fn gen_goon(assets: &EntityAssets, position: Vec2) -> impl Bundle {
    println!("goon generated");
    let ship = Ship {
        shiptype: ShipType::EmpireGoon,
        position,
    };

    (gen_enemy(ship, assets, Vec2::new(0.0, 0.0)), GoonAI)
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub struct FlagshipAI;
pub fn gen_flagship(assets: &EntityAssets) -> impl Bundle {
    let position = Vec2::new(-1580.0, 0.0);

    let flagship = Ship {
        shiptype: ShipType::Flagship,
        position,
    };

    (
        gen_enemy_trans(
            flagship,
            assets,
            Vec2::ZERO,
            Transform::from_translation(position.extend(0.0))
                .with_rotation(Quat::from_rotation_z(-PI / 2.0)),
        ),
        FlagshipAI,
        ExternalImpulse::new(Vec2::ZERO),
        Mass(10.0),
        ExternalTorque::default().with_persistence(false),
        LinearDamping(0.8),
        AngularDamping(0.1),
        CollisionEventsEnabled,
    )
}
pub fn process_flagship_ai(
    flagships: Query<
        (
            &Transform,
            &mut ExternalImpulse,
            &mut LinearDamping,
            &AngularVelocity,
            &mut ExternalTorque,
            &mut AngularDamping,
        ),
        With<FlagshipAI>,
    >,
    player: Single<&Transform, With<Player>>,
) {
    for (flagship_pos, mut force, mut linear_damping, angvel, mut torque, mut angular_damping) in
        flagships
    {
        let enemy_forward = (flagship_pos.rotation * Vec3::Y).xy();
        linear_damping.0 = 0.2;
        linear_damping.0 = 20.0;
        angular_damping.0 = 0.1;

        let to_player = (player.translation.xy() - flagship_pos.translation.xy()).normalize();

        // Get the dot product between the enemy forward vector and the direction to the player.
        let forward_dot_player = enemy_forward.dot(to_player);
        //If 1, we are already facing them
        if (forward_dot_player - 1.0).abs() < 0.1 {
            if angvel.0 > 0.1 {
                angular_damping.0 = 10.0;
            } else {
                force.apply_impulse(enemy_forward * 400.0);
            }
            continue;
        }
        let enemy_right = (flagship_pos.rotation * Vec3::X).xy();

        let right_dot_player = enemy_right.dot(to_player);

        let rotation_sign = -f32::copysign(1.0, right_dot_player);

        torque.apply_torque(rotation_sign * 700.0);
    }
}

#[derive(Component, Debug)]
pub struct AsteroidAI;
pub fn gen_asteroid(assets: &EntityAssets, position: Vec2, init_velocity: Vec2) -> impl Bundle {
    let asteroid = Ship {
        shiptype: ShipType::Asteroid,
        position,
    };
    (
        gen_enemy(asteroid, assets, init_velocity),
        AsteroidAI,
        Health(250.0),
    )
}

pub fn gen_shooter(assets: &EntityAssets, position: Vec2) -> impl Bundle {
    let ship = Ship {
        shiptype: ShipType::EmpireGoon,
        position,
    };
    (
        gen_enemy(ship, assets, Vec2::ZERO),
        BasicShooter::Moving,
        ExternalTorque::new(0.0).with_persistence(false),
        LinearDamping(0.1),
        AngularDamping(0.1),
        Mass(1.0),
        Health(250.0),
    )
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum RammerAI {
    Charging,
    Aiming,
}

pub fn gen_rammer(
    assets: &EntityAssets,
    position: Vec2,
    init_velocity: Vec2,
    ship_look: ShipType,
    rotation: f32,
) -> impl Bundle {
    let rammer = Ship {
        shiptype: ship_look,
        position,
    };
    (
        gen_enemy_trans(
            rammer,
            assets,
            init_velocity,
            Transform::from_translation(position.extend(0.0))
                .with_rotation(Quat::from_rotation_z(rotation)),
        ),
        RammerAI::Aiming,
        ExternalImpulse::new(Vec2::ZERO),
        Mass(1.0),
        ExternalTorque::default().with_persistence(false),
        LinearDamping(0.8),
        AngularDamping(0.1),
        CollisionEventsEnabled,
        Health(50.0),
    )
}

pub fn process_rammer_ai(
    rammers: Query<(
        &Transform,
        &LinearVelocity,
        &mut ExternalImpulse,
        &mut LinearDamping,
        &AngularVelocity,
        &mut ExternalTorque,
        &mut AngularDamping,
        &mut RammerAI,
    )>,
    player: Single<&Transform, With<Player>>,
) {
    for (
        rammer_pos,
        linvel,
        mut force,
        mut linear_damping,
        angvel,
        mut torque,
        mut angular_damping,
        mut ai,
    ) in rammers
    {
        let enemy_forward = (rammer_pos.rotation * Vec3::Y).xy();
        linear_damping.0 = 0.2;

        if *ai == RammerAI::Aiming {
            linear_damping.0 = 20.0;
            angular_damping.0 = 0.1;

            let to_player = (player.translation.xy() - rammer_pos.translation.xy()).normalize();

            // Get the dot product between the enemy forward vector and the direction to the player.
            let forward_dot_player = enemy_forward.dot(to_player);
            //If 1, we are already facing them
            if (forward_dot_player - 1.0).abs() < 0.001 {
                if angvel.0 > 0.1 {
                    angular_damping.0 = 10.0;
                } else {
                    *ai = RammerAI::Charging;
                    force.apply_impulse(enemy_forward * 600.0);
                }
                continue;
            }
            let enemy_right = (rammer_pos.rotation * Vec3::X).xy();

            let right_dot_player = enemy_right.dot(to_player);

            let rotation_sign = -f32::copysign(1.0, right_dot_player);

            torque.apply_torque(rotation_sign * 300.0);
        } else if *ai == RammerAI::Charging {
            angular_damping.0 = 80.0;
            if linvel.0.length() < 50.0 {
                linear_damping.0 = 100.0;
            }
            if linvel.0.length() < 2.0 {
                *ai = RammerAI::Aiming;
            }
        }
    }
}

pub fn process_goon_ai(goons: Query<&mut Transform, With<GoonAI>>) {
    for mut goon_pos in goons {
        goon_pos.translation.x += 1.0;
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct EntityAssets {
    #[dependency]
    flagship: Handle<Image>,
    #[dependency]
    empire_goon: Handle<Image>,
    #[dependency]
    pirate_ship: Handle<Image>,
    #[dependency]
    ramming_ship: Handle<Image>,
    #[dependency]
    outpost: Handle<Image>,
    #[dependency]
    asteroid: Handle<Image>,
    #[dependency]
    explosion: Handle<Image>,
    explosion_layout: Handle<TextureAtlasLayout>,
}

impl EntityAssets {
    pub fn get_explosion(&self) -> impl Bundle {
        (
            Sprite {
                image: self.explosion.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: self.explosion_layout.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(32.0, 32.0) * 4.0),
                ..default()
            },
            AnimatedSprite::new(30, 64, super::animation::AnimationType::Once),
        )
    }
}

impl FromWorld for EntityAssets {
    fn from_world(world: &mut World) -> Self {
        use crate::util::make_nearest;
        let assets = world.resource::<AssetServer>();
        Self {
            flagship: assets.load_with_settings("images/entities/Flagship.png", make_nearest),
            empire_goon: assets.load_with_settings("images/entities/Enemy1.png", make_nearest),
            pirate_ship: assets.load_with_settings("images/entities/Enemy2.png", make_nearest),
            outpost: assets.load_with_settings("images/mascot.png", make_nearest),
            asteroid: assets.load_with_settings("images/entities/Astroid 1 .png", make_nearest),
            ramming_ship: assets.load_with_settings("images/entities/Enemy3.png", make_nearest),
            explosion: assets.load_with_settings(
                "VFX/Flipbooks/TFlip_ExplosionRegular_Lower.png",
                make_nearest,
            ),
            explosion_layout: assets.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                8,
                8,
                None,
                None,
            )),
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct ContinuousDamage {
    pub damage_per_frame: f64,
}

pub fn cont_damage_update(
    mut commands: Commands,
    damage_zones: Query<(&ContinuousDamage, Entity)>,
    enemies: Query<Entity, With<Enemy>>,
    collisions: Collisions,
) {
    for (damage, zone_entity) in damage_zones {
        let currently_colliding = collisions.collisions_with(zone_entity);
        for one_collision in currently_colliding {
            let collision_target = one_collision.body2.unwrap();
            if enemies.contains(collision_target) {
                commands.trigger_targets(Damage(damage.damage_per_frame), collision_target);
            }
        }
    }
}

#[derive(Component, Debug, Copy, Clone, PartialEq)]
pub struct EvilContinuousDamage {
    pub damage_per_frame: f64,
}

pub fn evil_cont_damage_update(
    mut commands: Commands,
    damage_zones: Query<(&EvilContinuousDamage, Entity)>,
    stuff_to_hit: Query<Entity, (Or<(With<Enemy>, With<Player>)>, Without<FlagshipAI>)>,
    collisions: Collisions,
) {
    for (damage, zone_entity) in damage_zones {
        let currently_colliding = collisions.collisions_with(zone_entity);
        for one_collision in currently_colliding {
            let collision_target = one_collision.body2.unwrap();
            if stuff_to_hit.contains(collision_target) {
                commands.trigger_targets(Damage(damage.damage_per_frame), collision_target);
            }
        }
    }
}

const BASIC_SHOOTER_FIRING_INTERVAL_MS: u64 = 2000;

#[derive(Component, Debug, Clone)]
enum BasicShooter {
    Aiming { fire_clock: Timer },
    Moving,
}

impl BasicShooter {
    fn new_aiming() -> Self {
        BasicShooter::Aiming {
            fire_clock: Timer::new(
                Duration::from_millis(BASIC_SHOOTER_FIRING_INTERVAL_MS),
                TimerMode::Repeating,
            ),
        }
    }
}

fn update_shooter_behavior(
    shooters: Query<(
        &Transform,
        &mut BasicShooter,
        &mut LinearDamping,
        &mut AngularDamping,
    )>,
    player: Single<&Transform, With<Player>>,
) {
    let player_pos = player.into_inner();

    for (pos, mut enemy, mut damping, mut ang_damping) in shooters {
        match check_shooter_dist(pos, player_pos, &*enemy) {
            Some(new @ BasicShooter::Aiming { .. }) => {
                **damping = 100.0;
                *enemy = new;
            }
            Some(new @ BasicShooter::Moving) => {
                **ang_damping = 2.0;
                **damping = 2.0;
                *enemy = new;
            }
            _ => {}
        }
    }
}

fn run_aiming_shooters(
    mut commands: Commands,
    time: Res<Time>,
    shooters: Query<(
        &Transform,
        &mut BasicShooter,
        &mut ExternalTorque,
        &mut AngularDamping,
    )>,
    player: Single<&Transform, With<Player>>,
) {
    let pos = |t: &Transform| t.translation.xy();
    let player = player.into_inner();
    for (enemy, mut shooter, mut torque, mut damping) in shooters {
        let BasicShooter::Aiming { fire_clock } = &mut *shooter else {
            continue;
        };
        fire_clock.tick(time.delta());
        if rotate_towards(enemy, player, &mut *torque, 300.0) {
            **damping = 100.0;

            if fire_clock.finished() {
                shooter_fire(&mut commands, enemy, pos(player));
                fire_clock.reset();
            }
        } else {
            **damping = 2.0;
        }
    }
}

fn shooter_fire(commands: &mut Commands, shooter: &Transform, player_pos: Vec2) {
    commands
        .spawn((
            Sprite::from_color(Color::srgba(1.0, 0.0, 0.0, 1.0), Vec2::splat(30.0)),
            StateScoped(Screen::Gameplay),
            shooter.clone(),
            Collider::circle(10.0),
            Sensor,
            RigidBody::Kinematic,
            CollisionEventsEnabled,
            LinearVelocity((player_pos - shooter.translation.xy()).normalize() * 20.0),
        ))
        .observe(shooter_bullet_hit);
}

fn shooter_bullet_hit(
    trigger: Trigger<OnCollisionStart>,
    mut commands: Commands,
    player: Single<Entity, With<Player>>,
) {
    let player = player.into_inner();
    if player == trigger.collider {
        commands.trigger_targets(Damage(20.0), player);
        commands.entity(trigger.target()).despawn();
    }
}

fn rotate_towards(
    a: &Transform,
    b: &Transform,
    torque: &mut ExternalTorque,
    rotation_speed: f32,
) -> bool {
    let pos = |t: &Transform| t.translation.xy();

    let a_forward = (a.rotation * Vec3::Y).xy();
    let to_b = (pos(b) - pos(a)).normalize();

    // Get the dot product between the a forward vector and the direction to the player.
    let forward_dot_b = a_forward.dot(to_b);
    if (forward_dot_b - 1.0).abs() < 0.001 {
        return true;
    }

    let a_right = (a.rotation * Vec3::X).xy();
    let right_dot_b = a_right.dot(to_b);
    let rotation_sign = -f32::copysign(1.0, right_dot_b);

    torque.apply_torque(rotation_sign * rotation_speed);
    false
}

fn moving_shooter(
    shooters: Query<(
        &Transform,
        &BasicShooter,
        &mut ExternalImpulse,
        &mut ExternalTorque,
    )>,
    player: Single<&Transform, With<Player>>,
) {
    let player = player.into_inner();
    for (enemy, shooter, mut impulse, mut torque) in shooters {
        let BasicShooter::Moving = shooter else {
            continue;
        };
        let forward = (enemy.rotation * Vec3::Y).xy();
        rotate_towards(enemy, player, &mut *torque, 300.0);
        impulse.apply_impulse(forward * 20.0);
    }
}

// update a basic shooter and return whether it changed state
fn check_shooter_dist(
    pos: &Transform,
    player: &Transform,
    shooter: &BasicShooter,
) -> Option<BasicShooter> {
    let dist = pos.translation.distance(player.translation);

    match shooter {
        BasicShooter::Moving if dist <= 700.0 => Some(BasicShooter::new_aiming()),
        BasicShooter::Aiming { .. } if dist >= 900.0 => Some(BasicShooter::Moving),
        _ => None,
    }
}
