use bevy::{
    prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef, sprite_render::Material2d,
};

use crate::{
    PausableSystems,
    game::{
        materials::{
            CollectableMaterial, CollectionRadius, Material, MaterialAmount, MaterialRespawn,
            Materials,
        },
        player::Player,
    },
};

const MATERIALS_INVENTORY_SHADER_PATH: &str = "shaders/materials_inventory.wesl";

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_materials_inventory)
        .add_systems(Update, collect_materials.in_set(PausableSystems));
}

#[derive(Component, Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct MaterialInventory {
    #[uniform(0)]
    materials: Materials,
}

impl MaterialInventory {
    pub(super) fn new() -> Self {
        Self {
            materials: Materials::new(),
        }
    }
}

impl Default for MaterialInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl Material2d for MaterialInventory {
    fn fragment_shader() -> ShaderRef {
        MATERIALS_INVENTORY_SHADER_PATH.into()
    }
}

fn setup_materials_inventory(mut commands: Commands) {
    commands.spawn(MaterialInventory::new());
}

fn collect_materials(
    mut commands: Commands,
    mut collectable_materials: Query<
        (
            Entity,
            &Material,
            &Transform,
            &CollectionRadius,
            &MaterialAmount,
            &mut MaterialRespawn,
            &mut Visibility,
        ),
        With<CollectableMaterial>,
    >,
    player: Single<&Transform, With<Player>>,
    mut materials_inventory: Single<&mut MaterialInventory>,
) {
    collectable_materials.iter_mut().for_each(
        |(entity, material, transform, radius, amount, mut respawn, mut visibility)| {
            if transform.translation.distance(player.translation) < **radius {
                respawn.unpause();
                commands.entity(entity).remove::<CollectableMaterial>();
                *visibility = Visibility::Hidden;
                materials_inventory.materials.add(material, amount);
            }
        },
    );
}

// struct PotionInventory {
//     potions: Potions,
// }

// struct Potions {
//     dash: Vec2,
//     feather: Vec2,
//     speed: Vec2,
//     air_jump: Vec2,
//     avarice: Vec2,
// }
