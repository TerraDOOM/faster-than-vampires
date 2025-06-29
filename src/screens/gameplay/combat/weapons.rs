use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    screens::{
        gameplay::{
            animation::{AnimatedSprite, AnimationType},
            enemies::Enemy,
            player::Player,
            GameplayLogic,
        },
        Screen,
    },
};

use super::Damage;

pub fn plugin(app: &mut App) {
    app.register_type::<WeaponAssets>();
    app.load_resource::<WeaponAssets>();
    app.add_systems(Update, fire_cannon.in_set(GameplayLogic));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct WeaponAssets {
    #[dependency]
    pub laser_shot: Handle<Image>,
    pub laser_shot_layout: Handle<TextureAtlasLayout>,
    #[dependency]
    pub cannon: Handle<Image>,
}

impl WeaponAssets {
    fn get_laser_shot_sprite(&self) -> Sprite {
        Sprite {
            image: self.laser_shot.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: self.laser_shot_layout.clone(),
                index: 0,
            }),
            ..default()
        }
    }
}

impl FromWorld for WeaponAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_resource_mut::<AssetServer>().unwrap();
        WeaponAssets {
            cannon: assets.load("images/entities/Gun1.png"),
            laser_shot: assets.load("VFX/Flipbooks/TFlip_LaserBall.png"),
            laser_shot_layout: assets.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                5,
                3,
                None,
                None,
            )),
        }
    }
}

#[derive(Component)]
pub struct Cannon {
    timer: Timer,
}

#[derive(Component)]
pub struct CannonBullet;

pub fn spawn_cannons(cannon: &Handle<Image>, n: usize) -> Vec<impl Bundle> {
    let positions = match n {
        0 => vec![],
        1 => vec![Vec2::new(0.0, 16.0)],
        2 => vec![Vec2::new(-16.0, 0.0), Vec2::new(16.0, 0.0)],
        3 => vec![
            Vec2::new(-16.0, -10.0),
            Vec2::new(0.0, 10.0),
            Vec2::new(16.0, -10.0),
        ],
        4 => vec![
            Vec2::new(-16.0, 15.0),
            Vec2::new(16.0, 15.0),
            Vec2::new(-32.0, 0.0),
            Vec2::new(32.0, 0.0),
        ],
        _ => todo!(),
    };

    let mut cannons = Vec::new();

    for pos in positions {
        let pos = pos * 2.0;
        cannons.push((
            Sprite {
                image: cannon.clone(),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.0),
            Cannon {
                timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            },
        ))
    }

    cannons
}

#[derive(Component)]
pub struct Laser {
    firing: bool,
    level: usize,
    fire: Duration,
    cooldown: Duration,
    timer: Timer,
}

pub fn fire_cannon(
    mut commands: Commands,
    player: Single<(&LinearVelocity, &Transform, &Children), With<Player>>,
    mut cannons: Query<
        (&GlobalTransform, &mut Transform, &mut Cannon),
        (Without<Player>, Without<Enemy>),
    >,
    enemies: Query<&Transform, With<Enemy>>,
    assets: Res<WeaponAssets>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let (player_velocity, player_trans, children) = player.into_inner();
    let player_pos = player_trans.translation;

    let closest_enemy = enemies
        .iter()
        .map(|trans| trans.translation)
        .min_by_key(|x| ((x - player_pos).length() * 100.0) as i32);

    gizmos.circle_2d(
        Isometry2d::from_translation(closest_enemy.unwrap_or(Vec3::ZERO).xy()),
        50.0,
        Color::srgb(1.0, 0.0, 0.0),
    );

    for child in children {
        let Ok((global_transform, mut transform, mut cannon)) = cannons.get_mut(*child) else {
            continue;
        };

        if let Some(closest_enemy) = closest_enemy {
            let enemy_dir = (closest_enemy - global_transform.translation()).xy();

            let rotation = player_trans.rotation.inverse()
                * Quat::from_rotation_z(-enemy_dir.angle_to(Vec2::Y));
            transform.rotation = rotation;
        }

        cannon.timer.tick(time.delta());

        if !cannon.timer.just_finished() {
            continue;
        }

        let dir = global_transform.rotation() * Vec3::new(0.0, 1.0, 0.0);

        let pos = global_transform.translation();

        commands
            .spawn((
                assets.get_laser_shot_sprite(),
                StateScoped(Screen::Gameplay),
                Transform::from_translation(pos + dir * 10.0),
                AnimatedSprite::new(30, 15, AnimationType::Repeating),
                Collider::circle(10.0),
                CannonBullet,
                RigidBody::Kinematic,
                CollisionEventsEnabled,
                Sensor,
                LinearVelocity(dir.xy() * 400.0 + player_velocity.0),
            ))
            .observe(
                |trigger: Trigger<OnCollisionStart>,
                 mut commands: Commands,
                 player: Single<Entity, With<Player>>| {
                    if trigger.collider == player.into_inner() {
                        return;
                    }
                    commands.trigger_targets(Damage(50), trigger.collider);
                    commands.entity(trigger.target()).despawn();
                },
            );
    }
}

fn spawn_laser() {}

fn fire_laser(laser: Query<(&GlobalTransform, &Laser, &RayCaster)>) {
    for (transform, laser, raycaster) in laser {}
}
