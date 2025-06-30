//! Spawn the main level.

use avian2d::prelude::*;
use rand::Rng;

use bevy::{color::palettes::css::GREEN, ecs::entity, prelude::*};

use crate::{
    asset_tracking::LoadResource,
    menus::Menu,
    screens::{
        gameplay::{
            enemies::{gen_asteroid, gen_flagship, gen_goon, gen_rammer, EntityAssets},
            upgrade_menu::generate_buy_menu,
        },
        Screen,
    },
};

use super::{
    combat::{Damage, Health},
    enemies::{FlagshipAI, RammerAI, ShipType},
    player::{gen_player, Player, PlayerAssets},
    upgrade_menu::{UpgradeTypes, Upgrades},
    GameplayLogic,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();

    app.register_type::<UIAssets>();
    app.load_resource::<UIAssets>();

    app.add_systems(Update, world_update.in_set(GameplayLogic));
}

const LVL1X: f32 = 4000.0;
const LVL2X: f32 = 21000.0;
const LVL3X: f32 = 35000.0;
const LVL4X: f32 = 50000.0;
const LVL5X: f32 = 65000.0;
const LVL6X: f32 = 80000.0;
const LVL7X: f32 = 100000.0;
const YMAX: f32 = 15000.0;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    background: Handle<Image>,
    #[dependency]
    planet1: Handle<Image>,
    #[dependency]
    planet2: Handle<Image>,
    #[dependency]
    planet3: Handle<Image>,
    #[dependency]
    planet4: Handle<Image>,
    #[dependency]
    planet5: Handle<Image>,
    #[dependency]
    planet6: Handle<Image>,
    #[dependency]
    planet7: Handle<Image>,
    #[dependency]
    planet8: Handle<Image>,
    #[dependency]
    planet9: Handle<Image>,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct UIAssets {
    #[dependency]
    pub font: Handle<Font>,
    #[dependency]
    pub exclamation: Handle<Image>,
    #[dependency]
    pub button1: Handle<Image>,
    #[dependency]
    pub button2: Handle<Image>,
    #[dependency]
    pub mini_map: Handle<Image>,
}

impl FromWorld for UIAssets {
    fn from_world(world: &mut World) -> Self {
        use crate::util::make_nearest;
        let assets = world.resource::<AssetServer>();
        Self {
            font: assets.load("FiraSans.ttf"),
            exclamation: assets.load_with_settings("images/entities/Point.png", make_nearest),
            button1: assets.load_with_settings("images/ui/Main_button_clicked.png", make_nearest),
            button2: assets.load_with_settings("images/ui/Main_button_unclicked.png", make_nearest),
            mini_map: assets.load_with_settings("images/level/Map.png", make_nearest),
        }
    }
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        use crate::util::make_nearest;
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Orbital fight.ogg"),
            background: assets.load_with_settings("images/level/background.png", make_nearest),
            planet1: assets.load_with_settings("images/level/Planet1.png", make_nearest),
            planet2: assets.load_with_settings("images/level/Planet2.png", make_nearest),
            planet3: assets.load_with_settings("images/level/planet3.png", make_nearest),
            planet4: assets.load_with_settings("images/level/Planet4.png", make_nearest),
            planet5: assets.load_with_settings("images/level/Planet5.png", make_nearest),
            planet6: assets.load_with_settings("images/level/Planet6.png", make_nearest),
            planet7: assets.load_with_settings("images/level/Planet7.png", make_nearest),
            planet8: assets.load_with_settings("images/level/Planet8.png", make_nearest),
            planet9: assets.load_with_settings("images/level/Planet9.png", make_nearest),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct ObjectiveMarker;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct VisistedPlanet(pub PlanetType);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct BackgroundAccess;
/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    entity_assets: Res<EntityAssets>,
    ui_assets: Res<UIAssets>,
) {
    commands.spawn((
        Name::new("Background"),
        Transform::from_xyz(0.0, 0.0, -1.0),
        BackgroundAccess,
        Sprite {
            image: level_assets.background.clone(),
            custom_size: Some(Vec2 {
                x: 1920.0,
                y: 1080.0,
            }),
            ..default()
        },
    ));

    commands.spawn(VisistedPlanet(PlanetType::LavaPlanet));

    commands.spawn((
        Name::new("Level"),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            gen_player(400.0, &player_assets),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL1X * 0.1, 128.0),
                PlanetType::LavaPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL2X * 0.1, YMAX / 25.0),
                PlanetType::GreenPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL2X * 0.1, -YMAX / 25.0),
                PlanetType::DesertPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL3X * 0.1, 0.0),
                PlanetType::HollowPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL4X * 0.1, YMAX / 2.0),
                PlanetType::WaterPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL4X * 0.1, -YMAX / 2.0),
                PlanetType::DesertPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL5X * 0.1, 0.0),
                PlanetType::DesertPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL6X * 0.1, YMAX / 2.0),
                PlanetType::DesertPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL6X * 0.1, -YMAX / 2.0),
                PlanetType::PurplePlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL7X * 0.1, 0.0),
                PlanetType::EarthPlanet,
                false
            ),
            gen_flagship(&entity_assets),
        ],
    ));
    commands.spawn(gen_ui(&ui_assets));
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default, Reflect)]
pub enum PlanetType {
    #[default]
    LavaPlanet,
    GreenPlanet,
    EarthPlanet,
    WaterPlanet,
    DesertPlanet,
    PurplePlanet,
    GrayPlanet,
    HollowPlanet,
    SpaceStation,
}

#[derive(Component)]
pub struct ShopMarker;

#[derive(Component)]
pub struct Planet {
    pub x: f32,
    pub y: f32,
    pub has_shopped: bool,
    pub planet_type: PlanetType,
}

pub fn gen_planet(
    assets: &LevelAssets,
    ui_assets: &UIAssets,
    position: Vec2,
    planet_name: PlanetType,
    first_planet: bool,
) -> impl Bundle {
    (
        Planet {
            x: position.x,
            y: position.y,
            has_shopped: first_planet,
            planet_type: planet_name,
        },
        Sprite {
            image: match planet_name {
                PlanetType::GreenPlanet => assets.planet3.clone(),
                PlanetType::LavaPlanet => assets.planet2.clone(),
                PlanetType::EarthPlanet => assets.planet1.clone(),
                PlanetType::WaterPlanet => assets.planet4.clone(),
                PlanetType::DesertPlanet => assets.planet5.clone(),
                PlanetType::GrayPlanet => assets.planet6.clone(),
                PlanetType::PurplePlanet => assets.planet7.clone(),
                PlanetType::SpaceStation => assets.planet8.clone(),
                PlanetType::HollowPlanet => assets.planet9.clone(),
            },
            custom_size: Some(Vec2 { x: 512.0, y: 512.0 }),
            ..default()
        },
        //RigidBody::Static,
        //Collider::circle(256.0),
        Transform::from_xyz(position.x, position.y, -0.5),
        if !first_planet {
            children![
                (
                    ShopMarker,
                    Sprite {
                        image: ui_assets.exclamation.clone(),
                        custom_size: Some(Vec2 { x: 128.0, y: 128.0 }),
                        ..default()
                    },
                ),
                (
                    Name::new("ObjectiveMarker_Fake"),
                    Transform::from_xyz(0.0, 0.0, -0.3),
                    ObjectiveMarker,
                    Sprite {
                        color: Color::linear_rgba(1.0, 0.0, 0.0, 0.0),
                        custom_size: Some(Vec2 { x: 32.0, y: 32.0 }),
                        ..default()
                    },
                )
            ]
        } else {
            children![
                (
                    ShopMarker,
                    Sprite {
                        color: Color::linear_rgba(1.0, 0.0, 0.0, 0.0),
                        custom_size: Some(Vec2 { x: 32.0, y: 32.0 }),
                        ..default()
                    },
                ),
                (
                    Name::new("ObjectiveMarker"),
                    Transform::from_xyz(0.0, 0.0, -0.3),
                    ObjectiveMarker,
                    Sprite {
                        color: Color::linear_rgba(1.0, 0.0, 0.0, 0.9),
                        custom_size: Some(Vec2 { x: 32.0, y: 32.0 }),
                        ..default()
                    },
                )
            ]
        },
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIBox;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIPosition;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct HPBar;
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct HPBarAnti;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct MiniMap;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct MiniMapRed;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct MiniMapPos;

pub fn gen_ui(ui_assets: &Res<UIAssets>) -> impl Bundle {
    (
        Name::new("UIBox"),
        UIBox,
        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(15.0),
            right: Val::Vw(0.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            ..default()
        },
        ZIndex(2),
        children![
            (
                //HP-bar
                HPBar,
                Node {
                    width: Val::Percent(40.0),
                    height: Val::Percent(50.0),
                    left: Val::Percent(3.0),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 1.0, 0.2))
            ),
            (
                //HP-bar-dead
                HPBarAnti,
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(50.0),
                    left: Val::Percent(3.0),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.7, 0.15))
            ),
            (
                //Mini-map
                MiniMap,
                Node {
                    width: Val::Percent(40.0),
                    height: Val::Percent(60.0),
                    left: Val::Percent(6.0),
                    ..default()
                },
                ImageNode {
                    image: ui_assets.mini_map.clone(),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.2, 0.3)),
                children![
                    (
                        MiniMapRed,
                        Node {
                            width: Val::Percent(40.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.5)),
                    ),
                    (
                        MiniMapPos,
                        Node {
                            width: Val::Px(8.0),
                            height: Val::Px(8.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 1.0, 0.05, 0.9)),
                    )
                ]
            ),
            (
                //Spawns big button??
                Node {
                    width: Val::Px(256.0),
                    // horizontally center child text
                    justify_content: JustifyContent::Center,
                    // vertically center child text
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    UIPosition,
                    Text::new("Loading X"),
                    TextFont {
                        font: ui_assets.font.clone(),
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.9)),
                )]
            ),
        ],
    )
}

pub fn world_update(
    mut commands: Commands,
    entity_assets: Res<EntityAssets>,
    mut gizmo: Gizmos,
    player: Single<(&Transform, &Health, &Upgrades), With<Player>>,
    flagship: Single<&Transform, With<FlagshipAI>>,
    mut ui_position: Single<&mut Text, With<UIPosition>>,
    mut hp_bar: Single<
        &mut Node,
        (
            With<HPBar>,
            Without<HPBarAnti>,
            Without<MiniMapRed>,
            Without<MiniMapPos>,
        ),
    >,
    mut anti_hp_bar: Single<
        &mut Node,
        (
            With<HPBarAnti>,
            Without<HPBar>,
            Without<MiniMapRed>,
            Without<MiniMapPos>,
        ),
    >,
    mut mini_map_enemy: Single<
        &mut Node,
        (
            With<MiniMapRed>,
            Without<HPBarAnti>,
            Without<HPBar>,
            Without<MiniMapPos>,
        ),
    >,
    mut mini_map_pos: Single<
        &mut Node,
        (
            With<MiniMapPos>,
            Without<HPBarAnti>,
            Without<HPBar>,
            Without<MiniMapRed>,
        ),
    >,
    planets: Query<
        (&Transform, &mut Planet, Entity),
        (
            Without<HPBarAnti>,
            Without<HPBar>,
            Without<MiniMapRed>,
            Without<MiniMapPos>,
        ),
    >, //For shop checking
    mut next_planet: Single<&mut VisistedPlanet>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    gizmo.rect_2d(Isometry2d::IDENTITY, Vec2::new(100.0, 100.0), GREEN);

    let (player, health, upgrades) = player.into_inner();

    //Position
    ui_position.0 = ((player.translation.x) as i32).to_string();

    //HP bar
    let hp_max = (*upgrades
        .gotten_upgrades
        .get(&UpgradeTypes::Health)
        .clone()
        .unwrap()) as f32
        * 100.0;
    let hp_width = (health.0 as f32 / hp_max * 40.0).max(0.0);
    hp_bar.width = Val::Percent(hp_width);
    anti_hp_bar.width = Val::Percent(40.0 - hp_width);

    //Planet collision
    for (planet_transform, mut planet, entity) in planets {
        if !planet.has_shopped
            && (player.translation - planet_transform.translation).length() < 200.0
        {
            planet.has_shopped = true;
            next_planet.0 = planet.planet_type;
            commands.entity(entity).despawn_related::<Children>();
            next_menu.set(Menu::Buy);
        }
    }

    //Mini-map
    mini_map_enemy.width = Val::Percent(flagship.translation.x / LVL7X * 100.0);
    mini_map_pos.left = Val::Percent(player.translation.x / LVL7X * 100.0);
    mini_map_pos.top = Val::Percent(45.0 - player.translation.y / YMAX * 100.0);

    //mini_map[0].Node.width = Val::Percent(10.0);

    //Enemy spawning depending on biome

    if player.translation.y > YMAX / 2.0 {
        spawn_enemy(
            commands,
            entity_assets,
            5,
            ShipType::Asteroid,
            player.translation,
            SpawnPatterns::Top,
        );
    } else if player.translation.y < -YMAX / 2.0 {
        spawn_enemy(
            commands,
            entity_assets,
            5,
            ShipType::Asteroid,
            player.translation,
            SpawnPatterns::Bot,
        );
    } else if player.translation.x < LVL2X {
        spawn_enemy(
            commands,
            entity_assets,
            10,
            ShipType::Rammer,
            player.translation,
            SpawnPatterns::Circle,
        );
    } else if player.translation.x < LVL3X {
        spawn_enemy(
            commands,
            entity_assets,
            5,
            ShipType::Asteroid,
            player.translation,
            SpawnPatterns::Circle,
        );
    }
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SpawnPatterns {
    Circle,
    Top,
    Bot,
}

pub fn spawn_enemy(
    mut commands: Commands,
    entity_assets: Res<EntityAssets>,
    spawnrate: usize,
    ship_type: ShipType,
    player_pos: Vec3,
    spawn_patter: SpawnPatterns,
) {
    let mut rng = rand::thread_rng();
    if 0 == rng.gen_range(0..spawnrate) {
        let rand_angle = match spawn_patter {
            SpawnPatterns::Circle => (rng.gen_range(0..360) as f32 / 180.0 * 3.14) as f32,
            SpawnPatterns::Bot => (rng.gen_range(135..225) as f32 / 180.0 * 3.14) as f32,
            SpawnPatterns::Top => (rng.gen_range(135..225) as f32 / 180.0 * 3.14 + 135.0) as f32,
            _ => (rng.gen_range(45..135) as f32 / 180.0 * 3.14) as f32,
        };

        let relative_postion = Vec2::new(rand_angle.sin(), rand_angle.cos()) * 900.0;
        let position = Vec2::new(player_pos.x, player_pos.y) + relative_postion;

        let rand_deviation =
            Vec2::new(rng.gen_range(-10..10) as f32, rng.gen_range(-10..10) as f32) / 20.0;
        let rand_speed = (rng.gen_range(100..300) as f32) / 1500.0;

        match ship_type {
            ShipType::Asteroid => {
                commands.spawn((
                    Name::new("Asteroid"),
                    StateScoped(Screen::Gameplay),
                    gen_asteroid(
                        &entity_assets,
                        position,
                        -(relative_postion + rand_deviation) * rand_speed,
                    ),
                ));
            }
            ShipType::EmpireGoon => {
                commands.spawn((
                    Name::new("Goon"),
                    StateScoped(Screen::Gameplay),
                    gen_goon(&entity_assets, position),
                ));
            }
            ShipType::Rammer => {
                commands
                    .spawn((
                        Name::new("Rammer"),
                        StateScoped(Screen::Gameplay),
                        gen_rammer(&entity_assets, position, Vec2::ZERO),
                    ))
                    .observe(
                        |trigger: Trigger<OnCollisionStart>,
                         mut commands: Commands,
                         trans: Query<&Transform, With<RammerAI>>,
                         player: Single<Entity, With<Player>>,
                         assets: Res<EntityAssets>| {
                            commands.trigger_targets(Damage(30), trigger.collider);
                            commands.get_entity(trigger.target()).unwrap().despawn();
                            commands.spawn((
                                trans.get(trigger.target()).unwrap().clone(),
                                assets.get_explosion(),
                            ));
                        },
                    );
            }
            _ => {
                commands.spawn((
                    Name::new("???"),
                    StateScoped(Screen::Gameplay),
                    gen_goon(&entity_assets, position),
                ));
            }
        };
    }
}
