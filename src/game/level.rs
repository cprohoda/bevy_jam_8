//! Spawn the main level.

use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::player::{PlayerAssets, player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<LevelAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn_scene(bsn! {
        #Level
        Transform
        Visibility
        DespawnOnExit<Screen>(Screen::Gameplay)
        Children [
            @player(400.0, &player_assets, &mut texture_atlas_layouts)
            --
            @platform(Vec2::new(0.0, -300.0), Vec2::new(1200.0, 40.0))
            --
            @platform(Vec2::new(200.0, 0.0), Vec2::new(400.0, 400.0))
            --
            #GameplayMusic
            @music(level_assets.music.clone())
        ]
    });
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Platform;
pub fn platform(position: Vec2, size: Vec2) -> impl Scene {
    bsn! {
        Platform
        Transform::from_translation(position.extend(0.0))
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.4),
            custom_size: size
        }
        Sprite
        RigidBody::Static
        Collider::rectangle(size.x, size.y)   // полные размеры, не половины
    }
}
