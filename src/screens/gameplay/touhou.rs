use bevy::{
    ecs::query::QueryFilter, input::common_conditions::input_just_pressed,
    render::camera::ScalingMode,
};
use bullet::AltFire;
use enemy::{Animation, EnemyMarker, Health};

use crate::prelude::*;

mod bullet;
mod enemy;

#[derive(Component, Clone, Default, Debug)]
struct TouhouMarker;
#[derive(Component, Default, Debug)]
struct PlayerMarker;
#[derive(Component, Default)]
struct TouhouCamera;

#[derive(QueryFilter)]
struct PlayerFilter {
    filter: With<PlayerMarker>,
}

type PlayerQ<'a, T> = Single<'a, T, With<PlayerMarker>>;

#[derive(Component, Default, Copy, Clone, Debug)]
struct Collider {
    radius: f32,
}

impl Collider {
    fn to_circle(&self, pos: Vec2) -> Circle {
        let Self { radius } = *self;
        Circle { pos, radius }
    }

    fn new(radius: f32) -> Self {
        Self { radius }
    }
}

#[derive(Default, Copy, Clone, Debug)]
struct Circle {
    pos: Vec2,
    radius: f32,
}

#[derive(Resource, Default)]
struct ShowGizmos {
    enabled: bool,
}

impl Circle {
    fn new(radius: f32, pos: Vec2) -> Self {
        Self { pos, radius }
    }

    #[allow(dead_code)]
    fn within(&self, rect: Rect) -> bool {
        let Self { pos, radius } = *self;

        let bounding_rect = Rect::from_center_size(pos, Vec2::splat(radius));

        rect.contains(bounding_rect.min) && rect.contains(bounding_rect.max)
    }

    fn hits(&self, other: Circle) -> bool {
        (self.pos - other.pos).length() - (self.radius + other.radius) < 0.0
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MissionState {
    #[default]
    Ongoing,
    Success,
    Fail,
}

#[derive(Resource, Default)]
struct GameplayRect {
    rect: Rect,
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum TouhouSets {
    EnterTouhou,
    Gameplay,
}

#[derive(Component, Deref, DerefMut, Default)]
struct Speed(f32);

#[derive(Bundle)]
struct AnimatedSprite {
    sprite: Sprite,
    animation: Animation,
}

pub fn make_animation(
    sprite: Sprite,
    layout: Handle<TextureAtlasLayout>,
    time: f32,
    n_anims: usize,
) -> AnimatedSprite {
    AnimatedSprite {
        sprite: Sprite {
            texture_atlas: Some(TextureAtlas { layout, index: 0 }),
            ..sprite
        },
        animation: Animation::new(time, n_anims, 0),
    }
}

#[derive(Resource)]
pub struct PlayerAssets {
    dead: Handle<Image>,
    atlas_layout: TextureAtlasLayout,
    atlas_handle: Handle<TextureAtlasLayout>,
    alive_sheet: Handle<Image>,
}

pub fn touhou_plugin(app: &mut App) {
    let touhou_gameplay_pred = || {
        TouhouSets::Gameplay
            .run_if(in_state(GameState::Touhou).and(in_state(MissionState::Ongoing)))
    };

    app.add_plugins((bullet::bullet_plugin, enemy::enemy_plugin))
        .init_state::<MissionState>()
        .insert_resource(ShowGizmos { enabled: false })
        .add_systems(
            Startup,
            (load_player_assets, load_touhou_assets, create_gameplay_rect),
        )
        .add_systems(
            OnEnter(GameState::Touhou),
            (
                spawn_player,
                make_bg,
                bullet::config_loadout.after(spawn_player),
                make_game_camera,
                set_mission_status,
                play_music,
                spawn_hud,
            )
                .in_set(TouhouSets::EnterTouhou),
        )
        .add_systems(
            FixedUpdate,
            (update_invulnerability, do_movement, update_hud).in_set(TouhouSets::Gameplay),
        )
        .add_systems(
            FixedPostUpdate,
            (enemy_dead, last_enemy_dead).in_set(TouhouSets::Gameplay),
        )
        .add_systems(
            FixedPostUpdate,
            (on_death.run_if(player_dead), on_damage)
                .chain()
                .after(bullet::process_player_hits),
        )
        .add_systems(
            Update,
            (
                scroll_map.run_if(in_state(GameState::Touhou)),
                toggle_gizmos.run_if(input_just_pressed(KeyCode::Space)),
                animate_player,
            ),
        )
        .add_systems(PostUpdate, draw_gizmos.in_set(TouhouSets::Gameplay))
        // set them all to only run if gamestate is touhou
        .configure_sets(FixedUpdate, touhou_gameplay_pred())
        .configure_sets(FixedPreUpdate, touhou_gameplay_pred())
        .configure_sets(FixedPostUpdate, touhou_gameplay_pred())
        .add_systems(OnExit(GameState::Touhou), nuke_touhou);
}

#[derive(Component)]
struct AmmoCount;
#[derive(Component)]
struct LifeCount;
#[derive(Component)]
struct HPBar;

fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            TouhouMarker,
            Node {
                width: Val::Vw(20.0),
                height: Val::Vh(10.0),
                left: Val::Px(0.),
                bottom: Val::Px(0.),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ZIndex(1),
        ))
        .with_child((
            Text::new("lol"),
            AmmoCount,
            TextFont {
                font: asset_server.load("fonts/Pixelfont/slkscr.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.9)),
        ));

    commands
        .spawn((
            TouhouMarker,
            Node {
                width: Val::Vw(20.0),
                height: Val::Vh(10.0),
                left: Val::Px(0.),
                bottom: -Val::Vh(10.),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ZIndex(1),
        ))
        .with_child((
            Text::new("lol"),
            LifeCount,
            TextFont {
                font: asset_server.load("fonts/Pixelfont/slkscr.ttf"),
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.4, 0.4)),
        ));

    commands.spawn((
        TouhouMarker,
        Node {
            width: Val::Vw(80.0),
            height: Val::Vh(10.0),
            left: Val::Vw(10.),
            bottom: -Val::Vh(90.),
            ..default()
        },
        HPBar,
        BackgroundColor(Color::srgba(1.0, 0.0, 0.0, 0.95)),
        ZIndex(1),
    ));
}

fn update_hud(
    player: Option<Single<(&Ammo, &Life), PlayerFilter>>,
    enemy_hp: Option<Single<&Health, With<EnemyMarker>>>,
    mut ammo_text: Query<&mut Text, (With<AmmoCount>, Without<LifeCount>)>,
    mut hp_text: Query<&mut Text, (With<LifeCount>, Without<AmmoCount>)>,
    mut hp_bar: Query<&mut Node, (With<HPBar>)>,
) {
    let Some((ammo_count, lives_count)) = player.map(|x| x.into_inner()) else {
        return;
    };
    let Some(enemy_hp) = enemy_hp else {
        return;
    };
    for mut text in &mut ammo_text {
        **text = format!["Ammo: {}", **ammo_count];
    }

    for mut text in &mut hp_text {
        **text = format!["Lives: {}", **lives_count];
    }

    for mut node in &mut hp_bar {
        *node = Node {
            width: Val::Vw(80. * (***enemy_hp as f32) / 4000.),
            height: Val::Vh(10.0),
            left: Val::Vw(10.),
            bottom: -Val::Vh(90.),
            ..default()
        }
    }
}

fn play_music(
    mission_params: Res<MissionParams>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<TouhouAssets>,
) {
    commands.spawn((
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            ..default()
        },
        AudioPlayer::new(asset_server.load(match mission_params.enemy {
            Enemies::RedGirl => "Music/Comabt1.ogg",
            Enemies::Lizard => "Music/Combat2.ogg",
            Enemies::Tentacle => "Music/Combat3.ogg",
            Enemies::MoonGirl => "Music/Combat4.ogg",
            _ => "Music/Comabt1.ogg",
        })),
        TouhouMarker,
    ));
}

fn toggle_gizmos(mut r: ResMut<ShowGizmos>) {
    r.enabled = !r.enabled;
}

fn enemy_dead(mut commands: Commands, enemies: Query<(Entity, &Health), With<EnemyMarker>>) {
    for (ent, health) in &enemies {
        if **health == 0 {
            commands.entity(ent).remove::<(EnemyMarker, Health)>();
        }
    }
}

fn last_enemy_dead(
    enemies: Query<Entity, With<EnemyMarker>>,
    mut mission_state: ResMut<NextState<MissionState>>,
) {
    if enemies.is_empty() {
        mission_state.set(MissionState::Success)
    }
}

fn set_mission_status(mut mission_status: ResMut<NextState<MissionState>>) {
    mission_status.set(MissionState::Ongoing);
}

fn nuke_touhou(
    mut commands: Commands,
    touhou_objects: Query<Entity, With<TouhouMarker>>,
    touhou_camera: Query<Entity, With<TouhouCamera>>,
) {
    for obj in &touhou_objects {
        commands.entity(obj).try_despawn_recursive();
    }

    for obj in &touhou_camera {
        commands.entity(obj).despawn_recursive();
    }
}

const N_SHIP_TEXTURES: usize = 3;

fn load_player_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let atlas =
        TextureAtlasLayout::from_grid(UVec2::splat(64), N_SHIP_TEXTURES as u32, 2, None, None);

    commands.insert_resource(PlayerAssets {
        dead: asset_server.load("dead.png"),
        alive_sheet: asset_server.load("Xcom_hud/Playerrocket1-sheet.png"),
        atlas_layout: atlas.clone(),
        atlas_handle: asset_server.add(atlas),
    })
}

fn load_touhou_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(TouhouAssets {
        redgirl_sheet: asset_server.load("Enemies/Girlanimation1-sheet.png"),
        redgirl_layout: asset_server.add(TextureAtlasLayout::from_grid(
            UVec2::splat(64),
            3,
            1,
            None,
            None,
        )),
        bullet1: asset_server.load("bullets/bullet1.png"),
        kaguya_sheet: asset_server.load("Enemies/Moongirl1-sheet.png"),
        kaguya_layout: asset_server.add(TextureAtlasLayout::from_grid(
            UVec2::splat(128),
            5,
            1,
            None,
            None,
        )),
        tentacle: asset_server.load("Enemies/Babyalien.png"),
        lizard_sheet: asset_server.load("Enemies/Lizard1-sheet.png"),
        lizard_layout: asset_server.add(TextureAtlasLayout::from_grid(
            UVec2::splat(64),
            3,
            1,
            None,
            None,
        )),
        rocket: asset_server.load("Xcom_hud/rocket2.png"),

        moongirl_bullet_sheet: asset_server.load("bullets/Moonbullet1-sheet.png"),
        moongirl_layout: asset_server.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            5,
            1,
            None,
            None,
        )),

        lizard_bullet_sheet: asset_server.load("bullets/Lizard1_bullet-sheet.png"),
        lizard_bullet_layout: asset_server.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            3,
            1,
            None,
            None,
        )),

        girl_bullet_sheet: asset_server.load("bullets/Girlbullet.png"),
        girl_bullet_layout: asset_server.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            2,
            1,
            None,
            None,
        )),

        girl_bullet2_sheet: asset_server.load("bullets/Girlbullet2.png"),
        girl_bullet2_layout: asset_server.add(TextureAtlasLayout::from_grid(
            UVec2::splat(32),
            2,
            1,
            None,
            None,
        )),
    })
}

#[derive(Resource)]
pub struct TouhouAssets {
    redgirl_sheet: Handle<Image>,
    redgirl_layout: Handle<TextureAtlasLayout>,
    kaguya_sheet: Handle<Image>,
    kaguya_layout: Handle<TextureAtlasLayout>,
    bullet1: Handle<Image>,
    tentacle: Handle<Image>,
    lizard_sheet: Handle<Image>,
    lizard_layout: Handle<TextureAtlasLayout>,
    rocket: Handle<Image>,

    moongirl_bullet_sheet: Handle<Image>,
    moongirl_layout: Handle<TextureAtlasLayout>,

    lizard_bullet_sheet: Handle<Image>,
    lizard_bullet_layout: Handle<TextureAtlasLayout>,

    girl_bullet_sheet: Handle<Image>,
    girl_bullet_layout: Handle<TextureAtlasLayout>,

    girl_bullet2_sheet: Handle<Image>,
    girl_bullet2_layout: Handle<TextureAtlasLayout>,
}

fn player_dead(life: Option<PlayerQ<&Life>>) -> bool {
    life.is_some_and(|life| life.0 == 0)
}

fn on_damage(
    mut commands: Commands,
    player: Option<Single<(Entity, &mut Sprite), (PlayerFilter, Changed<Life>)>>,
) {
    let Some((ent, mut sprite)) = player.map(|p| p.into_inner()) else {
        return;
    };

    commands
        .entity(ent)
        .insert(Invulnerability(Timer::from_seconds(3.0, TimerMode::Once)));
}

fn on_death(
    player: PlayerQ<&mut Sprite>,
    player_assets: Res<PlayerAssets>,
    mut mission_status: ResMut<NextState<MissionState>>,
) {
    let mut sprite = player.into_inner();
    sprite.image = player_assets.dead.clone();
    mission_status.set(MissionState::Fail);
}

fn update_invulnerability(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invulnerability)>,
) {
    for (ent, mut timer) in &mut query {
        timer.0.tick(time.delta());

        if timer.0.finished() {
            commands.entity(ent).remove::<(Invulnerability)>();
        }
    }
}

fn make_game_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            near: -1000.0,
            far: 1000.0,
            viewport_origin: Vec2::splat(0.5),
            scaling_mode: ScalingMode::Fixed {
                width: 1920.0,
                height: 1080.0,
            },
            scale: 1.0,
            area: Rect::new(0.0, 0.0, 800.0, 600.0),
        },
        TouhouCamera,
    ));
}

#[derive(Component)]
struct ScrollingBg;

fn make_bg(mut commands: Commands, params: Res<MissionParams>, asset_server: Res<AssetServer>) {
    let map = params.map;

    let img = asset_server.load(match params.map {
        Map::Day => "Bakground/Sky1Side.png",
        Map::Night => "Bakground/Nightsky.png",
        Map::Dusk => "Bakground/sunset.png",
        Map::Moon => "Bakground/moon.png",
    });

    let img = if params.enemy == Enemies::MoonGirl {
        asset_server.load("Bakground/moon.png")
    } else {
        img
    };

    commands.spawn((
        Sprite {
            image: img,
            custom_size: Some(Vec2::new(1920.0, 1080.0)),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: false,
                stretch_value: 1080.0 / 64.0,
            },
            rect: Some(Rect {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(16.0 / 9.0 * 64.0, 64.0),
            }),
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.0, -1.0),
        ScrollingBg,
        TouhouMarker,
    ));
}

fn scroll_map(time: Res<Time>, mut map: Single<&mut Sprite, With<ScrollingBg>>) {
    let mut sprite = map.into_inner();

    let Some(rect) = sprite.rect.as_mut() else {
        panic!("wtf");
    };

    let scroll_value = time.delta_secs() * 8.0;

    rect.min.x += scroll_value;
    rect.max.x += scroll_value;
}

fn animate_player(
    player: Option<Single<(&mut Sprite, Option<&Invulnerability>), PlayerFilter>>,
    time: Res<Time>,
    player_assets: Res<PlayerAssets>,
    mut animation_timer: Local<Option<Timer>>,
    mut inverted: Local<bool>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Some((mut sprite, invuln)) = player.map(Single::into_inner) else {
        return;
    };

    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) as i32;
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) as i32;

    let direction = up - down;

    let atlas: &mut TextureAtlas = sprite.texture_atlas.as_mut().unwrap();

    atlas.index = match direction {
        1 => 0,
        0 => 1,
        -1 => 2,
        _ => unreachable!(),
    };

    if invuln.is_some() {
        let mut animation_timer =
            animation_timer.get_or_insert(Timer::from_seconds(0.05, TimerMode::Repeating));

        animation_timer.tick(time.delta());

        if animation_timer.just_finished() {
            animation_timer.reset();

            *inverted = !*inverted;
            // if inverted: +2 or +3 depending on how big the atlas is
            atlas.index += *inverted as usize * N_SHIP_TEXTURES;
        }
    }
}

#[derive(Component, Deref)]
struct Invulnerability(Timer);

#[derive(Bundle, Default)]
pub struct Player {
    sprite: Sprite,
    collider: Collider,
    transform: Transform,
    lives: Life,
    markers: (PlayerMarker, TouhouMarker),
    ammo: Ammo,
    speed: Speed,
}

#[derive(Component, Deref, DerefMut)]
pub struct Life(usize);

#[derive(Component, Deref, DerefMut, Default)]
pub struct Ammo(u32);

impl Default for Life {
    fn default() -> Self {
        Life(1)
    }
}

pub fn create_gameplay_rect(mut commands: Commands) {
    const SIZE: Vec2 = Vec2::new(1920.0, 1080.0);

    commands.insert_resource(GameplayRect {
        rect: Rect {
            min: -SIZE / 2.0,
            max: SIZE / 2.0,
        },
    });
}

pub fn spawn_player(mut commands: Commands, player_assets: Res<PlayerAssets>) {
    commands.spawn(Player {
        sprite: Sprite {
            custom_size: Some(Vec2::new(100.0, 100.0)),
            image: player_assets.alive_sheet.clone(),
            anchor: bevy::sprite::Anchor::Custom(Vec2::from((0.1, -0.05))),
            texture_atlas: Some(TextureAtlas {
                layout: player_assets.atlas_handle.clone(),
                index: 0,
            }),
            ..Default::default()
        },
        transform: Transform::from_xyz(-1920.0 / 3.0, 0.0, -0.5),
        collider: Collider { radius: 7.5 },
        lives: Life(3),
        speed: Speed(6.5),
        ammo: Ammo(1000),
        ..Default::default()
    });
}

fn draw_gizmos(
    mut gizmos: Gizmos,
    area: Res<GameplayRect>,
    enabled: ResMut<ShowGizmos>,
    colliders: Query<(&Transform, &Collider)>,
) {
    if enabled.enabled || true {
        return;
    }
    use bevy::color::palettes::css::RED;

    gizmos.rect_2d(
        Isometry2d::from_translation(area.rect.center()),
        Vec2 {
            x: area.rect.width(),
            y: area.rect.height(),
        },
        RED,
    );

    for (trans, coll) in &colliders {
        gizmos.circle_2d(
            Isometry2d::from_translation(trans.translation.xy()),
            coll.radius,
            RED,
        );
    }
}

fn do_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    area: Res<GameplayRect>,
    asset_server: ResMut<AssetServer>,
    mut player_info: Single<
        (
            &Speed,
            &mut Transform,
            &Collider,
            &mut Sprite,
            Option<&AltFire>,
        ),
        With<PlayerMarker>,
    >,
) {
    let (speed, mut trans, mut collider, mut sprite, alt_fire) = player_info.into_inner();
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]) as i32 as f32;
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) as i32 as f32;
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) as i32 as f32;
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) as i32 as f32;

    let dy = up + -down;
    let dx = right + -left;

    let mut speed = **speed;

    if alt_fire.is_some() {
        speed = speed / 2.0;
    }

    let wishdir = Vec3::new(dx, dy, 0.0).normalize_or_zero() * speed;

    let new_pos = (trans.translation + wishdir).xy();

    let rect = area.rect;

    let rv = Vec2::splat(collider.radius);

    let new_pos_clamped = new_pos.clamp(rect.min + rv, rect.max - rv);

    trans.translation = new_pos_clamped.extend(0.0);
}
