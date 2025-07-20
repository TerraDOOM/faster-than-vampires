use std::time::Duration;

use crate::{menus::Menu, screens::Screen};
use avian2d::prelude::LinearVelocity;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::{enemies::FlagshipAI, level::Planet, player::Player, GameplayLogic};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Menu::Cutscene),
        spawn_cutscene.in_set(GameplayLogic),
    );

    app.add_systems(
        FixedUpdate,
        cutscene_update.run_if(in_state(Menu::Cutscene)),
    );

    app.add_systems(FixedUpdate, tick_cutscene.run_if(in_state(Menu::Cutscene)));
}

fn finish_cutscene(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

pub fn spawn_cutscene(mut commands: Commands) {
    commands.spawn((
        Cutscene {
            timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
        },
        Name::new("Cutscene resource"),
        StateScoped(Screen::Gameplay),
    ));
}

#[derive(Component, Debug, Eq, PartialEq, Default, Reflect)]
pub struct Cutscene {
    timer: Timer,
}

pub fn tick_cutscene(
    time: Res<Time>,
    mut cutsceneprogress: Query<
        &mut Cutscene,
        (Without<Camera2d>, Without<FlagshipAI>, Without<Player>),
    >,
) {
    for mut cutscene in cutsceneprogress {
        cutscene.timer.tick(time.delta());
    }
}

pub fn cutscene_update(
    transforms: Query<&mut Transform, Without<Camera2d>>,
    player: Single<Entity, With<Player>>,
    mut player_physics: Single<&mut LinearVelocity, With<Player>>,
    flagship: Single<Entity, With<FlagshipAI>>,
    next_menu: ResMut<NextState<Menu>>,
    mut camera: Single<&mut Transform, With<Camera2d>>,

    cutsceneprogress: Single<&mut Cutscene, Without<Camera2d>>,
) {
    let progress = (1 as f32
        - cutsceneprogress.timer.elapsed().as_secs_f32()
            / cutsceneprogress.timer.duration().as_secs_f32())
    .clamp(0.0, 1.0);

    let [mut player, mut flagship] = transforms.get_many_mut_inner([*player, *flagship]).unwrap();
    player.translation = Vec3::new(-390.0 * progress, 0.0, 0.0);
    flagship.translation = Vec3::new(-200.0 * progress - 700.0, 0.0, 0.0);
    camera.translation = Vec3::new(
        flagship.translation.x * (1.0 - in_quad_blend(progress)),
        0.0,
        0.0,
    );
    //Planet collision
    //mini_map[0].Node.width = Val::Percent(10.0);

    if cutsceneprogress.timer.finished() {
        player_physics.x = 200.0;
        finish_cutscene(next_menu);
    }

    //Enemy spawning depending on biome
}

pub fn bezier_blend(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn in_quad_blend(t: f32) -> f32 {
    let quad1 = |t| 1.0 - t * t * (3.0 - 2.0 * t);
    let quad2 = |t| t * t;
    if t < 0.5 {
        quad1(t * 2.0)
    } else {
        quad2((t - 0.5) * 2.0)
    }
}
