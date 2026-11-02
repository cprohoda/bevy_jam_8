use std::collections::{HashMap, HashSet};

use avian2d::prelude::{Collider, RigidBody};
use bevy::{prelude::*, transform::helper::TransformHelper};
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};

use crate::{
    asset_tracking::LoadResource,
    audio::music,
    game::player::{Player, PlayerAssets, player},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin)
        // Start in the first room; its neighbours are loaded with it.
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LdtkSettings {
            // Draw the level background colour from LDtk.
            level_background: LevelBackground::Rendered,
            // Place rooms at their LDtk world positions, and keep neighbours loaded.
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            // We draw IntGrid tiles ourselves in `spawn_level_tiles`, so don't let
            // bevy_ecs_ldtk draw them too.
            int_grid_rendering: IntGridRendering::Invisible,
            ..default()
        })
        .init_resource::<CurrentRoom>();

    app.load_resource::<LevelAssets>();
    app.add_systems(
        Update,
        (
            spawn_player_at_spawn_point,
            spawn_level_tiles,
            track_current_room,
        )
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
    #[dependency]
    ldtk_project: Handle<LdtkProject>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
            ldtk_project: assets.load("levels/world.ldtk"),
        }
    }
}

/// The room the player is in, as a world-space rectangle. Empty until the player has spawned.
#[derive(Resource, Default, Clone, Copy)]
pub(super) struct CurrentRoom(pub(super) Rect);

/// Root entity of the gameplay level; everything level-related is its descendant.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct LevelRoot;

/// A system that spawns the main level.
pub fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
    let ldtk_project = level_assets.ldtk_project.clone();
    commands.spawn_scene(bsn! {
        #Level
        LevelRoot
        Transform
        Visibility
        DespawnOnExit<Screen>(Screen::Gameplay)
        Children [
            #LdtkWorld
            LdtkProjectHandle { handle: { ldtk_project.clone() } }
            --
            #GameplayMusic
            @music(level_assets.music.clone())
        ]
    });
}

/// When LDtk spawns the `Player` entity, spawn the real player at its position.
fn spawn_player_at_spawn_point(
    mut commands: Commands,
    spawn_points: Query<(Entity, &EntityInstance), Added<EntityInstance>>,
    level_root: Single<Entity, With<LevelRoot>>,
    transform_helper: TransformHelper,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (entity, instance) in &spawn_points {
        if instance.identifier != "Player" {
            continue;
        }
        let Ok(global) = transform_helper.compute_global_transform(entity) else {
            continue;
        };
        let position = global.translation().truncate();
        let player_entity = commands
            .spawn_scene(player(position,  &player_assets, &mut texture_atlas_layouts))
            .id();
        // Parent it to the level root so it despawns with the level,
        // but not to the LDtk hierarchy (which can be respawned on hot-reload).
        commands.entity(*level_root).add_child(player_entity);
        // The spawn point itself is just a marker.
        commands.entity(entity).despawn();
    }
}

/// What an IntGrid value in the `Collisions` layer looks like and how it behaves.
///
/// Placeholder visuals: flat colours. To use textures later, change [`Tile::color`] into
/// a texture handle and build the sprite from it in [`spawn_level_tiles`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Tile {
    Dirt,
    Ladder,
    Stone,
}

impl Tile {
    fn from_value(value: i32) -> Option<Self> {
        match value {
            1 => Some(Self::Dirt),
            2 => Some(Self::Ladder),
            3 => Some(Self::Stone),
            _ => None,
        }
    }

    fn color(self) -> Color {
        match self {
            Self::Dirt => Color::srgb(0.45, 0.30, 0.20),
            Self::Ladder => Color::srgb(0.80, 0.70, 0.30),
            Self::Stone => Color::srgb(0.40, 0.42, 0.48),
        }
    }

    /// Ladders are not walkable, so they get no collider.
    fn solid(self) -> bool {
        !matches!(self, Self::Ladder)
    }
}

/// Turn new `Collisions` IntGrid cells into sprites, and solid ones into static colliders.
///
/// Neighbouring cells of the same kind are merged into as few rectangles as possible, so the
/// player doesn't snag on the seams between tiles.
fn spawn_level_tiles(
    mut commands: Commands,
    cells: Query<(&GridCoords, &IntGridCell, &ChildOf), Added<IntGridCell>>,
    layers: Query<&LayerMetadata>,
) {
    // Group new cells by the layer they belong to and by tile kind.
    let mut by_layer: HashMap<(Entity, Tile), HashSet<GridCoords>> = HashMap::new();
    for (coords, cell, child_of) in &cells {
        if let Some(tile) = Tile::from_value(cell.value) {
            by_layer
                .entry((child_of.parent(), tile))
                .or_default()
                .insert(*coords);
        }
    }

    for ((layer_entity, tile), coords) in by_layer {
        let Ok(layer) = layers.get(layer_entity) else {
            continue;
        };
        let grid = layer.grid_size as f32;
        commands.entity(layer_entity).with_children(|parent| {
            for rect in merge_into_rects(&coords, layer.c_wid, layer.c_hei) {
                let size = rect.size().as_vec2() * grid;
                // The layer entity is the tilemap, and bevy_ecs_ldtk offsets it by half a
                // tile (tiles are anchored at their centre). Cancel that out here, or the
                // colliders end up half a tile right and up.
                let center = rect.min.as_vec2() * grid + size / 2.0 - Vec2::splat(grid / 2.0);
                let mut tile_entity = parent.spawn((
                    Name::new(format!("{tile:?} tile")),
                    Transform::from_translation(center.extend(0.0)),
                    Sprite::from_color(tile.color(), size),
                ));
                if tile.solid() {
                    tile_entity.insert((RigidBody::Static, Collider::rectangle(size.x, size.y)));
                }
            }
        });
    }
}

/// Select the room the player is standing in. Selecting a room also loads its neighbours
/// (see `load_level_neighbors`), so walking across an edge seamlessly reaches the next room.
fn track_current_room(
    player: Query<&Transform, With<Player>>,
    ldtk_projects: Res<Assets<LdtkProject>>,
    level_assets: Res<LevelAssets>,
    mut selection: ResMut<LevelSelection>,
    mut current: ResMut<CurrentRoom>,
) {
    let Some(project) = ldtk_projects.get(&level_assets.ldtk_project) else {
        return;
    };
    let Ok(player) = player.single() else {
        return;
    };
    let position = player.translation.truncate();

    for (index, level) in project.json_data().levels.iter().enumerate() {
        let rect = room_rect(level);
        if rect.contains(position) {
            current.0 = rect;
            let wanted = LevelSelection::index(index);
            if *selection != wanted {
                *selection = wanted;
            }
            return;
        }
    }
}

/// World-space rectangle of an LDtk level. LDtk's y axis points down, bevy's points up.
fn room_rect(level: &Level) -> Rect {
    let min = Vec2::new(level.world_x as f32, -(level.world_y + level.px_hei) as f32);
    Rect::from_corners(min, min + Vec2::new(level.px_wid as f32, level.px_hei as f32))
}

/// Greedy meshing: merge horizontal runs of cells, then stack identical runs vertically.
/// Returned rects are in grid cells, `max` exclusive.
fn merge_into_rects(cells: &HashSet<GridCoords>, width: i32, height: i32) -> Vec<IRect> {
    let mut done: Vec<IRect> = Vec::new();
    // Rects still open from the previous row, keyed by (x_start, x_end).
    let mut open: HashMap<(i32, i32), IRect> = HashMap::new();

    for y in 0..height {
        let mut next_open = HashMap::new();
        let mut x = 0;
        while x < width {
            if !cells.contains(&GridCoords::new(x, y)) {
                x += 1;
                continue;
            }
            let start = x;
            while x < width && cells.contains(&GridCoords::new(x, y)) {
                x += 1;
            }
            let rect = match open.remove(&(start, x)) {
                Some(mut rect) => {
                    rect.max.y = y + 1;
                    rect
                }
                None => IRect::new(start, y, x, y + 1),
            };
            next_open.insert((start, x), rect);
        }
        done.extend(open.into_values());
        open = next_open;
    }
    done.extend(open.into_values());
    done
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The level file must stay parseable by `bevy_ecs_ldtk`.
    #[test]
    fn world_ldtk_parses() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/levels/world.ldtk"
        ))
        .unwrap();
        let project: bevy_ecs_ldtk::ldtk::LdtkJson = serde_json::from_str(&text).unwrap();
        assert!(!project.levels.is_empty());
    }

    #[test]
    fn merges_rects() {
        // ##.
        // ##.
        // ###
        let cells: HashSet<_> = [(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (0, 2), (1, 2)]
            .into_iter()
            .map(|(x, y)| GridCoords::new(x, y))
            .collect();
        let rects = merge_into_rects(&cells, 3, 3);
        let area: i32 = rects.iter().map(|r| r.width() * r.height()).sum();
        assert_eq!(area, 7);
        assert_eq!(rects.len(), 2);
    }
}
