use std::thread::current;
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Assets, Color, Entity, GizmoPrimitive3d, Gizmos, Handle, Quat, Query, Res, ResMut, Sphere, Time, Transform};
use crate::components::{GroundStation, RoutingProtocol, Satellite, SimulationState};

fn has_line_of_sight(pos_a: Vec3, pos_b: Vec3, earth_radius: f32) -> bool {
    let ray = pos_b - pos_a;
    let ray_dir = ray.normalize();

    let t = -pos_a.dot(ray_dir);

    if t < 0.0 || t > ray.length() {
        return true;
    }

    let projection = pos_a + ray_dir * t;
    projection.length() > earth_radius
}


pub fn update_networks(
    time: Res<Time>,
    sim_state: Res<SimulationState>,
    station_query: Query<&GroundStation>,
    mut sat_query: Query<(Entity, &mut Transform, &mut Satellite, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmos: Gizmos,
) {
    if sim_state.is_paused { return; }

    let Ok(station) = station_query.get_single() else { return };
    let current_station_position = Vec3::new(0.0, sim_state.earth_radius, 0.0);

    struct SatData {
        entity:  Entity,
        pos: Vec3,
        has_path: bool,
        parnet_pos: Option<Vec3>,
    }

    let mut sat_list: Vec<SatData> = Vec::new();

    for (entity, transform, _, _) in sat_query.iter() {
        sat_list.push(SatData {
            entity,
            pos: transform.translation,
            has_path: false,
            parnet_pos: None,
        });
    }

    let mut queue = Vec::new();

    for sat in sat_list.iter_mut() {
        if has_line_of_sight(sat.pos, current_station_position, sim_state.earth_radius) {
            sat.has_path = true;
            sat.parnet_pos = Some(current_station_position);
            queue.push(sat.entity);
        }
    }

    while let Some(current_entity) = queue.pop() {
        let current_pos = sat_list.iter().find(|s| s.entity == current_entity).unwrap().pos;

        for i in 0..sat_list.len() {
            if !sat_list[i].has_path {
                let dist = current_pos.distance(sat_list[i].pos);
                if dist <= sim_state.max_isl_distance && has_line_of_sight(current_pos, sat_list[i].pos, sim_state.earth_radius) {
                    sat_list[i].has_path = true;
                    sat_list[i].parnet_pos = Some(current_pos);
                    queue.push(sat_list[i].entity);
                }
            }
        }
    }

    let line_color = match sim_state.current_protocol {
        RoutingProtocol::TerrestrialOSPF => Color::rgb(1.0, 0.3, 0.0), // Ciemny pomarańczowy
        RoutingProtocol::SatnetOSPF => Color::rgb(0.0, 1.0, 0.0),      // Jasny zielony
        RoutingProtocol::ContactGraphRouting => Color::rgb(1.0, 0.6, 0.0), // Żółto-pomarańczowy
        RoutingProtocol::CentralizedSDN => Color::rgb(0.0, 0.8, 1.0),   // Cyjan/Morski dla SDN
    };

    for sat in sat_list.iter() {
        if let Some(parent) = sat.parnet_pos {
            gizmos.line(sat.pos, parent, line_color);
        }
    }

    for (entity, _, mut sat, mat_handle) in sat_query.iter_mut() {
        if let Some(material) = materials.get_mut(mat_handle.id()) {

            let has_network_access = sat_list.iter().find(|s| s.entity == entity).map(|s| s.has_path).unwrap_or(false);

            if !has_network_access {
                sat.connection_timer = 0.0;
                sat.status_msg = "NO ROUTE TO GROUND".to_string();
                sat.cpu_load = 2.0;
                material.base_color = Color::rgb(1.0, 0.0, 0.0);
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
                        material.base_color = Color::rgb(1.0, 0.5, 0.0); // POMARAŃCZOWY
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
                        sat.status_msg = "STORE & FORWARD: Routing data".to_string();
                        material.base_color = Color::rgb(1.0, 0.5, 0.0);
                    }
                }

                RoutingProtocol::CentralizedSDN => {
                    let propagation_delay = 4.0;
                    sat.ram_usage = 4.5;
                    sat.cpu_load = 15.0;

                    if sat.connection_timer < propagation_delay {
                        sat.status_msg = "AWAITING SDN CONTROLLER FLOW...".to_string();
                        material.base_color = Color::rgb(1.0, 0.5, 0.0);
                    } else {
                        sat.status_msg = "SDN ROUTE ACTIVE".to_string();
                        material.base_color = Color::rgb(0.0, 1.0, 0.0);
                    }
                }
            }
        }
    }

    for (_entity, mut transform, sat, _mat_handle) in sat_query.iter_mut() {
        if sim_state.selected_satellite_id == Some(sat.id) {
            transform.scale = Vec3::splat(2.5);

            gizmos.primitive_3d(
                Sphere::new(0.4),
                transform.translation,
                Quat::IDENTITY,
                Color::rgb(0.0, 0.8, 1.0)
            );

            gizmos.line(
                transform.translation,
                Vec3::ZERO,
                Color::rgba(0.0, 0.8, 1.0, 0.3)
            );
        } else {
            transform.scale = Vec3::splat(1.0);
        }
    }
}