use std::time::Instant;

use avian2d::prelude::*;
use bevy::{math::VectorSpace, prelude::*};

use crate::{asset_tracking::LoadResource, PausableSystems};

use super::player::Player;

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ShipType {
    Flagship,
    EmpireGoon,
    PirateShip,
    Outpoust,
    Asteroid,
    Rammer,
}

#[derive(Component, Debug, Clone)]
pub struct Ship {
    pub shiptype: ShipType,
    pub position: Vec2,
    pub lifetime: Instant,
    pub weapons: Vec<()>,
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<EntityAssets>();
    app.load_resource::<EntityAssets>();

    app.add_systems(Update, process_goon_ai.in_set(PausableSystems));
    app.add_systems(Update, process_rammer_ai.in_set(PausableSystems));
}

#[derive(Component)]
pub struct Enemy;
pub fn gen_enemy(ship: Ship, assets: &EntityAssets, init_velocity: Vec2) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs

    (
        Enemy,
        Sprite {
            image: match ship.shiptype {
                ShipType::EmpireGoon => assets.empire_goon.clone(),
                ShipType::PirateShip => assets.empire_goon.clone(),
                ShipType::Asteroid => assets.asteroid.clone(),
                ShipType::Rammer => assets.ramming_ship.clone(),
                _ => assets.empire_goon.clone(),
            },
            custom_size: Some(Vec2 { x: 64.0, y: 64.0 }),
            ..default()
        },
        Transform::from_xyz(ship.position.x, ship.position.y, 0.0),
        RigidBody::Dynamic,
        Collider::circle(32.0),
        LinearVelocity(init_velocity),
    )
}

#[derive(Component, Debug)]
pub struct GoonAI;
pub fn gen_goon(assets: &EntityAssets, position: Vec2) -> impl Bundle {
    println!("goon generated");
    let ship = Ship {
        shiptype: ShipType::EmpireGoon,
        position: position,
        lifetime: Instant::now(),
        weapons: Vec::new(),
    };

    (gen_enemy(ship, assets, Vec2::new(0.0, 0.0)), GoonAI)
}

#[derive(Component, Debug)]
pub struct FlagshipAI;
pub fn gen_flagship(assets: &EntityAssets) -> impl Bundle {
    let ship = Ship {
        shiptype: ShipType::Flagship,
        position: Vec2::new(-700.0, 0.0),
        lifetime: Instant::now(),
        weapons: Vec::new(),
    };

    (
        FlagshipAI,
        Sprite {
            image: assets.flagship.clone(),
            custom_size: Some(Vec2 { x: 512.0, y: 512.0 }),
            ..default()
        },
        Transform::from_xyz(ship.position.x, ship.position.y, 0.0),
    )
}

#[derive(Component, Debug)]
pub struct AsteroidAI;
pub fn gen_asteroid(assets: &EntityAssets, position: Vec2, init_velocity: Vec2) -> impl Bundle {
    let asteroid = Ship {
        shiptype: ShipType::Asteroid,
        position: position,
        lifetime: Instant::now(),
        weapons: Vec::new(),
    };
    (gen_enemy(asteroid, assets, init_velocity), AsteroidAI)
}

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum RammerAI {
    Charging,
    Aiming,
}

pub fn gen_rammer(assets: &EntityAssets, position: Vec2, init_velocity: Vec2) -> impl Bundle {
    let rammer = Ship {
        shiptype: ShipType::Rammer,
        position: position,
        lifetime: Instant::now(),
        weapons: Vec::new(),
    };
    (
        gen_enemy(rammer, assets, init_velocity),
        RammerAI::Aiming,
        ExternalImpulse::new(Vec2::ZERO),
        Mass(1.0),
        ExternalTorque::default().with_persistence(false),
        LinearDamping(0.8),
        AngularDamping(0.1),
        CollisionEventsEnabled,
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
    mut gizmos: Gizmos,
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

            gizmos.arrow_2d(
                rammer_pos.translation.xy(),
                rammer_pos.translation.xy() + enemy_forward * 100.0,
                Color::srgba(1.0, 0.0, 0.0, 1.0),
            );

            // Get the dot product between the enemy forward vector and the direction to the player.
            let forward_dot_player = enemy_forward.dot(to_player);
            //If 1, we are already facing them
            if (forward_dot_player - 1.0).abs() < 0.001 {
                if angvel.0 > 0.1 {
                    angular_damping.0 = 10.0;
                } else {
                    *ai = RammerAI::Charging;
                    force.apply_impulse(enemy_forward * 1200.0);
                }
                continue;
            }
            let enemy_right = (rammer_pos.rotation * Vec3::X).xy();

            let right_dot_player = enemy_right.dot(to_player);

            let rotation_sign = -f32::copysign(1.0, right_dot_player);

            torque.apply_torque(rotation_sign * 300.0);
        } else if *ai == RammerAI::Charging {
            angular_damping.0 = 80.0;
            gizmos.arrow_2d(
                rammer_pos.translation.xy(),
                rammer_pos.translation.xy() + enemy_forward * 100.0,
                Color::srgba(0.0, 1.0, 0.0, 1.0),
            );
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
    fn get_explosion(&self) -> impl Bundle {}
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
        }
    }
}
