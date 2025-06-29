//! Handle player input and translate it into movement through a character
//! controller. A character controller is the collection of systems that govern
//! the movement of characters.
//!
//! In our case, the character controller has the following logic:
//! - Set [`MovementController`] intent based on directional keyboard input.
//!   This is done in the `player` module, as it is specific to the player
//!   character.
//! - Apply movement based on [`MovementController`] intent and maximum speed.
//! - Wrap the character within the window.
//!
//! Note that the implementation used here is limited for demonstration
//! purposes. If you want to move the player in a smoother way,
//! consider using a [fixed timestep](https://github.com/bevyengine/bevy/blob/main/examples/movement/physics_in_fixed_timestep.rs).

use avian2d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    screens::{gameplay::upgrade_menu::UpgradeTypes, Screen},
    AppSystems, PausableSystems,
};

use super::{
    level::{BackgroundAccess, Planet},
    player::Player,
    upgrade_menu::{UIShop, Upgrades},
};

//use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.register_type::<ScreenWrap>();

    app.add_systems(Update, update_camera.run_if(in_state(Screen::Gameplay)));

    app.add_systems(
        FixedUpdate,
        (apply_movement, apply_screen_wrap)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovementController {
    /// The direction the character wants to move in.
    pub intent: Vec2,

    /// Maximum speed in world units per second.
    /// 1 world unit = 1 pixel when using the default 2D camera and no physics engine.
    pub max_speed: f32,
    pub angle: f32,
}

impl Default for MovementController {
    fn default() -> Self {
        Self {
            intent: Vec2::ZERO,
            // 400 pixels per second is a nice default, but we can still vary this per character.
            max_speed: 400.0,
            angle: 0.0,
        }
    }
}

fn apply_movement(
    time: Res<Time>,
    mut movement_query: Query<(
        &MovementController,
        &Transform,
        &mut ExternalImpulse,
        &mut ExternalTorque,
    )>,
    upgrades: Single<&Upgrades>,
) {
    const ROTATION_SPEED: f32 = 30.0 * 100.0;

    let thrust;
    if let Some(engine_lvl) = upgrades.gotten_upgrades.get(&UpgradeTypes::Thrusters) {
        thrust = (*engine_lvl as f32) * 10.0;
    } else {
        thrust = 10.0;
    }
    for (controller, transform, mut linvel, mut angvel) in &mut movement_query {
        let rotation = -controller.intent.x * ROTATION_SPEED * time.delta_secs();

        angvel.apply_torque(rotation);
        linvel.apply_impulse(
            (transform.rotation * (controller.intent.y * thrust * Vec2::Y).extend(0.0)).xy(),
        );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ScreenWrap;

fn apply_screen_wrap(
    window: Single<&Window, With<PrimaryWindow>>,
    mut wrap_query: Query<&mut Transform, With<ScreenWrap>>,
) {
    let size = window.size() + 256.0;
    let half_size = size / 2.0;
    for mut transform in &mut wrap_query {
        let position = transform.translation.xy();
        let wrapped = (position + half_size).rem_euclid(size) - half_size;
        transform.translation = wrapped.extend(transform.translation.z);
    }
}

fn update_camera(
    mut set: ParamSet<(
        Query<&mut Transform, With<Camera2d>>,
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<BackgroundAccess>>,
        Query<(&mut Transform, &Planet)>,
    )>,
) {
    let p = {
        if let player = set.p1().single_inner().unwrap() {
            player.translation.xy()
        } else {
            return;
        }
    };
    let camera_layer = {
        let mut camera = set.p0().single_inner().unwrap();
        camera.translation.x = p.x;
        camera.translation.y = p.y;
        camera.translation.z
    };

    let camera_pos = Vec3::new(p.x, p.y, camera_layer);
    {
        let mut p2 = set.p2();
        let mut background = p2.single_mut().unwrap();
        background.translation = camera_pos * 0.95 - Vec3::new(0.0, 0.0, 5.0);
    }

    //Planet paralaxing
    for (mut transform, init_position) in set.p3().iter_mut() {
        transform.translation =
            Vec3::new(init_position.x, init_position.y, -0.5) + camera_pos * 0.9;
    }
}
