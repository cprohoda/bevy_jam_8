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
        movement::{AirJump, Dash, FeatherWeight},
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

#[derive(Component, Clone)]
pub(super) struct PotionsInventory {
    dash: Potions<Dash>,
    featherweight: Potions<FeatherWeight>,
    air_jump: Potions<AirJump>,
}

impl PotionsInventory {
    pub(super) fn new() -> Self {
        Self {
            dash: Potions::new(),
            featherweight: Potions::new(),
            air_jump: Potions::new(),
        }
    }

    pub(super) fn use_dash(&mut self) -> Result<Dash, ()> {
        self.dash.use_potion()
    }

    pub(super) fn use_featherweight(&mut self) -> Result<FeatherWeight, ()> {
        self.featherweight.use_potion()
    }

    pub(super) fn use_air_jump(&mut self) -> Result<AirJump, ()> {
        if let Ok(mut potion) = self.air_jump.use_potion() {
            potion.use_jump();

            Ok(potion)
        } else {
            Err(())
        }
    }

    pub(super) fn enhance_inventory(&mut self) {
        self.dash.increase_capacity(2);
        self.featherweight.increase_capacity(2);
        self.air_jump.increase_capacity(2);
    }
}

impl Default for PotionsInventory {
    fn default() -> Self {
        Self {
            dash: Potions::new(),
            featherweight: Potions::new(),
            air_jump: Potions::new(),
        }
    }
}

#[derive(Component, Clone)]
struct Potions<T: Default> {
    count: u32,
    max: u32,
    potions: Vec<T>,
}

impl<T: Default> Potions<T> {
    fn new() -> Potions<T> {
        let count = 3;

        let potions = (0..count).map(|_| T::default()).collect::<Vec<T>>();

        Potions {
            count: count,
            max: count,
            potions,
        }
    }

    fn increase_capacity(&mut self, capacity: u32) {
        self.max += capacity;
    }

    fn use_potion(&mut self) -> Result<T, ()> {
        if let Some(potion) = self.potions.pop() {
            self.count -= 1;
            Ok(potion)
        } else {
            Err(())
        }
    }
}
