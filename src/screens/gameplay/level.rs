//! Spawn the main level.

use std::time::Instant;

use rand::Rng;

use bevy::{color::palettes::css::GREEN, gizmos, prelude::*};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    screens::{
        gameplay::enemies::{gen_asteroid, gen_enemy, EntityAssets, Ship, ShipType},
        Screen,
    },
    PausableSystems,
};

use super::{
    enemies::gen_goon,
    player::{gen_player, PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();

    app.add_systems(
        Update,
        (world_update
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay))),
    );
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
            gen_goon(&entity_assets)
        ],
    ));
}

pub fn world_update(
    time: Res<Time>,
    mut commands: Commands,
    entity_assets: Res<EntityAssets>,
    mut gizmo: Gizmos,
) {
    let mut rng = rand::thread_rng();

    gizmo.rect_2d(Isometry2d::IDENTITY, Vec2::new(100.0, 100.0), GREEN);

    if 0 == rng.gen_range(0..10) {
        println!("Spawned enemy");

        commands.spawn((
            Name::new("Goon"),
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
            children![gen_asteroid(
                &entity_assets,
                Vec2::new(
                    rng.gen_range(-1000..1000) as f32,
                    rng.gen_range(-1000..1000) as f32
                ),
                Vec2::new(rng.gen_range(0..10) as f32, rng.gen_range(0..10) as f32)
            )],
        ));
    }
}
