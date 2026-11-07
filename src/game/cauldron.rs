use bevy::prelude::*;

use crate::{
    PausableSystems,
    game::{
        cauldron,
        inventory::{MaterialInventory, PotionsInventory},
        materials::Materials,
        player::Player,
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_cauldrons)
        .add_systems(Update, spawn_cauldrons.run_if(in_state(Screen::Gameplay)))
        .add_systems(Update, craft_with_cauldrons.in_set(PausableSystems));
}

#[derive(Resource)]
pub(super) struct CauldronAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub(super) struct Cauldron;

fn setup_cauldrons(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(10.0, 10.0));
    let material = materials.add(Color::BLACK);

    commands.insert_resource(CauldronAssets { material, mesh });
}

fn spawn_cauldrons(mut commands: Commands, cauldron_assets: Res<CauldronAssets>) {
    commands.spawn((
        Cauldron,
        Transform::from_translation(Vec3::new(1900.0, -50.0, 0.0)),
        Mesh2d(cauldron_assets.mesh.clone()),
        MeshMaterial2d(cauldron_assets.material.clone()),
    ));
}

fn craft_with_cauldrons(
    cauldrons: Query<&Transform, With<Cauldron>>,
    player: Single<(&Transform, &mut MaterialInventory, &mut PotionsInventory), With<Player>>,
) {
    let (transform, mut material_inventory, mut potions_inventory) = player.into_inner();

    cauldrons.iter().for_each(|cauldron_transform| {
        if cauldron_transform
            .translation
            .distance(transform.translation)
            < 100.0
        {
            // TODO: some activation condition via UI, tbd
            if material_inventory
                .get_indexed_item()
                .and_then(|item| Some(potions_inventory.can_add_item(item)))
                .or(Some(false))
                .unwrap()
                && let Ok(item) = material_inventory.brew_item()
            {
                potions_inventory.add_item(item);
            }
        }
    });
}

fn display_cauldron() -> impl Scene {
    bsn! {
        DespawnOnExit::<Screen>(Screen::Gameplay)
    }
}
