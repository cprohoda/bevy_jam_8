//! Development tools for the game. This plugin is only enabled in dev builds.

use avian2d::prelude::{PhysicsDebugPlugin, PhysicsGizmos};
use bevy::{
    dev_tools::states::log_transitions,
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

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

    // Inspect and edit entities and components at runtime.
    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::new().run_if(input_toggle_active(false, INSPECTOR_KEY)),
    ));
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;
const INSPECTOR_KEY: KeyCode = KeyCode::F1;

fn toggle_debug_ui(mut options: ResMut<GlobalUiDebugOptions>) {
    options.toggle();
}


fn toggle_physics_debug(
    mut store: ResMut<GizmoConfigStore>,
) {
    let (config, _) = store.config_mut::<PhysicsGizmos>();
    config.enabled = !config.enabled;
}