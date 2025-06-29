use std::time::Duration;

use bevy::prelude::*;

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_animations
            .in_set(PausableSystems)
            .in_set(AppSystems::TickTimers)
            .in_set(AppSystems::Update),
    );
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct AnimatedSprite {
    timer: Timer,
    frame: usize,
    total_frames: usize,
    animation_type: AnimationType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationType {
    Once,
    Repeating,
}

impl AnimatedSprite {
    pub fn new(ms_per_frame: usize, frames: usize, animation_type: AnimationType) -> Self {
        Self {
            timer: Timer::new(
                Duration::from_millis(ms_per_frame as u64),
                TimerMode::Repeating,
            ),
            frame: 0,
            total_frames: frames,
            animation_type,
        }
    }

    pub fn update_timer(&mut self, delta: Duration) {
        self.timer.tick(delta);
        if !self.timer.finished() {
            return;
        }
    }
}

pub fn update_animations(
    mut commands: Commands,
    animations: Query<(Entity, &mut Sprite, &mut AnimatedSprite)>,
    time: Res<Time>,
) {
    for (ent, mut sprite, mut animation) in animations {
        animation.update_timer(time.delta());
        animation.frame += 1;
        if animation.animation_type == AnimationType::Once
            && animation.frame >= animation.total_frames
        {
            commands.get_entity(ent).unwrap().despawn();
            return;
        }
        animation.frame %= animation.total_frames;
        let Some(atlas) = sprite.texture_atlas.as_mut() else {
            continue;
        };
        atlas.index = animation.frame;
    }
}
