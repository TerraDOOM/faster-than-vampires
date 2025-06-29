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

#[derive(Component, Debug)]
pub struct RammerAI;
pub fn gen_rammer(assets: &EntityAssets, position: Vec2, init_velocity: Vec2) -> impl Bundle {
    let rammer = Ship {
        shiptype: ShipType::Rammer,
        position: position,
        lifetime: Instant::now(),
        weapons: Vec::new(),
    };
    (
        gen_enemy(rammer, assets, init_velocity),
        RammerAI,
        ExternalImpulse::new(Vec2::ZERO),
        ExternalTorque::default().with_persistence(false),
    )
}
pub fn process_rammer_ai(
    rammers: Query<
        (&mut Transform, &mut ExternalImpulse, &mut ExternalTorque),
        (With<RammerAI>, Without<Player>),
    >,
    player: Single<&Transform, (With<Player>, Without<GoonAI>)>,
) {
    for (rammer_pos, mut linvel, mut angvel) in rammers {
        let enemy_forward = (rammer_pos.rotation * Vec3::Y).xy();
        let to_player = (player.translation.xy() - rammer_pos.translation.xy()).normalize();

        // Get the dot product between the enemy forward vector and the direction to the player.
        let forward_dot_player = enemy_forward.dot(to_player);
        //If 1, we are already facing them
        println!("LOL");
        if (forward_dot_player - 1.0).abs() < f32::EPSILON {
            println!("Thrusting")
        }
        println!("Rotating");
        let enemy_right = (rammer_pos.rotation * Vec3::X).xy();

        let right_dot_player = enemy_right.dot(to_player);

        let rotation_sign = -f32::copysign(1.0, right_dot_player);

        angvel.apply_torque(rotation_sign * 1000000.0);
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
