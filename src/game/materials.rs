use std::time::Duration;

use bevy::{prelude::*, render::render_resource::ShaderType};

use crate::{PausableSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_materials)
        .add_systems(OnEnter(Screen::Gameplay), spawn_materials)
        .add_systems(Update, respawn_materials.in_set(PausableSystems));
}

#[derive(Clone, Component, PartialEq, Eq, Debug)]
pub(crate) enum Material {
    Cerulean,
    Lavender,
    Verdant,
}

#[derive(Component, Deref, Default, Clone)]
pub(crate) struct MaterialAmount(f32);

#[derive(ShaderType, Debug, Clone)]
pub(crate) struct MaterialCapacity {
    current: f32,
    max: f32,
}

#[derive(ShaderType, Debug, Clone)]
pub(crate) struct Materials {
    cerulean: MaterialCapacity,
    lavender: MaterialCapacity,
    verdant: MaterialCapacity,
    gold: f32,
}

impl Materials {
    pub(crate) fn new() -> Self {
        Self {
            cerulean: MaterialCapacity {
                current: 0.0,
                max: 10.0,
            },
            lavender: MaterialCapacity {
                current: 0.0,
                max: 10.0,
            },
            verdant: MaterialCapacity {
                current: 0.0,
                max: 10.0,
            },
            gold: 0.0,
        }
    }

    pub(crate) fn get_material(&self, material: &Material) -> &MaterialCapacity {
        match material {
            Material::Cerulean => &self.cerulean,
            Material::Lavender => &self.lavender,
            Material::Verdant => &self.verdant,
        }
    }

    pub(super) fn get_material_mut(&mut self, material: &Material) -> &mut MaterialCapacity {
        match material {
            Material::Cerulean => &mut self.cerulean,
            Material::Lavender => &mut self.lavender,
            Material::Verdant => &mut self.verdant,
        }
    }

    pub(crate) fn current_amount(&self, material: &Material) -> f32 {
        self.get_material(material).current
    }

    pub(crate) fn current_percent(&self, material: &Material) -> f32 {
        let item = self.get_material(material);

        item.current * 100.0 / item.max
    }

    pub(crate) fn max(&self, material: &Material) -> f32 {
        self.get_material(material).max
    }

    pub(super) fn add(&mut self, material: &Material, amount: &MaterialAmount) {
        let item = self.get_material_mut(material);

        item.current = f32::max(item.max, item.current + **amount);
    }

    pub(super) fn consume(&mut self, material: &Material, amount: &MaterialAmount) -> Result<(), ()> {
        let item = self.get_material_mut(material);

        if **amount > item.current {
            Err(())
        } else {
            item.current -= **amount;
            Ok(())
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
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub(super) struct CollectableMaterial;

#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub(super) struct MaterialRespawn(Timer);

impl Default for MaterialRespawn {
    fn default() -> Self {
        let mut timer = Timer::new(Duration::from_secs(10), TimerMode::Once);
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

fn spawn_materials(mut commands: Commands, material_assets: Res<MaterialAssets>) {
    commands.spawn_scene_list(materials_scene(&material_assets));
}

fn materials_scene(material_assets: &MaterialAssets) -> impl SceneList {
    bsn_list! {
        @spawn_material(Material::Cerulean, Vec3::new(0.0, 100.0, 0.0), material_assets)
        --
        @spawn_material(Material::Lavender, Vec3::new(100.0, 50.0, 0.0), material_assets)
        --
        @spawn_material(Material::Verdant, Vec3::new(-100.0, 50.0, 0.0), material_assets)
    }
}

fn spawn_material(
    material: Material,
    translation: Vec3,
    material_assets: &MaterialAssets,
) -> impl Scene {
    let (material_handle, mesh_handle) = material_assets.handles(&material);

    bsn! {
        material
        Mesh2d(mesh_handle)
        MeshMaterial2d<ColorMaterial>(material_handle)
        Transform::from_translation(translation).with_scale(Vec2::splat(30.0).extend(1.0))
        MaterialAmount(10.0)
        CollectableMaterial
        MaterialRespawn
        Visibility::Visible
        CollectionRadius
        DespawnOnExit<Screen>(Screen::Gameplay)
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
    respawnable_materials.iter_mut().for_each(|(entity, mut respawn, mut visibility)| {
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
