use avian2d::prelude::*;
use bevy::prelude::*;

pub mod weapons;

use crate::{screens::Screen, PausableSystems};

use super::{
    enemies::AsteroidAI,
    player::{Player, PlayerAssets},
    GameplayLogic,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(weapons::plugin);

    app.add_systems(
        Update,
        (
            process_asteroid_collisions,
            process_player_dead,
            remove_dead_enemies,
        )
            .in_set(GameplayLogic),
    );

    app.add_observer(damage_trigger);
}

#[derive(Event)]
pub struct Damage(pub i32);

#[derive(Component)]
pub struct Health(pub i32);

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct HasWeapon;

fn damage_trigger(trigger: Trigger<Damage>, mut killable: Query<&mut Health>) {
    let Ok(mut target) = killable.get_mut(trigger.target()) else {
        return;
    };

    target.0 -= trigger.0;
}

fn process_asteroid_collisions(
    mut commands: Commands,
    collisions: Collisions,
    player: Single<Entity, With<Player>>,
    asteroids: Query<&AsteroidAI>,
) {
    for contact_pair in collisions.collisions_with(*player) {
        let total_impulse;

        if contact_pair.collider1 == *player && asteroids.contains(contact_pair.collider2) {
            total_impulse = contact_pair.max_normal_impulse().0
        } else if contact_pair.collider2 == *player && asteroids.contains(contact_pair.collider1) {
            total_impulse = contact_pair.max_normal_impulse().0;
        } else {
            continue;
        }

        let mut damage = total_impulse * 2.0;
        if damage < 10.0 {
            damage = 0.0;
        }
        if damage > 0.0 {
            println!("Emitting damage equal to {}", damage);
        }
        commands.trigger_targets(Damage(damage as i32), *player);
    }
}


fn remove_dead_enemies(enemies: Query<(Entity, &Health), Without<Player>>, mut commands: Commands) {
    for (enemy, health) in enemies {
        if health.0 < 0 {
            commands.get_entity(enemy).unwrap().despawn();
        }
    }
}

fn process_player_dead(
    player: Option<Single<(Entity, &Health, &Transform), (With<Player>, Without<Dead>)>>,
    assets: Res<PlayerAssets>,
    mut commands: Commands,
) {
    let Some(player) = player else {
        return;
    };
    let (ent, health, transform) = player.into_inner();

    if health.0 <= 0 {
        let mut player = commands.get_entity(ent).unwrap();
        player.remove::<Sprite>();
        player.remove::<RigidBody>();
        player.insert(Dead);
        player.despawn_related::<Children>();
        commands.spawn((
            Transform::from_xyz(transform.translation.x, transform.translation.y, 0.0),
            assets.get_explosion(),
        ));

        commands.spawn(crate::audio::sound_effect(assets.crash_sfx.clone()));
    }
}
