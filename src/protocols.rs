use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Assets, Color, Handle, Query, Res, ResMut, Time, Transform};
use crate::components::{GroundStation, RoutingProtocol};
use crate::ui::{Satellite, SimulationState};

fn has_line_of_sight(sat_pos: Vec3, station_pos: Vec3, earth_radius: f32) -> bool {
    let ray = sat_pos - station_pos;
    let ray_dir = ray.normalize();

    let t = -station_pos.dot(ray_dir);

    let projection = station_pos + ray_dir * t;

    if projection.length() > earth_radius {
        return true;
    }

    t < 0.0 || t > ray.length()
}


pub fn update_networks(
    time: Res<Time>,
    sim_state: Res<SimulationState>,
    station_query: Query<&GroundStation>,
    mut sat_query: Query<(&Transform, &mut Satellite, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if sim_state.is_paused { return; }

    let Ok(station) = station_query.get_single() else { return };
    let earth_radius = 2.5;

    for (transform, mut sat, mat_handle) in sat_query.iter_mut() {
        if let Some(material) = materials.get_mut(mat_handle.id()) {

            let los = has_line_of_sight(transform.translation, station.position, earth_radius);

            if !los {
                sat.connection_timer = 0.0;
                sat.status_msg = "OUT OF RANGE".to_string();
                sat.cpu_load = 2.0;
                material.base_color = Color::rgb(0.3, 0.3, 0.3); // Szary - brak zasięgu
                continue;
            }

            sat.connection_timer += time.delta_seconds() * sim_state.sim_speed;

            match sim_state.current_protocol {

                RoutingProtocol::TerrestrialOSPF => {
                    let handshake_time = 10.0;

                    sat.cpu_load = 85.0;
                    sat.ram_usage = 32.4;

                    if sat.connection_timer < handshake_time {
                        sat.status_msg = format!("OSPF INIT: ({:.1}s/10s)", sat.connection_timer);
                        material.base_color = Color::rgb(1.0, 0.5, 0.0);
                    } else {
                        sat.status_msg = "TIMEOUT: Link dropped before SPF Conv.".to_string();
                        sat.cpu_load = 99.0;
                        material.base_color = Color::rgb(1.0, 0.0, 0.0);
                    }
                }

                RoutingProtocol::SatnetOSPF => {
                    let handshake_time = 0.5;

                    sat.cpu_load = 12.0;
                    sat.ram_usage = 1.8;

                    if sat.connection_timer < handshake_time {
                        sat.status_msg = "SYNCHRONIZING...".to_string();
                        material.base_color = Color::rgb(1.0, 0.5, 0.0);
                    } else {
                        sat.status_msg = "CONNECTED (SATNET OPTIMIZED)".to_string();
                        material.base_color = Color::rgb(0.0, 1.0, 0.0);
                    }
                }

                RoutingProtocol::ContactGraphRouting => {
                    sat.cpu_load = 20.0;

                    sat.ram_usage = 5.0 + (sat.connection_timer * 2.0);

                    if sat.ram_usage > 25.0 {
                        sat.status_msg = "RAM OVERFLOW: Buffer Full!".to_string();
                        material.base_color = Color::rgb(1.0, 0.0, 0.0);
                    } else {
                        sat.status_msg = "STORE & FORWARD: Buffering packets".to_string();
                        material.base_color = Color::rgb(0.0, 0.6, 1.0);
                    }
                }

                RoutingProtocol::CentralizedSDN => {
                    let propagation_delay = 4.0;

                    sat.ram_usage = 4.5;
                    sat.cpu_load = 15.0;

                    if sat.connection_timer < propagation_delay {
                        sat.status_msg = "AWAITING SDN CONTROLLER FLOWS...".to_string();
                        material.base_color = Color::rgb(1.0, 0.5, 0.0);
                    } else {
                        sat.status_msg = "SDN ROUTE ACTIVE".to_string();
                        material.base_color = Color::rgb(0.0, 1.0, 0.0);
                    }
                }
            }
        }
    }
}