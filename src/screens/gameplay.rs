//! The screen state for the main gameplay.
mod combat;
mod enemies;
mod level;
mod movement;
mod player;
mod upgrade_menu;

mod animation;

use avian2d::{
    prelude::{Gravity, Physics},
    schedule::PhysicsTime,
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*, ui::Val::*};

use crate::{menus::Menu, screens::Screen, PausableSystems, Pause};
use level::spawn_level;

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct GameplayLogic;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        player::plugin,
        movement::plugin,
        enemies::plugin,
        upgrade_menu::plugin,
        combat::plugin,
        animation::plugin,
    ));

    app.add_systems(OnEnter(Screen::Gameplay), spawn_level);

    // Toggle pause on key press.
    app.add_systems(
        Update,
        (
            (pause, spawn_pause_overlay, open_pause_menu).run_if(
                in_state(Screen::Gameplay)
                    .and(in_state(Menu::None))
                    .and(input_just_pressed(KeyCode::KeyP).or(input_just_pressed(KeyCode::Escape))),
            ),
            close_menu.run_if(
                in_state(Screen::Gameplay)
                    .and(not(in_state(Menu::None)))
                    .and(input_just_pressed(KeyCode::KeyP)),
            ),
            (pause, spawn_pause_overlay, open_buy_menu).run_if(
                in_state(Screen::Gameplay)
                    .and(in_state(Menu::None))
                    .and(input_just_pressed(KeyCode::KeyB)),
            ),
        ),
    );
    app.add_systems(OnExit(Screen::Gameplay), (close_menu, unpause));
    app.add_systems(
        OnEnter(Menu::None),
        unpause.run_if(in_state(Screen::Gameplay)),
    );

    app.configure_sets(
        Update,
        GameplayLogic.in_set(PausableSystems).run_if(
            resource_exists::<level::LevelAssets>
                .and(resource_exists::<player::PlayerAssets>)
                .and(resource_exists::<enemies::EntityAssets>)
                .and(any_with_component::<player::Player>)
                .and(any_with_component::<Camera2d>),
        ),
    );

    app.insert_resource(Gravity(Vec2::ZERO));
}

fn unpause(mut next_pause: ResMut<NextState<Pause>>, mut time: ResMut<Time<Physics>>) {
    next_pause.set(Pause(false));
    time.unpause();
}

fn pause(mut next_pause: ResMut<NextState<Pause>>, mut time: ResMut<Time<Physics>>) {
    next_pause.set(Pause(true));
    time.pause();
}

fn spawn_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Overlay"),
        Node {
            width: Percent(100.0),
            height: Percent(100.0),
            ..default()
        },
        GlobalZIndex(1),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        StateScoped(Pause(true)),
    ));
}

fn open_pause_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn open_buy_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Buy);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
