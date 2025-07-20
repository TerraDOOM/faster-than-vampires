//! The pause menu.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(
        Update,
        go_back.run_if(
            (in_state(Menu::Pause).or(in_state(Menu::Freecam)))
                .and(input_just_pressed(KeyCode::Escape)),
        ),
    );

    app.add_systems(Update, move_freecam.run_if(in_state(Menu::Freecam)));
}

fn spawn_pause_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Pause Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Pause),
        children![
            widget::header("Game paused"),
            widget::button("Continue", close_menu),
            widget::button("Freecam", set_freecam),
            widget::button("Settings", open_settings_menu),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

struct Freecam;

fn set_freecam(
    _: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut next_menu: ResMut<NextState<Menu>>,
    overlay: Query<(Entity, &Name)>,
) {
    next_menu.set(Menu::Freecam);
    let Some((overlay, _)) = overlay.iter().find(|(_, name)| &***name == "Pause Overlay") else {
        return;
    };

    commands.entity(overlay).despawn();
}

fn move_freecam(
    mut camera: Single<&mut Transform, With<Camera2d>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let mut intent = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        intent.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        intent.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }

    intent *= 20.0;

    camera.translation += intent.extend(0.0);
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
