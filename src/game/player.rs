//! Player-specific behavior.

use avian2d::{dynamics::rigid_body::mass_properties::components::Mass, prelude::{CoefficientCombine, Collider, Friction, LockedAxes, RigidBody, ShapeCaster}};
use bevy::{
    ecs::template::OptionTemplate, image::{ImageLoaderSettings, ImageSampler, TextureAtlasTemplate}, prelude::*, sprite::Anchor,
};

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    game::{
        animation::PlayerAnimation,
        movement::MovementController,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        record_player_directional_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(
    position: Vec2,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Scene {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    bsn! {
        #Player
        @{ bsn!{ player_animation } }
        Player
        Sprite {
            image: { player_assets.ducky.clone() },
            texture_atlas: OptionTemplate::Some(TextureAtlasTemplate {
                layout: texture_atlas_layout,
                index: { player_animation.get_atlas_index() },
            }),
        }
        Anchor(Vec2::new(0.0, -3.0 / 32.0))
        Transform::from_translation(position.extend(10.0))
        MovementController

        RigidBody::Dynamic
        Mass(50.0)
        Collider::round_rectangle(12.0, 18.0, 4.0)
        LockedAxes::ROTATION_LOCKED
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min)
        ShapeCaster::new(Collider::rectangle(18.0, 2.0), Vec2::ZERO, 0.0, Dir2::NEG_Y)
            .with_max_distance(15.0)

    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub(super) struct Player;

fn record_player_directional_input(
    input: Res<ButtonInput<KeyCode>>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Collect directional input.
    let mut intent = Vec2::ZERO;
    // if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
    //     intent.y += 1.0;
    // }
    // if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
    //     intent.y -= 1.0;
    // }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        intent.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        intent.x += 1.0;
    }




    // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
    // This should be omitted if the input comes from an analog stick instead.
    let intent = intent.normalize_or_zero();

    let jump = input.just_pressed(KeyCode::Space);
    for mut controller in &mut controller_query {
        controller.intent = intent;
        if jump {
            controller.jump_requested = true;
        }
    }


}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                })
                .load("images/ducky.png"),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
