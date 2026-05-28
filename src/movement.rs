use bevy::prelude::*;
use crate::components::{Earth, GroundStation, Satellite, SimulationState};

pub fn move_satellites(
    time: Res<Time>,
    sim_state: Res<SimulationState>,
    mut sat_query: Query<(&mut Transform, &mut Satellite)>,
    mut earth_query: Query<&mut Transform, (With<Earth>, Without<Satellite>)>,
    mut station_query: Query<&mut Transform, (With<GroundStation>, Without<Earth>, Without<Satellite>)>,
) {
    if sim_state.is_paused { return; }

    if let Ok(mut earth_transform) = earth_query.get_single_mut() {
        earth_transform.scale = Vec3::splat(sim_state.earth_radius / 2.5);
    }

    if let Ok(mut station_transform) = station_query.get_single_mut() {
        station_transform.scale = Vec3::new(0.0, sim_state.earth_radius, 0.0);
    }

    for (mut transform, mut satellite) in sat_query.iter_mut() {
        satellite.current_angle += satellite.orbit_speed * time.delta_seconds() * sim_state.sim_speed;

        let dynamic_orbit_radius = sim_state.earth_radius + 1.2;

        let local_x = satellite.orbit_radius * satellite.current_angle.cos();
        let local_z = satellite.orbit_radius * satellite.current_angle.sin();
        let local_position = Vec3::new(local_x, 0.0, local_z);

        let inclination_rot = Quat::from_rotation_z(satellite.inclination);
        let lan_rot = Quat::from_rotation_y(satellite.lan);

        let global_position = lan_rot * inclination_rot * local_position;

        transform.translation = global_position;
    }
}

pub fn calculate_satellite_position(sat: &Satellite, angle_offset: f32) -> Vec3 {
    let angle = sat.current_angle + angle_offset;
    let local_x = sat.orbit_radius * angle.cos();
    let local_z = sat.orbit_radius * angle.sin();
    let local_position = Vec3::new(local_x, 0.0, local_z);

    let inclination_rot = Quat::from_rotation_z(sat.inclination);
    let lan_rot = Quat::from_rotation_y(sat.lan);

    lan_rot * inclination_rot * local_position
}