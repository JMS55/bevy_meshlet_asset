//! Meshlet rendering for dense high-poly scenes (experimental).

// Note: This example showcases the meshlet API, but is not the type of scene that would benefit from using meshlets.

#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

use bevy::{
    pbr::{
        experimental::meshlet::{
            MaterialMeshletMeshBundle, MeshletPlugin, DEFAULT_VERTEX_POSITION_QUANTIZATION_FACTOR,
        },
        CascadeShadowConfigBuilder, DirectionalLightShadowMap,
    },
    prelude::*,
    render::render_resource::AsBindGroup,
};
use bytemuck::Pod;
use camera_controller::{CameraController, CameraControllerPlugin};
use lz4_flex::frame::FrameEncoder;
use std::{f32::consts::PI, io::Write, path::Path, process::ExitCode};

fn main() -> ExitCode {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins((
            DefaultPlugins,
            MeshletPlugin {
                cluster_buffer_slots: 8192,
            },
            MaterialPlugin::<MeshletDebugMaterial>::default(),
            CameraControllerPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, draw_bounding_spheres)
        .run();

    ExitCode::SUCCESS
}

#[derive(Resource)]
struct Foo(Handle<Mesh>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Foo(asset_server.load("models/cliff.glb#Mesh0/Primitive0")));
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Default)]
struct MeshletDebugMaterial {
    _dummy: (),
}
impl Material for MeshletDebugMaterial {}

fn draw_bounding_spheres(mut assets: ResMut<Assets<Mesh>>, foo: ResMut<Foo>) {
    if let Some(mesh) = assets.get_mut(foo.0.id()) {
        let asset = bevy::pbr::experimental::meshlet::MeshletMesh::from_mesh(
            mesh,
            DEFAULT_VERTEX_POSITION_QUANTIZATION_FACTOR,
        )
        .unwrap();

        let mut f = std::io::BufWriter::new(
            std::fs::File::create("assets/models/cliff.meshlet_mesh").unwrap(),
        );

        // Write asset magic number
        f.write_all(&1717551717668u64.to_le_bytes()).unwrap();

        // Write asset version
        f.write_all(&1u64.to_le_bytes()).unwrap();

        // Compress and write asset data
        let mut writer = FrameEncoder::new(f);
        write_slice(&asset.vertex_positions, &mut writer);
        write_slice(&asset.vertex_normals, &mut writer);
        write_slice(&asset.vertex_uvs, &mut writer);
        write_slice(&asset.indices, &mut writer);
        write_slice(&asset.meshlets, &mut writer);
        write_slice(&asset.meshlet_bounding_spheres, &mut writer);
        write_slice(&asset.meshlet_simplification_errors, &mut writer);
        writer.finish().unwrap();

        panic!("Exit program");
    }
}

fn write_slice<T: Pod>(field: &[T], writer: &mut dyn Write) {
    writer
        .write_all(&(field.len() as u64).to_le_bytes())
        .unwrap();
    writer.write_all(bytemuck::cast_slice(field)).unwrap();
}
