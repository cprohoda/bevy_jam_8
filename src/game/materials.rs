use std::time::Duration;

use bevy::{prelude::*, render::render_resource::ShaderType};
use bevy_ecs_ldtk::prelude::EntityInstance;

use crate::{
    PausableSystems,
    game::inventory::InventoryItem,
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_materials)
        .add_systems(Update, spawn_materials.run_if(in_state(Screen::Gameplay)))
        .add_systems(Update, respawn_materials.in_set(PausableSystems));
}

#[derive(Clone, Component, PartialEq, Eq, Debug)]
pub(crate) enum Material {
    Cerulean,
    Lavender,
    Verdant,
    Aether,
    Gold,
}

#[derive(Component, Deref, Default, Clone)]
pub(crate) struct MaterialAmount(f32);

#[derive(ShaderType, Debug, Clone)]
pub(crate) struct Materials {
    cerulean: f32, // would use hashmap but that's painful for shaders
    lavender: f32,
    verdant: f32,
    aether: f32,
    gold: f32,
}

impl Materials {
    pub(crate) fn new(cerulean: f32, lavender: f32, verdant: f32, aether: f32, gold: f32) -> Self {
        Self {
            cerulean,
            lavender,
            verdant,
            aether,
            gold,
        }
    }

    pub(super) fn add_material(&mut self, material: &Material, amount: f32) {
        match material {
            Material::Cerulean => {
                self.cerulean += amount;
            }
            Material::Lavender => {
                self.lavender += amount;
            }
            Material::Verdant => {
                self.verdant += amount;
            }
            Material::Aether => {
                self.aether += amount;
            }
            Material::Gold => {
                self.gold += amount;
            }
        }
    }

    pub(super) fn consume(&mut self, recipe: Recipe) -> Result<InventoryItem, ()> {
        if self.aether >= recipe.ingredients.aether
            && self.cerulean >= recipe.ingredients.cerulean
            && self.gold >= recipe.ingredients.gold
            && self.lavender >= recipe.ingredients.lavender
            && self.verdant >= recipe.ingredients.verdant
        {
            self.aether -= recipe.ingredients.aether;
            self.cerulean -= recipe.ingredients.cerulean;
            self.gold -=  recipe.ingredients.gold;
            self.lavender -= recipe.ingredients.lavender;
            self.verdant -= recipe.ingredients.verdant;
            Ok(recipe.output)
        } else {
            Err(())
        }
    }
}

#[derive(Component, Clone, Debug)]
pub(super) struct Recipe {
    output: InventoryItem,
    ingredients: Materials,
}

impl Recipe {
    pub(super) fn new(output: InventoryItem, ingredients: Materials) -> Self {
        Self {
            output,
            ingredients,
        }
    } 
}

#[derive(Resource)]
pub(super) struct MaterialAssets {
    verdant: (Handle<ColorMaterial>, Handle<Mesh>),
    lavender: (Handle<ColorMaterial>, Handle<Mesh>),
    cerulean: (Handle<ColorMaterial>, Handle<Mesh>),
}

impl MaterialAssets {
    pub(crate) fn handles(&self, material: &Material) -> (Handle<ColorMaterial>, Handle<Mesh>) {
        match material {
            Material::Cerulean => self.cerulean.clone(),
            Material::Lavender => self.lavender.clone(),
            Material::Verdant => self.verdant.clone(),
            _ => self.cerulean.clone(), // default to cerulean for now
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub(super) struct CollectableMaterial;

#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub(super) struct MaterialRespawn(Timer);

impl MaterialRespawn {
    fn from_duration(duration: Duration) -> Self {
        let mut timer = Timer::new(duration, TimerMode::Once);
        timer.pause();
        Self(timer)
    }
}

#[derive(Component, Debug, Clone, Deref)]
pub(super) struct CollectionRadius(f32);

impl Default for CollectionRadius {
    fn default() -> Self {
        CollectionRadius(100.0)
    }
}

fn setup_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let triangle_mesh = meshes.add(Mesh::from(Triangle2d::new(
        Vec2::new(0.0, 0.57734),
        Vec2::new(-0.5, -0.28865),
        Vec2::new(0.5, -0.28865),
    )));
    let square_mesh = meshes.add(Mesh::from(Rectangle::from_corners(
        Vec2::new(-0.5, -0.5),
        Vec2::new(0.5, 0.5),
    )));
    let circle_mesh = meshes.add(Mesh::from(Circle::new(0.5)));

    let verdant_material = materials.add(Color::LinearRgba(LinearRgba {
        red: 0.44,
        green: 0.78,
        blue: 0.66,
        alpha: 1.0,
    }));
    let lavender_material = materials.add(Color::LinearRgba(LinearRgba {
        red: 0.51,
        green: 0.30,
        blue: 0.73,
        alpha: 1.0,
    }));
    let cerulean_material = materials.add(Color::LinearRgba(LinearRgba {
        red: 0.11,
        green: 0.67,
        blue: 0.84,
        alpha: 1.0,
    }));

    let material_assets = MaterialAssets {
        verdant: (verdant_material, square_mesh),
        lavender: (lavender_material, triangle_mesh),
        cerulean: (cerulean_material, circle_mesh),
    };

    commands.insert_resource(material_assets);
}

/// When LDtk spawns a `Cerulean` / `Lavender` / `Verdant` entity, turn it into a collectable material.
fn spawn_materials(
    mut commands: Commands,
    mut spawn_points: Query<(Entity, &EntityInstance, &mut Transform), Added<EntityInstance>>,
    material_assets: Res<MaterialAssets>,
) {
    for (entity, instance, mut transform) in &mut spawn_points {
        let material = match instance.identifier.as_str() {
            "Cerulean" => Material::Cerulean,
            "Lavender" => Material::Lavender,
            "Verdant" => Material::Verdant,
            _ => continue,
        };
        let (material_handle, mesh_handle) = material_assets.handles(&material);
        transform.scale = Vec2::splat(30.0).extend(1.0);
        commands.entity(entity).insert((
            material,
            Mesh2d(mesh_handle),
            MeshMaterial2d(material_handle),
            MaterialAmount(10.0),
            CollectableMaterial,
            MaterialRespawn::from_duration(Duration::from_secs(3)),
            Visibility::Visible,
            CollectionRadius::default(),
        ));
    }
}

fn respawn_materials(
    mut commands: Commands,
    mut respawnable_materials: Query<
        (Entity, &mut MaterialRespawn, &mut Visibility),
        (With<Material>, Without<CollectableMaterial>),
    >,
    time: Res<Time>,
) {
    respawnable_materials
        .iter_mut()
        .for_each(|(entity, mut respawn, mut visibility)| {
            if respawn.is_finished() {
                respawn.reset();
                respawn.pause();
                commands.entity(entity).insert(CollectableMaterial);
                *visibility = Visibility::Visible;
            }

            if !respawn.is_paused() {
                respawn.tick(time.delta());
            }
        });
}
