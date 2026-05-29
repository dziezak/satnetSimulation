use std::collections::VecDeque;
use std::thread::current;
use bevy::math::Vec3;
use bevy::pbr::StandardMaterial;
use bevy::prelude::{Assets, Color, Entity, GizmoPrimitive3d, Gizmos, Handle, Quat, Query, Res, ResMut, Sphere, Time, Transform};
use bevy::utils::petgraph::algo::greedy_feedback_arc_set;
use bevy_egui::egui::Shape::Vec;
use crate::components::{GroundStation, RoutingProtocol, Satellite, SimulationState};
use crate::movement::calculate_satellite_position;

struct SatData {
    entity:  Entity,
    pos: Vec3,
    has_path: bool,
    parent_pos: Option<Vec3>,
    will_lose_link: bool,
}

fn has_line_of_sight(pos_a: Vec3, pos_b: Vec3, earth_radius: f32) -> bool {
    let ray = pos_b - pos_a;
    let ray_dir = ray.normalize();

    let t = -pos_a.dot(ray_dir);

    if t < 0.0 || t > ray.length() {
        return true;
    }

    let projection = pos_a + ray_dir * t;
    projection.length() > earth_radius * 1.001
}


pub fn update_networks(
    time: Res<Time>,
    mut sim_state: ResMut<SimulationState>,
    station_query: Query<&GroundStation>,
    mut sat_query: Query<(Entity, &mut Transform, &mut Satellite, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmos: Gizmos,
) {
    if sim_state.is_paused { return; }

    let Ok(station) = station_query.get_single() else { return };
    let sat_list = building_network_topology(&*sim_state, &sat_query);

    let line_color = get_protocol_line_color(&sim_state.current_protocol);

    for (entity, mut transform, mut sat, mat_handle) in sat_query.iter_mut() {
        if let Some(material) = materials.get_mut(mat_handle.id()) {

            let has_access = sat_list.iter()
                .find(|s| s.entity == entity)
                .map(|s| s.has_path)
                .unwrap_or(false);

            let parent_position = sat_list.iter()
                .find(|s| s.entity == entity)
                .and_then(|s| s.parent_pos);

            let will_lose = sat_list.iter()
                .find(|s| s.entity == entity)
                .map(|s| s.will_lose_link)
                .unwrap_or(false);

            calculate_satellite_metrics(&time, &sim_state, &mut sat, has_access, will_lose);

            if let Some(parent) = parent_position {
                if has_access {
                    gizmos.line(transform.translation, parent, line_color);
                } else if sat.status_msg.contains("GHOST LINK") {
                    let pulse = (time.elapsed_seconds() * 10.0).sin().abs();
                    gizmos.line(transform.translation, parent, Color::rgb(1.0, 0.0, 0.0));
                }
            }

            material.base_color = interpret_metrics_to_color(&sat);
            apply_inspector_visuals(&sim_state, &mut transform, &sat, &mut gizmos);
        }
    }

    if sim_state.reset_ram {
        for(_, _, mut sat, _) in sat_query.iter_mut() {
            sat.ram_usage = 0.0;
            sat.is_dead = false;
            sat.status_msg = String::new();
            sat.connection_timer = -1.0;
        }
        sim_state.reset_ram = false;
    }
}

fn building_network_topology(
    sim_state: &SimulationState,
    sat_query: &Query<(Entity, &mut Transform, &mut Satellite, &mut Handle<StandardMaterial>)>,
) -> std::vec::Vec<SatData> {
    let current_station_position = Vec3::new(0.0, sim_state.earth_radius, 0.0);
    let mut sat_list: std::vec::Vec<SatData> = std::vec::Vec::new();
    let mut queue = std::collections::VecDeque::new();
    let effective_distance = if sim_state.satnet_options.opt_fast_link_lock {
        sim_state.max_isl_distance
    } else {
        sim_state.max_isl_distance * 0.8 //20% stracone na detection time
    };

    for (entity, transform, _, _) in sat_query.iter() {

        sat_list.push(SatData {
            entity,
            pos: transform.translation,
            has_path: false,
            parent_pos: None,
            will_lose_link: false,
        });
    }

    for sat in sat_list.iter_mut() {
        if has_line_of_sight(sat.pos, current_station_position, sim_state.earth_radius) {
            sat.has_path = true;
            sat.parent_pos = Some(current_station_position);
            queue.push_back(sat.entity);
        }
    }

    while let Some(current_entity) = queue.pop_front() {
        let current_pos = sat_list.iter().find(|s| s.entity == current_entity).unwrap().pos;

        for i in 0..sat_list.len() {
            let sat_is_dead = sat_query.iter()
                .find(|(e, _, _, _)| *e == sat_list[i].entity)
                .map(|(_, _, s, _)| s.is_dead)
                .unwrap_or(false);

            let sat_status = sat_query.iter()
                .find(|(e, _, _, _)| *e == sat_list[i].entity)
                .map(|(_, _, s, _)| s.status_msg.clone())
                .unwrap_or_default();

            let is_ready = !sat_status.contains("BROADCAST")
                && !sat_status.contains("INIT")
                && !sat_status.is_empty()
                || sat_status.contains("RFP");

            if !sat_list[i].has_path && !sat_is_dead && is_ready {
                let dist = current_pos.distance(sat_list[i].pos);
                if dist <= effective_distance && has_line_of_sight(current_pos, sat_list[i].pos, sim_state.earth_radius) {
                    sat_list[i].has_path = true;
                    sat_list[i].parent_pos = Some(current_pos);
                    queue.push_back(sat_list[i].entity);
                }
            }
        }
    }

    let predict_seconds = 3.0;
    for sat_data in sat_list.iter_mut() {
        if let Some(parent) = sat_data.parent_pos {
            if let Some((_, _, sat, _)) = sat_query.iter().find(|(e, _, _,_)| *e == sat_data.entity) {
                let future_angle = sat.orbit_speed * predict_seconds * sim_state.sim_speed;
                let future_pos = calculate_satellite_position(sat, future_angle);

                let current_dist = sat_data.pos.distance(parent);
                let future_dist = future_pos.distance(parent);

                let loses_los = !has_line_of_sight(future_pos, parent, sim_state.earth_radius);
                let loses_dist = future_dist > effective_distance;
                let near_edge = current_dist > effective_distance * 0.8;

                sat_data.will_lose_link = near_edge && ( loses_los || loses_dist);
            }
        }
    }

    if sim_state.satnet_options.opt_rfp_predictable {
        for i in 0..sat_list.len() {
            if !sat_list[i].will_lose_link {continue;}

            if let Some(parent) = sat_list[i].parent_pos {
                if parent == current_station_position {continue;}
            }

            let future_pos = if let Some((_, _, sat, _)) = sat_query.iter()
                .find(|(e, _, _, _)| *e == sat_list[i].entity)
            {
                let future_angle = sat.orbit_speed * predict_seconds * sim_state.sim_speed;
                calculate_satellite_position(sat, future_angle)
            } else {
                continue;
            };

            let mut best_parent: Option<Vec3> = None;
            let mut best_dist = f32::MAX;

            let future_dist_to_station = future_pos.distance(current_station_position);
            if future_dist_to_station <= effective_distance
                && has_line_of_sight(future_pos, current_station_position, sim_state.earth_radius)
            {
                best_dist = future_dist_to_station;
                best_parent = Some(current_station_position);
            }

            for j in 0..sat_list.len() {
                if i == j {continue;}
                if !sat_list[j].has_path {continue;}
                if sat_list[j].will_lose_link {continue;}

                let parent_reaches_ground = has_path_to_ground(&sat_list, j, current_station_position);
                if !parent_reaches_ground { continue; }

                let parent_future_pos = if let Some((_, _, sat, _)) = sat_query.iter()
                    .find(|(e, _, _, _)| *e == sat_list[j].entity) //j
                {
                    let future_angle = sat.orbit_speed * predict_seconds * sim_state.sim_speed;
                    calculate_satellite_position(sat, future_angle)
                } else {
                    sat_list[j].pos
                };

                let dist = future_pos.distance(parent_future_pos);

                if dist < effective_distance
                    && has_line_of_sight(future_pos, parent_future_pos, sim_state.earth_radius)
                    && dist < best_dist
                {
                    best_dist = dist;
                    best_parent = Some(sat_list[j].pos);
                }
            }
            if let Some(new_parent) = best_parent {
                sat_list[i].parent_pos = Some(new_parent);
            }
        }
    }

    sat_list
}


fn update_ram_is_ok(
    sat: &mut Satellite,
    amount: f32,
)-> bool {
    sat.ram_usage += amount;
    if sat.ram_usage >= 25.0 {
        sat.is_dead = true;
        sat.status_msg = "DEAD: RAM Overflow - system failure".to_string();
        sat.cpu_load = 0.0;
        return false;
    }
    true
}

fn calculate_satellite_metrics(
    time: &Res<Time>,
    sim_state: &SimulationState,
    sat: &mut Satellite,
    has_network_access: bool,
    will_lose_link: bool,
) {
    if sat.is_dead {return;}

    if will_lose_link && sim_state.satnet_options.opt_rfp_predictable {
        sat.status_msg = "RFP: Pre-routing to next node...".to_string();
        return;
    }

    let delta = time.delta_seconds() * sim_state.sim_speed;

    if !has_network_access {
        sat.cpu_load = 2.0;

        if sim_state.current_protocol == RoutingProtocol::SatnetOSPF {
            if sim_state.satnet_options.opt_rfp_predictable {
                sat.status_msg = "RFP ACTIVE: Rerouted seamlessly".to_string();
                sat.ram_usage = 1.8;
                //sat.connection_timer = -1.0;
            } else if sim_state.satnet_options.opt_fast_link_lock {
                sat.status_msg = "LINK LOST: HW Lock Flag Triggered".to_string();
                sat.ram_usage = 1.0;
                sat.connection_timer = -1.0;
            } else {
                if sat.status_msg.contains("CONNECTED") {
                    sat.connection_timer = 0.0;
                    sat.status_msg = "GHOST LINK: Missing Hello packets (0.0s/6s)".to_string();
                }

                if sat.status_msg.contains("GHOST LINK") {
                    if sat.connection_timer < 6.0 {
                        sat.connection_timer += delta;
                        sat.status_msg = format!("GHOST LINK: Missing Hello packets ({:.1}s/6s)",sat.connection_timer);
                        sat.cpu_load = 40.0;
                        if !sim_state.satnet_options.opt_low_footprint_top {
                            if !update_ram_is_ok(sat, delta * 2.0) {return;}
                        }
                        return;
                    } else {
                        sat.status_msg = "CRITICAL: Waiting for Hello Timeout".to_string();
                        sat.connection_timer = -1.0;
                    }
                }

                if sat.status_msg.contains("CRITICAL") {
                    sat.cpu_load = 99.0;
                    sat.ram_usage = 24.0;
                }
            }
        } else {
            sat.status_msg = "NO ROUTE TO GROUND".to_string();
            sat.ram_usage = 1.0;
            sat.connection_timer = -1.0;
        }
        return;
    }

    if sat.connection_timer < 0.0
        || sat.status_msg.contains("NO ROUTE")
        || sat.status_msg.contains("LINK LOST")
        || sat.status_msg.contains("CRITICAL")
        || sat.status_msg.contains("GHOST LINK")
        || sat.status_msg.contains("BROADCAST")
        || sat.status_msg.is_empty()
    {
        sat.connection_timer = 0.0;
        sat.status_msg = "INIT".to_string();
        sat.ram_usage = 8.5;
    }

    sat.connection_timer += delta;

    match sim_state.current_protocol {
        RoutingProtocol::TerrestrialOSPF => {
            let handshake_time = 10.0;
            sat.cpu_load = 85.0;
            sat.ram_usage = 32.4;

            if sat.connection_timer < handshake_time {
                sat.status_msg = format!("OSPF INIT: ({:.1}s/10s)", sat.connection_timer);
            } else {
                sat.status_msg = "TIMEOUT: Link dropped before SPF Conv.".to_string();
                sat.cpu_load = 99.5;
            }
        }

        RoutingProtocol::SatnetOSPF => {
            let handshake_time = if sim_state.satnet_options.opt_p2p_mapping { 0.3} else {5.0};
            let base_cpu = if sim_state.satnet_options.opt_p2p_mapping {12.0} else {24.0};

            if sim_state.satnet_options.opt_low_footprint_top {
                if sat.ram_usage < 1.8 {
                    sat.ram_usage = 1.8;
                }
            } else {
                if sat.ram_usage < 8.5 {
                    sat.ram_usage = 8.5;
                }
                if !update_ram_is_ok(sat, delta * 2.0) {return}
            }

            sat.cpu_load = base_cpu;

            if sat.connection_timer < handshake_time {
                if sim_state.satnet_options.opt_p2p_mapping {
                    sat.status_msg = "P2P ADJACENCY: Immediate (No DR/BDR)".to_string()
                } else {
                    sat.status_msg = format!("OSPF BROADCAST: Electing DR/BDR ({:.1}s/5s)",sat.connection_timer).to_string();
                    sat.cpu_load += 20.0; // dodatkowe obciazenie na czas elekcji
                }
            } else {
                if !sim_state.satnet_options.opt_low_footprint_top && sat.ram_usage > 20.0 {
                    sat.status_msg = "CONNECTED [DB OVERFLOW: Performance Degaded]".to_string();
                    sat.cpu_load = 80.0;
                } else if will_lose_link && sim_state.satnet_options.opt_rfp_predictable {
                    sat.status_msg = "RFP: Pre-prerouting to next node...".to_string();
                    sat.cpu_load += 5.0;
                } else {
                    let mut msg = "CONNECTED".to_string();
                    if sim_state.satnet_options.opt_p2p_mapping { msg += " [P2P]"; }
                    if sim_state.satnet_options.opt_low_footprint_top { msg += " [AdjacencyList]"; }
                    sat.status_msg = msg;
                }
            }
        }

        RoutingProtocol::ContactGraphRouting => {
            sat.cpu_load = 20.0;
            sat.ram_usage = (5.0 + sat.connection_timer * 1.5).min(25.0);

            if sat.ram_usage > 25.0 {
                sat.status_msg = "RAM OVERFLOW: Buffer Full!".to_string();
            } else {
                sat.status_msg = "STORE & FORWARD: Routing data".to_string();
            }
        }

        RoutingProtocol::CentralizedSDN => {
            let propagation_delay = 4.0;
            sat.ram_usage = 4.5;
            sat.cpu_load = 15.0;

            if sat.connection_timer < propagation_delay {
                sat.status_msg = "AWAITING SDN CONTROLLER FLOW...".to_string();
            } else {
                sat.status_msg = "SDN ROUTE ACTIVE".to_string();
            }
        }
    }
}

fn interpret_metrics_to_color(sat: &Satellite) -> Color {
    if sat.is_dead || sat.ram_usage > 25.0 || sat.cpu_load >= 99.0 || sat.status_msg.contains("TIMEOUT") || sat.status_msg.contains("CRITICAL"){
        return Color::rgb(1.0, 0.0, 0.0);
    }

    if sat.status_msg.contains("RFP") {
        return Color::rgb(0.0, 0.5, 1.0);
    }

    if sat.status_msg.contains("GHOST LINK") {
        return Color::rgb(0.0, 0.0, 1.0);
    }

    if sat.status_msg.contains("INIT")
        || sat.status_msg.contains("Election")
        || sat.status_msg.contains("AWAITING")
        || sat.status_msg.contains("BROADCAST")
        || sat.status_msg.contains("RFP ACTIVE")
    {
        return Color::rgb(1.0, 0.5, 0.0);
    }

    if sat.status_msg.contains("CONNECTED") || sat.status_msg.contains("ACTIVE") || sat.status_msg.contains("HW LOCK"){
        return Color::rgb(0.0, 1.0, 0.0);
    }
    Color::rgb(0.5, 0.5, 0.5)
}

fn apply_inspector_visuals(
    sim_state:&SimulationState,
    transform: &mut Transform,
    sat: &Satellite,
    gizmos: &mut Gizmos,
) {
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


fn get_protocol_line_color(protocol: &RoutingProtocol) -> Color {
    match protocol {
        RoutingProtocol::TerrestrialOSPF => Color::rgb(1.0, 0.3, 0.0),
        RoutingProtocol::SatnetOSPF => Color::rgb(0.0, 1.0, 0.0),
        RoutingProtocol::ContactGraphRouting => Color::rgb(1.0, 0.6, 0.0),
        RoutingProtocol::CentralizedSDN => Color::rgb(0.0, 0.8, 1.0),
    }
}

fn has_path_to_ground(
    sat_list: &[SatData],
    start_idx: usize,
    station_pos: Vec3,
) -> bool {
    if let Some(parent) = sat_list[start_idx].parent_pos {
        if parent == station_pos {
            return true;
        }
    }

    let mut visited = std::collections::HashSet::new();
    let mut current = start_idx;

    loop {
        if visited.contains(&current) { return false; }
        visited.insert(current);

        let parent_pos = match sat_list[current].parent_pos {
            Some(p) => p,
            None => return false,
        };

        if parent_pos == station_pos { return true; }

        match sat_list.iter().position(|s| s.pos == parent_pos) {
            Some(idx) => current = idx,
            None => return true,
        }
    }
}
