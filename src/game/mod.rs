//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

mod animation;
mod background;
mod cauldron;
mod wind;
mod inventory;
pub mod level;
mod materials;
mod movement;
pub mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        background::plugin,
        cauldron::plugin,
        wind::plugin,
        inventory::plugin,
        level::plugin,
        materials::plugin,
        movement::plugin,
        player::plugin,
    ));
}
