use std::time::Instant;

use bevy::prelude::*;

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

#[derive(Component)]
pub struct Enemy;

pub fn gen_enemy(ship: Ship) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    (Name::new("Enemy"), Enemy)
}
