//! Development tools for the game. This plugin is only enabled in dev builds.

use avian2d::prelude::{PhysicsDebugPlugin, PhysicsGizmos};
use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        (toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
        toggle_physics_debug.run_if(input_just_pressed(KeyCode::F3)),),
    );

    app.add_plugins(PhysicsDebugPlugin::default());
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<GlobalUiDebugOptions>) {
    options.toggle();
}


fn toggle_physics_debug(
    mut store: ResMut<GizmoConfigStore>,
) {
    let (config, _) = store.config_mut::<PhysicsGizmos>();
    config.enabled = !config.enabled;
}