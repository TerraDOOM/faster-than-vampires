use std::time::Instant;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, PausableSystems};

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ShipType {
    Flagship,
    EmpireGoon,
    PirateShip,
    Outpoust,
    Asteroid,
}

#[derive(Component)]
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
                _ => assets.empire_goon.clone(),
            },
            custom_size: Some(Vec2 { x: 32.0, y: 32.0 }),
            ..default()
        },
        Transform::from_xyz(ship.position.x, ship.position.y, 0.0),
        RigidBody::Dynamic,
        Collider::circle(32.0),
        LinearVelocity(init_velocity),
    )
}

#[derive(Component)]
pub struct EntityGoon;
pub fn gen_goon(assets: &EntityAssets) -> impl Bundle {
    let ship = Ship {
        shiptype: ShipType::EmpireGoon,
        position: Vec2::new(0.0, 0.0),
        lifetime: Instant::now(),
        weapons: Vec::new(),
    };

    (gen_enemy(ship, assets, Vec2::new(0.0, 0.0)), EntityGoon);
}

#[derive(Component)]
pub struct EntityAsteroid {
    health: u32,
}

pub fn gen_asteroid(assets: &EntityAssets, position: Vec2, init_velocity: Vec2) -> impl Bundle {
    let asteroid = Ship {
        shiptype: ShipType::Asteroid,
        position: position,
        lifetime: Instant::now(),
        weapons: Vec::new(),
    };
    (
        gen_enemy(asteroid, assets, init_velocity),
        EntityAsteroid { health: 3 },
    )
}

pub fn process_goon_ai(goons: Query<&mut Transform, With<EntityGoon>>) {
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
    outpost: Handle<Image>,
    #[dependency]
    asteroid: Handle<Image>,
}

impl FromWorld for EntityAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            flagship: assets.load("images/mascot.png"),
            empire_goon: assets.load("images/mascot.png"),
            pirate_ship: assets.load("images/mascot.png"),
            outpost: assets.load("images/mascot.png"),
            asteroid: assets.load("images/mascot.png"),
        }
    }
}
