use bevy::prelude::*;
use crate::components::{Satellite, SimulationState};

pub fn move_satellites(
    time: Res<Time>,
    sim_state: Res<SimulationState>,
    mut query: Query<(&mut Transform, &mut Satellite)>,
) {
    if sim_state.is_paused { return; }

    for (mut transform, mut satellite) in query.iter_mut() {
        satellite.current_angle += satellite.orbit_speed * time.delta_seconds() * sim_state.sim_speed;

        let local_x = satellite.orbit_radius * satellite.current_angle.cos();
        let local_z = satellite.orbit_radius * satellite.current_angle.sin();
        let local_position = Vec3::new(local_x, 0.0, local_z);

        let inclination_rot = Quat::from_rotation_z(satellite.inclination);
        let lan_rot = Quat::from_rotation_y(satellite.lan);

        let global_position = lan_rot * inclination_rot * local_position;

        transform.translation = global_position;
    }
}