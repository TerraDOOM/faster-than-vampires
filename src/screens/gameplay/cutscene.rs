use std::time::Duration;

use crate::{menus::Menu, screens::Screen};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::{enemies::FlagshipAI, player::Player, GameplayLogic};

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
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    flagship: Single<&Transform, (With<FlagshipAI>, Without<Camera2d>)>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<FlagshipAI>, Without<Player>)>,

    mut cutsceneprogress: Single<
        &mut Cutscene,
        (Without<Camera2d>, Without<FlagshipAI>, Without<Player>),
    >,
) {
    let player = player.into_inner();

    let progress = 1 as f32
        - cutsceneprogress.timer.elapsed().as_secs_f32()
            / cutsceneprogress.timer.duration().as_secs_f32();
    dbg!(progress);
    camera.translation = Vec3::new(-1000.0 * progress, 0.0, 0.0);

    //Planet collision
    //mini_map[0].Node.width = Val::Percent(10.0);

    if cutsceneprogress.timer.finished() {
        finish_cutscene(next_menu);
    }

    //Enemy spawning depending on biome
}
