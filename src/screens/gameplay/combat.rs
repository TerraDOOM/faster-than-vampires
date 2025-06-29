use bevy::prelude::*;

use super::player::PlayerAssets;

pub(super) fn plugin(app: &mut App) {

}

#[derive(Event)]
pub struct Damage(pub i32);

#[derive(Component)]
pub struct Health(pub i32);

fn process_asteroid_collisions(
    mut commands: Commands,
    collisions: Collisions,
    player: Single<Entity, With<Player>>,
    asteroids: Query<&Ship>,
) {
    for contact_pair in collisions.collisions_with(*player) {
        let total_impulse;
        if contact_pair.collider1 == *player
            && asteroids
                .get(contact_pair.collider2)
                .is_ok_and(|ship| ship.shiptype == ShipType::Asteroid)
        {
            total_impulse = contact_pair.total_normal_impulse_magnitude();
        } else if contact_pair.collider2 == *player
            && asteroids
                .get(contact_pair.collider1)
                .is_ok_and(|ship| ship.shiptype == ShipType::Asteroid)
        {
            total_impulse = contact_pair.total_normal_impulse_magnitude();
        } else {
            continue;
        }

        commands.trigger_targets(Damage(total_impulse as i32), *player);
    }
}

fn process_player_dead(
    player: Option<Single<(&Health, &mut Sprite), With<Player>>>,
    assets: Res<PlayerAssets>,
) {
    let Some(mut player) = player else {
        return;
    };
    if player.0 <= 0 {
        player.1.image = assets.exploded.clone();
    }
}
