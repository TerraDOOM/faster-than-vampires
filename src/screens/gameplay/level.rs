//! Spawn the main level.

use avian2d::prelude::{Collider, RigidBody, Sensor};
use rand::Rng;

use bevy::{color::palettes::css::GREEN, prelude::*};

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
    combat::Health,
    enemies::ShipType,
    player::{gen_player, Player, PlayerAssets},
    GameplayLogic,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();

    app.register_type::<UIAssets>();
    app.load_resource::<UIAssets>();

    app.add_systems(Update, world_update.in_set(GameplayLogic));
}

const LVL1X: f32 = 0.0;
const LVL2X: f32 = 10000.0;
const LVL3X: f32 = 20000.0;
const LVL4X: f32 = 30000.0;
const LVL5X: f32 = 40000.0;
const LVL6X: f32 = 50000.0;
const LVL7X: f32 = 60000.0;

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
        }
    }
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        use crate::util::make_nearest;
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            background: assets.load_with_settings("images/level/background.png", make_nearest),
            planet1: assets.load_with_settings("images/level/Planet1.png", make_nearest),
            planet2: assets.load_with_settings("images/level/Planet2.png", make_nearest),
            planet3: assets.load_with_settings("images/level/planet3.png", make_nearest),
            planet4: assets.load_with_settings("images/level/Planet4.png", make_nearest),
            planet5: assets.load_with_settings("images/level/Planet5.png", make_nearest),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct ObjectiveMarker;

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
                PlanetType::EarthPlanet,
                true
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL2X * 0.1, 0.0),
                PlanetType::LavaPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL3X * 0.1, -300.0),
                PlanetType::WaterPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL3X * 0.1, 300.0),
                PlanetType::DesertPlanet,
                false
            ),
            gen_planet(
                &level_assets,
                &ui_assets,
                Vec2::new(LVL4X * 0.1, 300.0),
                PlanetType::GreenPlanet,
                false
            ),
            gen_flagship(&entity_assets),
        ],
    ));
    commands.spawn(gen_ui(&ui_assets));
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PlanetType {
    GreenPlanet,
    LavaPlanet,
    EarthPlanet,
    WaterPlanet,
    DesertPlanet,
}

#[derive(Component)]
pub struct ShopMarker;

#[derive(Component)]
pub struct Planet {
    pub x: f32,
    pub y: f32,
    pub has_shopped: bool,
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
        },
        Sprite {
            image: match planet_name {
                PlanetType::GreenPlanet => assets.planet3.clone(),
                PlanetType::LavaPlanet => assets.planet2.clone(),
                PlanetType::EarthPlanet => assets.planet1.clone(),
                PlanetType::WaterPlanet => assets.planet4.clone(),
                PlanetType::DesertPlanet => assets.planet5.clone(),
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
                            width: Val::Px(16.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 1.0, 0.1, 0.8)),
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
    player: Single<(&Transform, &Health), With<Player>>,
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
    planets: Query<(&Transform, &mut Planet)>, //For shop checking
    mut next_menu: ResMut<NextState<Menu>>,
) {
    gizmo.rect_2d(Isometry2d::IDENTITY, Vec2::new(100.0, 100.0), GREEN);

    let (player, health) = player.into_inner();

    //Position
    ui_position.0 = ((player.translation.x) as i32).to_string();

    //HP bar
    let hp_width = health.0 as f32 / 100.0 * 40.0;
    hp_bar.width = Val::Percent(hp_width);
    anti_hp_bar.width = Val::Percent(40.0 - hp_width);

    //Planet collision
    for (planet_transform, mut planet) in planets {
        if !planet.has_shopped
            && (player.translation - planet_transform.translation).length() < 200.0
        {
            planet.has_shopped = true;
            next_menu.set(Menu::Buy);
        }
    }

    //Mini-map
    mini_map_enemy.width = Val::Percent(1000.0 / LVL7X * 100.0);
    mini_map_pos.left = Val::Percent(player.translation.x / LVL7X * 100.0);
    mini_map_pos.top = Val::Percent(45.0 - player.translation.y / LVL7X * 100.0);

    //mini_map[0].Node.width = Val::Percent(10.0);

    //Enemy spawning depending on biome

    if player.translation.x < LVL2X {
        spawn_enemy(
            commands,
            entity_assets,
            10,
            ShipType::Asteroid,
            player.translation,
        );
    } else if player.translation.x < LVL3X {
        spawn_enemy(
            commands,
            entity_assets,
            5,
            ShipType::Asteroid,
            player.translation,
        );
    }
}

pub fn spawn_enemy(
    mut commands: Commands,
    entity_assets: Res<EntityAssets>,
    spawnrate: usize,
    ship_type: ShipType,
    player_pos: Vec3,
) {
    let mut rng = rand::thread_rng();
    if 0 == rng.gen_range(0..spawnrate) {
        let rand_angle = (rng.gen_range(0..360) as f32 / 180.0 * 3.14) as f32;
        let relative_postion = Vec2::new(rand_angle.sin(), rand_angle.cos()) * 900.0;
        let position = Vec2::new(player_pos.x, player_pos.y) + relative_postion;

        let rand_deviation =
            Vec2::new(rng.gen_range(-10..10) as f32, rng.gen_range(-10..10) as f32) / 20.0;
        let rand_speed = (rng.gen_range(100..300) as f32) / 1500.0;

        match ship_type {
            ShipType::Asteroid => commands.spawn((
                Name::new("Asteroid"),
                Transform::default(),
                Visibility::default(),
                StateScoped(Screen::Gameplay),
                children![gen_asteroid(
                    &entity_assets,
                    position,
                    -(relative_postion + rand_deviation) * rand_speed
                )],
            )),
            ShipType::EmpireGoon => commands.spawn((
                Name::new("Goon"),
                Transform::default(),
                Visibility::default(),
                StateScoped(Screen::Gameplay),
                children![gen_goon(&entity_assets, position,)],
            )),
            ShipType::Rammer => commands.spawn((
                Name::new("Rammer"),
                Transform::default(),
                Visibility::default(),
                StateScoped(Screen::Gameplay),
                children![gen_rammer(&entity_assets, position, Vec2::ZERO)],
            )),
            _ => commands.spawn((
                Name::new("???"),
                Transform::default(),
                Visibility::default(),
                StateScoped(Screen::Gameplay),
                children![gen_goon(&entity_assets, position,)],
            )),
        };
    }
}
