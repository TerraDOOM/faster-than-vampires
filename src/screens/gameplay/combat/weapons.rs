use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    screens::gameplay::{
        animation::{AnimatedSprite, AnimationType},
        player::Player,
        GameplayLogic,
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
pub struct Laser {}

#[derive(Component)]
pub struct Cannon {
    timer: Timer,
}

#[derive(Component)]
pub struct CannonBullet;

pub fn spawn_cannons(n: usize) -> Vec<impl Bundle> {
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
        let pos = pos * 3.0;
        cannons.push((
            Sprite {
                color: Color::srgba(0.4, 0.4, 0.4, 1.0),
                custom_size: Some(Vec2::new(12.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.0),
            Cannon {
                timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            },
        ))
    }

    cannons
}

pub fn fire_cannon(
    mut commands: Commands,
    player: Single<(&LinearVelocity, &Children), With<Player>>,
    mut cannons: Query<(&GlobalTransform, &mut Cannon)>,
    assets: Res<WeaponAssets>,
    time: Res<Time>,
) {
    let (player_velocity, children) = player.into_inner();
    for child in children {
        let Ok((cannon_transform, mut cannon)) = cannons.get_mut(*child) else {
            continue;
        };

        cannon.timer.tick(time.delta());
        if !cannon.timer.just_finished() {
            continue;
        }

        let dir = cannon_transform.rotation() * Vec3::new(0.0, 1.0, 0.0);

        let pos = cannon_transform.translation();

        commands
            .spawn((
                assets.get_laser_shot_sprite(),
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
                |trigger: Trigger<OnCollisionStart>, mut commands: Commands| {
                    commands.trigger_targets(Damage(50), trigger.collider);
                    commands.entity(trigger.target()).despawn();
                },
            );
    }
}
