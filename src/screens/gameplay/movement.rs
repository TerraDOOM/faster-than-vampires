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
use std::f32::consts::PI;

use crate::{AppSystems, PausableSystems};

use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MovementController>();
    app.register_type::<ScreenWrap>();

    //    app.add_systems(Update, update_camera);

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
        &mut MovementController,
        &Transform,
        &mut ExternalImpulse,
        &mut ExternalTorque,
    )>,
) {
    const ROTATION_SPEED: f32 = 30.0 * 100.0;
    const THRUST: f32 = 10.0;
    for (mut controller, transform, mut linvel, mut angvel) in &mut movement_query {
        let rotation = -controller.intent.x * ROTATION_SPEED * time.delta_secs();

        angvel.apply_torque(rotation);
        linvel.apply_impulse(
            (transform.rotation * (controller.intent.y * THRUST * Vec2::Y).extend(0.0)).xy(),
        );

        // dbg!(&angvel);
        // dbg!(&linvel);
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

// fn update_camera(
//     mut camera: Single<&mut Transform, With<Camera2d>>,
//     player: Single<&Transform, With<Player>>,
//     time: Res<Time>,
// ) {
//     //let mut camera = Some(camera) else {
//     //     return;
//     // };

//     // let player = player else {
//     //     return;
//     // };

//     let Vec3 { x, y, .. } = player.translation;
//     let direction = Vec3::new(x, y, camera.translation.z);

//     // Applies a smooth effect to camera movement using stable interpolation
//     // between the camera position and the player position on the x and y axes.
//     camera.translation = direction;
//}
