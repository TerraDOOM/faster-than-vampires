//! Player-specific behavior.

use std::{collections::HashMap, f32::consts::PI};

use avian2d::prelude::*;
use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};

use crate::{asset_tracking::LoadResource, AppSystems, PausableSystems};

use super::{
    animation::{AnimatedSprite, AnimationType},
    combat::Health,
    movement::MovementController,
    upgrade_menu::{UpgradeTypes, Upgrades},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.

    app.add_systems(
        Update,
        record_player_directional_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

#[derive(Component)]
pub struct ThrusterUpgrade;

/// The player character.
pub fn gen_player(max_speed: f32, player_assets: &PlayerAssets) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs

    (
        Name::new("Player"),
        Player,
        Sprite {
            image: player_assets.ducky.clone(),
            ..default()
        },
        Transform::from_scale(Vec2::splat(2.0).extend(1.0)),
        MovementController {
            max_speed,
            ..default()
        },
        Upgrades {
            gotten_upgrades: HashMap::from([
                (UpgradeTypes::Cannon, 1),
                (UpgradeTypes::Health, 1),
                (UpgradeTypes::Thrusters, 1),
            ]),
        },
        player_physics_params(),
        Health(100),
    )
}

fn player_physics_params() -> impl Bundle {
    (
        RigidBody::Dynamic,
        Collider::capsule(0.75, 1.5),
        Mass(1.0),
        ExternalTorque::default().with_persistence(false),
        ExternalImpulse::default(),
        AngularDamping(5.0),
        LinearDamping(1.2),
        MaxLinearSpeed(1000.0),
        MaxAngularSpeed(PI),
        CollisionEventsEnabled,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = intent;
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    pub ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
    #[dependency]
    pub crash_sfx: Handle<AudioSource>,
    #[dependency]
    pub exploded: Handle<Image>,
    pub exploded_layout: Handle<TextureAtlasLayout>,
}

impl PlayerAssets {
    pub fn get_explosion(&self) -> impl Bundle {
        (
            Sprite {
                image: self.exploded.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: self.exploded_layout.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(256.0, 256.0)),
                ..default()
            },
            AnimatedSprite::new(30, 64, AnimationType::Once),
        )
    }
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                "images/entities/Flight.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            crash_sfx: assets.load("audio/sound_effects/metal_crash.ogg"),
            exploded: assets.load("VFX/Flipbooks/TFlip_EpicExplosion.png"),
            exploded_layout: assets.add(TextureAtlasLayout::from_grid(
                UVec2::splat(256),
                8,
                8,
                None,
                None,
            )),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
