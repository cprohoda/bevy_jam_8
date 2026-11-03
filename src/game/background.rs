use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    sprite::SpriteImageMode,
};

use crate::{PausableSystems, asset_tracking::LoadResource, game::movement, screens::Screen};

/// How much the background follows the camera: `1.0` is glued to the camera (infinitely far
/// away), `0.0` is glued to the level. Smaller values look closer.
const PARALLAX: f32 = 0.8;
/// Far behind the level, which LDtk spawns at z >= 0.
const BACKGROUND_Z: f32 = -100.0;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<BackgroundAssets>();
    app.add_systems(OnEnter(Screen::Gameplay), spawn_background);
    app.add_systems(
        PostUpdate,
        update_background
            .after(movement::camera_follow_player)
            .before(TransformSystems::Propagate)
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct BackgroundAssets {
    #[dependency]
    image: Handle<Image>,
}

impl FromWorld for BackgroundAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            image: assets
                .load_builder()
                .with_settings(|settings: &mut ImageLoaderSettings| {
                    // Keep the pixel art crisp.
                    settings.sampler = ImageSampler::nearest();
                })
                .load("images/back.png"),
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Background;

fn spawn_background(mut commands: Commands, assets: Res<BackgroundAssets>) {
    commands.spawn((
        Name::new("Background"),
        Background,
        Sprite {
            image: assets.image.clone(),
            // Real size and tiling are set every frame in `update_background`.
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, BACKGROUND_Z),
        DespawnOnExit(Screen::Gameplay),
    ));
}

/// Stretch the background to the camera's height, cover its width with whole tiles, and shift
/// the tiles so the pattern scrolls at `1 - PARALLAX` of the camera speed.
fn update_background(
    camera: Single<(&Transform, &Projection), (With<Camera2d>, Without<Background>)>,
    mut background: Single<(&mut Transform, &mut Sprite), With<Background>>,
    images: Res<Assets<Image>>,
) {
    let (camera_transform, projection) = *camera;
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    let (transform, sprite) = &mut *background;
    let Some(image) = images.get(&sprite.image) else {
        return;
    };

    let view = ortho.area.size();
    if view.x <= 0.0 || view.y <= 0.0 {
        return;
    }
    let image_size = image.size_f32();
    let scale = view.y / image_size.y;
    let tile_width = image_size.x * scale;

    // A whole number of tiles, one more than needed, so there is room to shift the pattern.
    let tiles = (view.x / tile_width).ceil() + 1.0;
    let width = tiles * tile_width;

    let camera_pos = camera_transform.translation.truncate();
    // Where the pattern starts in the world: it moves with the camera at `PARALLAX` speed.
    let pattern_origin = camera_pos.x * PARALLAX;
    let view_left = camera_pos.x - view.x / 2.0;
    let left = view_left - (view_left - pattern_origin).rem_euclid(tile_width);

    transform.translation.x = left + width / 2.0;
    transform.translation.y = camera_pos.y;

    let size = Vec2::new(width, view.y);
    if sprite.custom_size != Some(size) {
        sprite.custom_size = Some(size);
        sprite.image_mode = SpriteImageMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: scale,
        };
    }
}
