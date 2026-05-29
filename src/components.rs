use bevy::prelude::{Bundle, Component, Resource, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingProtocol {
    #[default]
    TerrestrialOSPF,
    SatnetOSPF,
    ContactGraphRouting,
    CentralizedSDN,
}

#[derive(Resource, Default)]
pub struct SimulationState {
    pub current_protocol: RoutingProtocol,
    pub sim_speed: f32,
    pub is_paused: bool,
    pub earth_radius: f32,
    pub max_isl_distance: f32,
    pub selected_satellite_id: Option<u32>,
    pub satnet_options: SimulationSatnetPlugin,
    pub reset_ram: bool,
}


#[derive(Default)]
pub struct SimulationSatnetPlugin {
    pub opt_p2p_mapping: bool,
    pub opt_fast_link_lock: bool,
    pub opt_rfp_predictable: bool,
    pub opt_low_footprint_top: bool,
}

#[derive(Component)]
pub struct Earth;

#[derive(Component)]
pub struct GroundStation {
    //pub position: Vec3,
}

#[derive(Component)]
pub struct Satellite {
    pub id: u32,
    pub orbit_radius: f32,
    pub current_angle: f32,
    pub orbit_speed: f32,
    pub inclination: f32,
    pub lan: f32,

    pub connection_timer: f32,
    pub ram_usage: f32,
    pub cpu_load: f32,
    pub status_msg: String,
    pub is_dead: bool,
}