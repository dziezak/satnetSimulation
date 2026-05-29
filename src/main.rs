use std::f32::consts::PI;
use std::ops::Add;
use bevy::prelude::*;

mod components;
mod ui;
mod movement;
mod protocols;

use bevy_egui::{EguiPlugin};
use components::{Satellite, SimulationState};
use crate::components::{Earth, GroundStation};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .init_resource::<SimulationState>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            movement::move_satellites,
            protocols::update_networks,
            ui::draw_gui,
        ).chain())
        .run();
}

fn setup_scene(
    mut command: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sim_state: ResMut<SimulationState>,
    asset_server: Res<AssetServer>,
) {
    let num_planes = 6;
    let sats_per_plane = 8;
    let inclination = 45.0 * (PI / 180.0);
    let earth_texture: Handle<Image> = asset_server.load("earth.png");
    let stars_texture: Handle<Image> = asset_server.load("stars.png");

    sim_state.sim_speed = 1.0;
    sim_state.earth_radius = 2.5;
    sim_state.max_isl_distance = 6.0;
    sim_state.reset_ram = false;

    command.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 12.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    command.insert_resource(AmbientLight {
        color: Color::rgb(0.6, 0.6, 0.7),
        brightness: 800.0,
    });

    command.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    command.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 2000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(-10.0, -10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    command.spawn(PbrBundle {
        mesh: meshes.add(Sphere::new(50.0).mesh().ico(3).unwrap()),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(stars_texture),
            unlit: true,
            cull_mode: None,
            ..default()
        }),
        ..default()
    });

    command.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(2.5).mesh().ico(5).unwrap()),
            material: materials.add(StandardMaterial{
                base_color_texture: Some(earth_texture),
                ..default()
            }),
            ..default()
        },
        Earth,
    ));

    let station_pos = Vec3::new(0.0, sim_state.earth_radius, 0.0);
    command.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.15).mesh().ico(3).unwrap()),
            material: materials.add(Color::rgb(1.0, 1.0, 1.0)),
            transform: Transform::from_translation(station_pos),
            ..default()
        },
        GroundStation { position: station_pos },
    ));

    let sat_body = meshes.add(Cuboid::new(0.18, 0.10, 0.10));
    let sat_panel = meshes.add(Cuboid::new(0.30, 0.02, 0.09));
    let antenna_mesh = meshes.add(Cuboid::new(0.02, 0.08, 0.02));
    let panel_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.05, 0.05, 0.4),
        emissive: Color::rgb(0.0, 0.0, 0.15),
        metallic: 0.9,
        perceptual_roughness: 0.2,
        ..default()
    });
    let antenna_mat = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.8, 0.8),
        metallic: 1.0,
        ..default()
    });

    let mut sat_id = 0;
    for plane in 0..num_planes {
        let lan = (plane as f32) * (2.0 * PI / num_planes as f32);

        for sat in 0..sats_per_plane {
            let starting_angle = (sat as f32) * (2.0 * PI / sats_per_plane as f32);

            command.spawn((
                PbrBundle {
                    mesh: sat_body.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::rgb(0.5, 0.5, 0.5),
                        metallic: 0.8,
                        perceptual_roughness: 0.3,
                        ..default()
                    }),
                    ..default()
                },
                Satellite {
                    id: sat_id,
                    orbit_radius: sim_state.earth_radius + 1.5,
                    current_angle: starting_angle,
                    orbit_speed: 0.4,
                    inclination,
                    lan,
                    connection_timer: 0.0,
                    ram_usage: 1.0,
                    cpu_load: 5.0,
                    status_msg: "OK".to_string(),
                    is_dead: false,
                },
            )).with_children(|parent| {
                parent.spawn(PbrBundle {
                    mesh: sat_panel.clone(),
                    material: panel_mat.clone(),
                    transform: Transform::from_xyz(-0.25, 0.0, 0.0),
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: sat_panel.clone(),
                    material: panel_mat.clone(),
                    transform: Transform::from_xyz(0.25, 0.0, 0.0),
                    ..default()
                });
                parent.spawn(PbrBundle {
                    mesh: antenna_mesh.clone(),
                    material: antenna_mat.clone(),
                    transform: Transform::from_xyz(0.0, 0.09, 0.0),
                    ..default()
                });
            });
            sat_id += 1;
        }
    }
}