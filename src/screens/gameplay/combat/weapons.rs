use std::{f32::consts::PI, time::Duration};

use avian2d::prelude::*;
use bevy::{
    image::{ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
    sprite::Anchor,
};

use crate::{
    asset_tracking::LoadResource,
    screens::{
        gameplay::{
            animation::{AnimatedSprite, AnimationType},
            enemies::{ContinuosDamage, Enemy},
            player::Player,
            GameplayLogic,
        },
        Screen,
    },
    util::make_nearest,
};

use super::Damage;

pub fn plugin(app: &mut App) {
    app.register_type::<WeaponAssets>();
    app.load_resource::<WeaponAssets>();
    app.add_systems(Update, fire_cannon.in_set(GameplayLogic));
    app.add_systems(
        Update,
        (fire_laser, animate_laser).chain().in_set(GameplayLogic),
    );
    app.add_plugins((PhysicsDebugPlugin::default(),));
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct WeaponAssets {
    #[dependency]
    pub laser_shot: Handle<Image>,
    pub laser_shot_layout: Handle<TextureAtlasLayout>,
    #[dependency]
    pub cannon: Handle<Image>,

    #[dependency]
    pub e_field: Handle<Image>,
    pub e_field_layout: Handle<TextureAtlasLayout>,

    #[dependency]
    pub e_field_big: Handle<Image>,
    pub e_field_big_layout: Handle<TextureAtlasLayout>,

    #[dependency]
    pub muzzle_flash: Handle<Image>,
    pub muzzle_flash_layout: Handle<TextureAtlasLayout>,

    #[dependency]
    pub laser_beam: Handle<Image>,
    #[dependency]
    pub laser_hit: Handle<Image>,
    pub laser_hit_layout: Handle<TextureAtlasLayout>,
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

    fn get_laser_hit(&self) -> impl Bundle {
        (
            Sprite {
                image: self.laser_hit.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: self.laser_hit_layout.clone(),
                    index: 0,
                }),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            AnimatedSprite::new(30, 16, AnimationType::Repeating),
        )
    }
}

impl FromWorld for WeaponAssets {
    fn from_world(world: &mut World) -> Self {
        use crate::util::make_nearest;
        let assets = world.get_resource_mut::<AssetServer>().unwrap();
        WeaponAssets {
            cannon: assets.load_with_settings("images/entities/Gun1.png", make_nearest),
            laser_shot: assets.load("VFX/Flipbooks/TFlip_LaserBall.png"),
            laser_shot_layout: assets.add(TextureAtlasLayout::from_grid(
                UVec2::splat(32),
                5,
                3,
                None,
                None,
            )),
            muzzle_flash: assets.load_with_settings("VFX/Flipbooks/TFlip_Blast.png", make_nearest),
            muzzle_flash_layout: assets.add(TextureAtlasLayout::from_grid(
                UVec2::splat(86),
                3, //Width
                3,
                None,
                None,
            )),
            e_field: assets
                .load_with_settings("VFX/Flipbooks/TFlip_ElectricShield.png", make_nearest),
            e_field_layout: assets.add(TextureAtlasLayout::from_grid(
                UVec2::splat(64),
                4, //Width
                4,
                None,
                None,
            )),
            e_field_big: assets.load_with_settings(
                "VFX/Flipbooks/TFlip_ElectricShield_Higher.png",
                make_nearest,
            ),
            e_field_big_layout: assets.add(TextureAtlasLayout::from_grid(
                UVec2::splat(128),
                4, //Width
                4,
                None,
                None,
            )),

            laser_beam: assets.load_with_settings(
                "VFX/Other/T_TilingLaserBeam.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: bevy::image::ImageAddressMode::Repeat,
                        address_mode_v: bevy::image::ImageAddressMode::ClampToEdge,
                        ..ImageSamplerDescriptor::nearest()
                    })
                },
            ),
            laser_hit: assets
                .load_with_settings("VFX/Flipbooks/TFlip_LaserBeamImpact.png", make_nearest),
            laser_hit_layout: assets.add(TextureAtlasLayout::from_grid(
                UVec2::splat(128),
                4,
                4,
                None,
                None,
            )),
        }
    }
}

#[derive(Component)]
pub struct Cannon {
    timer: Timer,
}

#[derive(Component)]
pub struct CannonBullet;

pub fn spawn_cannons(cannon: &Handle<Image>, n: usize) -> Vec<impl Bundle> {
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
        let pos = pos * 2.0;
        cannons.push((
            Sprite {
                image: cannon.clone(),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.0),
            Cannon {
                timer: Timer::from_seconds(0.2, TimerMode::Repeating),
            },
        ))
    }

    cannons
}

#[derive(Component)]
pub struct EField;

pub fn spawn_e_field(assets: &Res<WeaponAssets>, n: usize) -> Vec<impl Bundle> {
    let radius = match n {
        0 => 0.0,
        1 => 96.0,
        2 => 128.0,
        3 => 256.0,
        _ => todo!(),
    };

    let mut fields = Vec::new();

    fields.push((
        match n {
            3 => Sprite {
                image: assets.e_field_big.clone(),
                custom_size: Some(Vec2 {
                    x: radius,
                    y: radius,
                }),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.e_field_big_layout.clone(),
                    index: 0,
                }),
                ..default()
            },

            _ => Sprite {
                image: assets.e_field.clone(),
                custom_size: Some(Vec2 {
                    x: radius,
                    y: radius,
                }),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.e_field_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
        },
        CollisionEventsEnabled,
        ContinuosDamage {
            damage_per_frame: 100,
        },
        Collider::circle(radius / 2.4),
        AnimatedSprite::new(30, 15, AnimationType::Repeating),
        Transform::from_xyz(0.0, 0.0, -0.2),
        EField,
        Sensor,
    ));

    fields
}

#[derive(Component)]
pub struct Laser {
    firing: bool,
    level: usize,
    damage: usize,
    fire: Duration,
    cooldown: Duration,
    timer: Timer,
}

pub fn fire_cannon(
    mut commands: Commands,
    player: Single<(&LinearVelocity, &Transform, &Children), With<Player>>,
    mut cannons: Query<
        (&GlobalTransform, &mut Transform, &mut Cannon),
        (Without<Player>, Without<Enemy>),
    >,
    enemies: Query<&Transform, With<Enemy>>,
    assets: Res<WeaponAssets>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let (player_velocity, player_trans, children) = player.into_inner();
    let player_pos = player_trans.translation;

    let closest_enemy = enemies
        .iter()
        .map(|trans| trans.translation)
        .min_by_key(|x| ((x - player_pos).length() * 100.0) as i32);

    gizmos.circle_2d(
        Isometry2d::from_translation(closest_enemy.unwrap_or(Vec3::ZERO).xy()),
        50.0,
        Color::srgb(1.0, 0.0, 0.0),
    );

    for child in children {
        let Ok((global_transform, mut transform, mut cannon)) = cannons.get_mut(*child) else {
            continue;
        };

        if let Some(closest_enemy) = closest_enemy {
            let enemy_dir = (closest_enemy - global_transform.translation()).xy();

            let rotation = player_trans.rotation.inverse()
                * Quat::from_rotation_z(-enemy_dir.angle_to(Vec2::Y));
            transform.rotation = rotation;
        }

        cannon.timer.tick(time.delta());

        if !cannon.timer.just_finished() {
            continue;
        }

        let dir = global_transform.rotation() * Vec3::new(0.0, 1.0, 0.0);

        let pos = global_transform.translation();

        //Spawning bullet
        commands
            .spawn((
                assets.get_laser_shot_sprite(),
                StateScoped(Screen::Gameplay),
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
                |trigger: Trigger<OnCollisionStart>,
                 mut commands: Commands,
                 enemies: Query<Entity, With<Enemy>>| {
                    if !enemies.contains(trigger.collider) {
                        return;
                    }
                    commands.trigger_targets(Damage(50), trigger.collider);
                    commands.entity(trigger.target()).despawn();
                },
            );

        //Spawning muzzle flash
        commands.spawn((
            Sprite {
                image: assets.muzzle_flash.clone(),
                custom_size: Some(Vec2 { x: 86.0, y: 86.0 }),
                texture_atlas: Some(TextureAtlas {
                    layout: assets.muzzle_flash_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            StateScoped(Screen::Gameplay),
            Transform::from_translation(pos + dir * 20.0),
            AnimatedSprite::new(15, 9, AnimationType::Once),
        ));
    }
}

const LASER_FIRE_TIME: u64 = 6000;
const LASER_COOLDOWN_TIME: u64 = 4000;

impl Laser {
    fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);

        let just_finished = self.timer.just_finished();
        if just_finished {
            self.firing = !self.firing;

            self.timer.reset();

            self.timer
                .set_duration(Duration::from_millis(if self.firing {
                    LASER_FIRE_TIME
                } else {
                    LASER_COOLDOWN_TIME
                }));
        }
    }
}

pub fn spawn_laser(level: usize) -> impl Bundle {
    let fire = Duration::from_millis(LASER_FIRE_TIME);
    let cooldown = Duration::from_millis(LASER_COOLDOWN_TIME);

    (
        Transform::from_translation(Vec3::new(0.0, 16.0, 0.0)),
        Laser {
            firing: true,
            level,
            fire,
            damage: 100,
            cooldown,
            timer: Timer::new(fire, TimerMode::Once),
        },
        RayCaster::new(Vec2 { x: 0.0, y: 0.0 }, Dir2::Y)
            .with_max_distance(4000.0)
            .with_max_hits(100)
            .with_solidness(false),
    )
}

#[derive(Component)]
struct LaserBeam {
    len: f32,
}

#[derive(Component)]
struct LaserHit;

fn animate_laser(
    beams: Query<(&mut Sprite, &LaserBeam, &Children)>,
    mut hit: Query<&mut Transform, With<LaserHit>>,
    mut collider: Query<(&mut Transform, &mut Collider), Without<LaserHit>>,
) {
    for (mut sprite, beam, children) in beams {
        let Some(rect) = sprite.rect.as_mut() else {
            continue;
        };
        rect.min.x -= 2.0;
        rect.max.x -= 2.0;

        sprite.custom_size.as_mut().unwrap().x = beam.len / 2.0;

        let Ok(mut hit) = hit.get_mut(children[1]) else {
            continue;
        };
        hit.translation.x = beam.len / 2.0;

        let Ok(mut collider) = collider.get_mut(children[0]) else {
            continue;
        };

        collider.0.translation = Vec3::new(beam.len / 4.0, 0.0, 0.0);
        *collider.1 = Collider::rectangle(beam.len, 32.0);
    }
}

fn fire_laser(
    timer: Res<Time>,
    mut commands: Commands,
    assets: Res<WeaponAssets>,
    lasers: Query<(Entity, &mut Laser, &RayHits, &RayCaster, Option<&Children>)>,
    enemies: Query<Entity, With<Enemy>>,
    mut laser_sprite: Query<&mut LaserBeam>,
) {
    for (laser_ent_id, mut laser, ray_hits, raycaster, children) in lasers {
        laser.update_timer(timer.delta());
        let mut laser_ent = commands.entity(laser_ent_id);

        let closest_hit = match ray_hits
            .iter_sorted()
            .find(|hit| enemies.contains(hit.entity))
        {
            Some(hit) => hit.distance,
            None => raycaster.max_distance,
        };

        if !laser.firing {
            laser_ent.despawn_related::<Children>();
            continue;
        } else {
            // spawn in the laser
            if children.is_none_or(|x| x.is_empty()) {
                laser_ent.with_children(|parent| {
                    parent
                        .spawn((
                            Transform::from_rotation(Quat::from_rotation_z(PI / 2.0)),
                            Sprite {
                                custom_size: Some(Vec2::new(closest_hit, 32.0)),
                                image: assets.laser_beam.clone(),
                                rect: Some(Rect {
                                    min: Vec2::ZERO,
                                    max: Vec2::splat(128.0),
                                }),
                                image_mode: SpriteImageMode::Tiled {
                                    tile_x: true,
                                    tile_y: false,
                                    stretch_value: 3.0,
                                },
                                anchor: Anchor::CenterLeft,
                                ..default()
                            },
                            LaserBeam { len: closest_hit },
                        ))
                        .with_children(|laser_sprite| {
                            laser_sprite.spawn((
                                Transform::from_xyz(closest_hit / 2.0 / 2.0, 0.0, 0.0),
                                Collider::rectangle(closest_hit, 32.0),
                                CollisionEventsEnabled,
                                ContinuosDamage {
                                    damage_per_frame: laser.damage,
                                },
                                Sensor,
                            ));
                            laser_sprite.spawn((
                                Transform::from_xyz(closest_hit, 0.0, 0.0),
                                assets.get_laser_hit(),
                                LaserHit,
                            ));
                        });
                });
                continue;
            }
        }
        let Some(children) = children else { continue };
        // change the current laser
        let Ok(mut beam) = laser_sprite.get_mut(children[0]) else {
            continue;
        };
        beam.len = closest_hit;
    }
}
