use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
pub use crate::components::{SimulationState, Satellite, RoutingProtocol};

pub fn draw_gui(
    mut contexts: EguiContexts,
    mut sim_state: ResMut<SimulationState>,
) {
    egui::SidePanel::left("control_panel")
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("SPACE-ROUTER SIMULATOR");
            ui.separator();

            ui.label("Wybierz protokół:");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::TerrestrialOSPF, "Terrestrial OSPFv3");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::SatnetOSPF, "⚡ SATNET-OSPF Framework");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::ContactGraphRouting, "DTN / CGR");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::CentralizedSDN, "SDN Controller");

            ui.separator();
            ui.add(egui::Slider::new(&mut sim_state.sim_speed, 0.0..=5.0).text("Speed"));
        });
}