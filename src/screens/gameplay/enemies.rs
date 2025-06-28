use std::time::Instant;

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, PausableSystems};

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ShipType {
    Flagship,
    EmpireGoon,
    PirateShip,
    Outpoust,
}

#[derive(Component)]
pub struct Ship {
    pub shiptype: ShipType,
    pub position: (f32, f32),
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

pub fn gen_enemy(ship: Ship, assets: &EntityAssets) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs

    (
        Name::new("Enemy"),
        Enemy,
        Sprite {
            image: match ship.shiptype {
                ShipType::EmpireGoon => assets.empire_goon.clone(),
                _ => assets.empire_goon.clone(),
            },
            custom_size: Some(Vec2 { x: 50.0, y: 50.0 }),
            ..default()
        },
    )
}

#[derive(Component)]
struct EntityGoon;

pub fn gen_goon(ship: Ship, assets: &EntityAssets) -> impl Bundle {
    (gen_enemy(ship, assets), EntityGoon)
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
}

impl FromWorld for EntityAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            flagship: assets.load("images/mascot.png"),
            empire_goon: assets.load("images/mascot.png"),
            pirate_ship: assets.load("images/mascot.png"),
            outpost: assets.load("images/mascot.png"),
        }
    }
}
