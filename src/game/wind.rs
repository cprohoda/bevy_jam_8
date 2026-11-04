use avian2d::dynamics::rigid_body::{LinearVelocity, mass_properties::components::Mass};
use bevy::{
    ecs::{query::QueryItem, system::lifetimeless::Read},
    prelude::*,
    render::{
        gpu_component_array_buffer::{
            GpuComponentArray, GpuComponentArrayBuffer, GpuComponentArrayBufferPlugin,
        },
        render_resource::{AsBindGroup, ShaderType},
        storage::ShaderBuffer,
    },
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};
use bytemuck::{Pod, Zeroable};

use crate::{PausableSystems, game::player::Player, screens::Screen};

pub(super) const WIND_DIMENSIONS: Vec2 = Vec2::new(50.0, 50.0);
pub(super) const WIND_SHADER_PATH: &str = "shaders/wind.wesl";

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        Material2dPlugin::<WindMaterial>::default(),
        GpuComponentArrayBufferPlugin::<Wind>::default(),
    ))
    .add_systems(Startup, setup_wind)
    .add_systems(Update, spawn_wind.run_if(in_state(Screen::Gameplay)))
    .add_systems(Update, apply_directional_wind.in_set(PausableSystems));
}

#[derive(Resource)]
pub(super) struct WindAssets {
    wind: (Handle<WindMaterial>, Handle<Mesh>),
}

impl WindAssets {
    fn handles(&self) -> (Handle<WindMaterial>, Handle<Mesh>) {
        self.wind.clone()
    }
}

#[derive(Component, Clone, Copy)]
pub(super) struct Wind {
    vector: Vec2,
}

#[derive(Component, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub(super) struct GpuWind {
    vector: Vec2,
    padding: Vec2, // needs 16 byte alignment on gpu
}

impl GpuComponentArrayBuffer for Wind {
    type QueryData = Read<Wind>;
    type QueryFilter = Changed<Wind>;
    type Out = GpuWind;

    fn extract_component(item: QueryItem<'_, '_, Self::QueryData>) -> Option<Self::Out> {
        Some(GpuWind {
            vector: item.vector,
            padding: Vec2::ZERO,
        })
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct WindMaterial {
    #[uniform(0)]
    colors: WindColors,
    #[storage(1, read_only)]
    data: Handle<ShaderBuffer>,
}

#[derive(ShaderType, Debug, Clone)]
struct WindColors {
    main_color_weak: LinearRgba,
    main_color_strong: LinearRgba,
    accent_color: LinearRgba,
}

impl Material2d for WindMaterial {
    fn fragment_shader() -> ShaderRef {
        WIND_SHADER_PATH.into()
    }
}

#[derive(Component, Deref)]
pub(super) struct FeatherWeight(f32);

fn setup_wind(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WindMaterial>>,
    wind_gpu_data: Res<GpuComponentArray<Wind>>,
) {
    let buffer = wind_gpu_data.buffer.clone();
    let wind_mesh = meshes.add(Mesh::from(Rectangle::from_size(WIND_DIMENSIONS)));
    let wind_material = materials.add(WindMaterial {
        colors: WindColors {
            main_color_strong: LinearRgba::new(1.0, 1.0, 1.0, 1.0),
            main_color_weak: LinearRgba::new(0.8, 0.8, 0.8, 0.5),
            accent_color: LinearRgba::new(0.7, 0.0, 0.7, 0.8),
        },
        data: buffer,
    });

    commands.insert_resource(WindAssets {
        wind: (wind_material, wind_mesh),
    });
}

fn spawn_wind(mut commands: Commands, wind_assets: Res<WindAssets>) {
    let (material_handle, mesh_handle) = wind_assets.handles();

    commands.spawn((
        Mesh2d(mesh_handle.clone()),
        MeshMaterial2d(material_handle.clone()),
        Transform::from_translation(Vec3::new(1800.0, -50.0, 0.0)),
        Wind {
            vector: Vec2::new(1.0, 1.0),
        },
    ));
    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(material_handle),
        Transform::from_translation(Vec3::new(1800.0, 0.0, 0.0)),
        Wind {
            vector: Vec2::new(1.0, 1.0),
        },
    ));
}

fn apply_directional_wind(
    wind: Query<(&Wind, &Transform)>,
    player: Single<(&Transform, Option<&FeatherWeight>), With<Player>>,
    player_physics: Single<(&Mass, &mut LinearVelocity), With<Player>>,
) {
    let (transform, featherweight) = player.into_inner();
    let pt = transform.translation; // pt for player_translation
    let (mass, mut velocity) = player_physics.into_inner();
    let effective_mass = **mass * **(featherweight.or(Some(&FeatherWeight(1.0))).unwrap());

    wind.iter().for_each(|(wind, transform)| {
        // could probably streamline this with avian but I'm not particularly familiar
        let top = transform.translation.y + (WIND_DIMENSIONS.y / 2.0);
        let bot = transform.translation.y - (WIND_DIMENSIONS.y / 2.0);
        let right = transform.translation.x + (WIND_DIMENSIONS.x / 2.0);
        let left = transform.translation.x - (WIND_DIMENSIONS.x / 2.0);

        if pt.x > left && pt.x < right && pt.y > bot && pt.y < top {
            velocity.0.x += wind.vector.x / effective_mass;
            velocity.0.y += wind.vector.y / effective_mass;
        }
    });
}
