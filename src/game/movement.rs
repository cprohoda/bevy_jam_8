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

use bevy::{prelude::*, window::PrimaryWindow};
use avian2d::prelude::*;

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (apply_movement, apply_screen_wrap, apply_jump)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// These are the movement parameters for our character controller.
/// For now, this is only used for a single player, but it could power NPCs or
/// other players as well.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct MovementController {
    pub intent: Vec2,
    pub max_speed: f32,
    pub jump_speed: f32,
    pub jump_requested: bool,
}

impl Default for MovementController {
    fn default() -> Self {
        Self { intent: Vec2::ZERO, max_speed: 400.0, jump_speed: 1200.0, jump_requested: false }
    }
}

fn apply_movement(mut movement_query: Query<(&MovementController, &mut LinearVelocity)>) {
    for (controller, mut velocity) in &mut movement_query {
        velocity.0.x = controller.max_speed * controller.intent.x;
    }
}


fn apply_jump(mut movement_query: Query<(&mut MovementController, &mut LinearVelocity, &ShapeHits)>) {
    for (mut controller, mut velocity, hits) in &mut movement_query {
        if controller.jump_requested && !hits.is_empty() {
            velocity.0.y = controller.jump_speed;
        }
        controller.jump_requested = false;
    }
}

#[derive(Component, Reflect, Clone, Default)]
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
