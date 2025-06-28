//! Spawn the main level.

use std::time::Instant;

use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    screens::{
        gameplay::enemies::{gen_enemy, EntityAssets, Ship, ShipType},
        Screen,
    },
};

use super::{
    enemies::gen_goon,
    player::{gen_player, PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    background: Handle<Image>,
    #[dependency]
    planet1: Handle<Image>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            background: assets.load("images/level/deeper_deeper_galaxy.png"),
            planet1: assets.load("images/mascot.png"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    entity_assets: Res<EntityAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            gen_player(400.0, &player_assets, &mut texture_atlas_layouts),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
            gen_goon(
                Ship {
                    shiptype: ShipType::EmpireGoon,
                    position: (32.0, 32.0),
                    lifetime: Instant::now(),
                    weapons: Vec::new()
                },
                &entity_assets
            )
        ],
    ));
}
