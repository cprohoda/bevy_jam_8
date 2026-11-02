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
    AppSystems, PausableSystems,
    game::{level::CurrentRoom, player::Player},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (apply_movement, apply_screen_wrap, apply_jump)
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    )
    .add_systems(PostUpdate, camera_follow_player.in_set(PausableSystems));
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
        Self { intent: Vec2::ZERO, max_speed: 100.0, jump_speed: 230.0, jump_requested: false }
    }
}

fn apply_movement(mut movement_query: Query<(&MovementController, &mut LinearVelocity)>) {
    for (controller, mut velocity) in &mut movement_query {
        velocity.0.x = controller.max_speed * controller.intent.x;
    }
}

fn apply_jump(
    mut movement_query: Query<(&mut MovementController, &mut LinearVelocity, &ShapeHits)>,
) {
    for (mut controller, mut velocity, hits) in &mut movement_query {
        if controller.jump_requested && !hits.is_empty() {
            velocity.0.y = controller.jump_speed;
        }
        controller.jump_requested = false;
    }
}


fn camera_follow_player(
    mut camera: Single<(&mut Transform, &Projection), (With<Camera2d>, Without<Player>)>,
    player_transform: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    room: Res<CurrentRoom>,
) {
    let (camera_transform, projection) = &mut *camera;
    let target = player_transform.translation;
    // Half the size of what the camera currently sees, in world units.
    let half_view = match projection {
        Projection::Orthographic(ortho) => ortho.area.size() / 2.0,
        _ => Vec2::ZERO,
    };
    let clamped = clamp_to_room(target.truncate(), room.0, half_view);
    camera_transform.translation = clamped.extend(target.z);
}

fn clamp_to_room(target: Vec2, room: Rect, half_view: Vec2) -> Vec2 {
    if room.is_empty() || half_view == Vec2::ZERO {
        return target;
    }
    let axis = |t: f32, min: f32, max: f32, half: f32| {
        if max - min <= half * 2.0 {
            (min + max) / 2.0
        } else {
            t.clamp(min + half, max - half)
        }
    };
    Vec2::new(
        axis(target.x, room.min.x, room.max.x, half_view.x),
        axis(target.y, room.min.y, room.max.y, half_view.y),
    )
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
