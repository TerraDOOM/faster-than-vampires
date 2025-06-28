//! Spawn the main level.

use std::time::Instant;

use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    screens::{
        gameplay::enemies::{gen_enemy, Ship, ShipType},
        Screen,
    },
};

use super::player::{gen_player, PlayerAssets};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
