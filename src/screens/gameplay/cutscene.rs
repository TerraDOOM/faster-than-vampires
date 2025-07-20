use std::{f32::consts::PI, time::Duration};

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
    let progress = cutsceneprogress.timer.elapsed().as_secs_f32()
        / cutsceneprogress.timer.duration().as_secs_f32();

    fn player_pos(t: f32) -> Vec3 {
        Vec3::new(-390.0 * (1.0 - t), 0.0, 0.0)
    }

    fn flagship_pos(t: f32) -> Vec3 {
        Vec3::new(-200.0 * (1.0 - t) - 700.0, 0.0, 0.0)
    }

    let [mut player, mut flagship] = transforms.get_many_mut_inner([*player, *flagship]).unwrap();
    player.translation = player_pos(progress);
    flagship.translation = flagship_pos(progress);

    camera.translation = flagship_pos(0.5).lerp(
        if progress < 0.5 {
            player_pos(0.0)
        } else {
            player_pos(1.0)
        },
        double_ease_in_out(progress),
    );

    //Planet collision
    //mini_map[0].Node.width = Val::Percent(10.0);

    if cutsceneprogress.timer.finished() {
        player_physics.x = 200.0;
        finish_cutscene(next_menu);
    }

    //Enemy spawning depending on biome
}

fn ease_in_out(t: f32) -> f32 {
    -((PI * t).cos() - 1.0) / 2.0
}

fn double_ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        1.0 - ease_in_out(2.0 * t)
    } else {
        ease_in_out((t - 0.5) * 2.0)
    }
}
