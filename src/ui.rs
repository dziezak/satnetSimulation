use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
pub use crate::components::{SimulationState, Satellite, RoutingProtocol};

pub fn draw_gui(
    mut contexts: EguiContexts,
    mut sim_state: ResMut<SimulationState>,
    sat_query: Query<&Satellite>,
) {
    egui::SidePanel::left("control_panel")
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("SPACE-ROUTER SIMULATOR");
            ui.separator();

            ui.horizontal(|ui| {
                let button_label = if sim_state.is_paused { "play" } else { "pause" };
                if ui.button(button_label).clicked() {
                    sim_state.is_paused = !sim_state.is_paused;
                }
                ui.label(format!("Speed: {:.1}x", sim_state.sim_speed));
            });

            ui.separator();
            ui.label("Choose Protocol:");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::TerrestrialOSPF, "Terrestrial OSPFv3");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::SatnetOSPF, "⚡ SATNET-OSPF Framework");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::ContactGraphRouting, "DTN / CGR");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::CentralizedSDN, "SDN Controller");

            ui.separator();
            ui.add(egui::Slider::new(&mut sim_state.sim_speed, 0.0..=5.0).text("Speed"));
            ui.add(egui::Slider::new(&mut sim_state.earth_radius, 1.0..=8.0).text("Earth Radius"));
            ui.add(egui::Slider::new(&mut sim_state.max_isl_distance, 0.0..=25.0).text("Max IS distance"));

            ui.separator();
            if ui.button("Snap back to reality").clicked() {
                sim_state.sim_speed = 1.0;
                sim_state.earth_radius = 2.5;
                sim_state.max_isl_distance = 4.2;
            }

            ui.heading("Satelite Inspector");
            egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                let mut sorted_sats: Vec<&Satellite> = sat_query.iter().collect();
                sorted_sats.sort_by_key(|s| s.id);

                for sat in sorted_sats {
                    let is_selected = sim_state.selected_satellite_id == Some(sat.id);

                    let response = ui.selectable_label(
                        is_selected,
                        format!("Sat #{}: [{}]", sat.id, sat.status_msg)
                    );

                    if response.clicked() {
                        if is_selected {
                            sim_state.selected_satellite_id = None;
                        } else {
                            sim_state.selected_satellite_id = Some(sat.id);
                        }
                    }
                }
            });

            if let Some(selected_id) = sim_state.selected_satellite_id {
                if let Some(sat) = sat_query.iter().find(|s| s.id == selected_id) {
                    ui.separator();
                    ui.colored_label(egui::Color32::LIGHT_BLUE, format!("Details Sat #{}", sat.id));
                    ui.label(format!("Status: {}", sat.status_msg));
                    ui.label(format!("Connection Time: {}", sat.connection_timer));
                    ui.label(format!("CPU load: {}", sat.cpu_load));
                    ui.label(format!("Ram usage: {}", sat.ram_usage));
                }
            }
        });
}