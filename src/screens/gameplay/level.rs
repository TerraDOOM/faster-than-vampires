//! Spawn the main level.

use std::time::Instant;

use rand::Rng;

use bevy::{
    color::{self, palettes::css::GREEN},
    gizmos,
    image::{ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    prelude::*,
};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    screens::{
        gameplay::{
            enemies::{gen_asteroid, gen_enemy, EntityAssets, Ship, ShipType},
            upgrade_menu::generate_buy_menu,
        },
        Screen,
    },
    PausableSystems,
};

use super::{
    combat::{Damage, Health},
    enemies::gen_goon,
    player::{gen_player, Player, PlayerAssets},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();

    app.register_type::<UIAssets>();
    app.load_resource::<UIAssets>();

    app.add_systems(
        Update,
        (world_update
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay))),
    );

    app.add_observer(|trigger: Trigger<Damage>, mut query: Query<&mut Health>| {
        let Ok(mut health) = query.get_mut(trigger.target()) else {
            return;
        };
        health.0 -= trigger.0;
    });
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
    #[dependency]
    planet2: Handle<Image>,
    #[dependency]
    planet3: Handle<Image>,
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct UIAssets {
    #[dependency]
    pub font: Handle<Font>,
}

impl FromWorld for UIAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            font: assets.load("FiraSans.ttf"),
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
        }
    }
}

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
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Name::new("Background"),
        Transform::from_xyz(0.0, 0.0, -1.0),
        BackgroundAccess,
        Sprite {
            color: Color::linear_rgba(0.0, 0.0, 0.0, 0.9),
            custom_size: Some(Vec2 {
                x: 1280.0,
                y: 960.0,
            }),
            ..default()
        },
        children![
            Transform::from_xyz(0.0, 0.0, -2.0),
            Sprite {
                image: level_assets.background.clone(),
                custom_size: Some(Vec2 {
                    x: 1280.0,
                    y: 960.0,
                }),
                ..default()
            },
        ],
    ));

    commands.spawn((
        Name::new("Level"),
        Transform::from_xyz(0.0, 0.0, -1.0),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            gen_player(400.0, &player_assets, &mut texture_atlas_layouts),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            ),
            gen_goon(&entity_assets),
            map_gen(&level_assets),
        ],
    ));
    commands.spawn(gen_UI(&ui_assets));

    generate_buy_menu(commands, &ui_assets);
    //commands.spawn(gen_shop(&ui_assets));
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PlanetType {
    GreenPlanet,
    LavaPlanet,
    EarthPlanet,
}

#[derive(Component)]
pub struct Planet;
pub fn gen_planet(assets: &LevelAssets, position: Vec2, planet_name: PlanetType) -> impl Bundle {
    println!("Planet spawned");

    (
        Planet,
        Sprite {
            image: match planet_name {
                PlanetType::GreenPlanet => assets.planet3.clone(),
                PlanetType::LavaPlanet => assets.planet2.clone(),
                PlanetType::EarthPlanet => assets.planet1.clone(),
                _ => assets.planet1.clone(),
            },
            custom_size: Some(Vec2 { x: 128.0, y: 128.0 }),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 0.0),
    )
}

pub fn map_gen(assets: &LevelAssets) -> impl Bundle {
    (gen_planet(assets, Vec2::new(0.0, 0.0), PlanetType::EarthPlanet));
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIBox;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct UIPostition;

pub fn gen_UI(UI_assets: &Res<UIAssets>) -> impl Bundle {
    (
        Name::new("UIBox"),
        UIBox,
        BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(15.0),
            right: Val::Vw(0.0),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ZIndex(2),
        children![(
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
                UIPostition,
                Text::new("Loading X"),
                TextFont {
                    font: UI_assets.font.clone(),
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.9)),
            )]
        ),],
    )
}

pub fn world_update(
    time: Res<Time>,
    mut commands: Commands,
    entity_assets: Res<EntityAssets>,
    mut gizmo: Gizmos,
    player: Single<&Transform, With<Player>>,
    mut ui_position: Single<&mut Text, With<UIPostition>>,
) {
    let mut rng = rand::thread_rng();

    gizmo.rect_2d(Isometry2d::IDENTITY, Vec2::new(100.0, 100.0), GREEN);

    ui_position.0 = ((15000.0 - player.translation.x) as i32).to_string();

    if 0 == rng.gen_range(0..30) {
        let rand_angle = (rng.gen_range(0..360) as f32 / 180.0 * 3.14) as f32;
        let position = Vec2::new(rand_angle.sin(), rand_angle.cos()) * 900.0;

        let rand_deviation =
            Vec2::new(rng.gen_range(-10..10) as f32, rng.gen_range(-10..10) as f32) / 20.0;
        let rand_speed = (rng.gen_range(100..300) as f32) / 1500.0;

        commands.spawn((
            Name::new("Goon"),
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
            children![gen_asteroid(
                &entity_assets,
                position,
                -(position + rand_deviation) * rand_speed
            )],
        ));
    }
}
