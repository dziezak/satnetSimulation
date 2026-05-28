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

            if sim_state.current_protocol == RoutingProtocol::SatnetOSPF {
                ui.spacing_mut().item_spacing.x = 5.0;
                ui.group(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(0, 200, 255),
                        "SATNET-OSPF Optimizations (enable in order):"
                    );
                    ui.separator();

                    // KROK 1
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut sim_state.satnet_options.opt_p2p_mapping, "① P2P Mapping");
                        ui.label("ℹ").on_hover_text(
                            "Problem: Terrestrial OSPF zakłada sieć broadcast (DR/BDR election ~5s).\n\
                 Fix: Każdy link traktowany jako punkt-punkt → natychmiastowa adjacency (0.3s).\n\
                 Efekt: Eliminuje wybór DR/BDR, drastycznie skraca czas połączenia."
                        );
                    });

                    // KROK 2 — zależy od kroku 1
                    ui.add_enabled_ui(sim_state.satnet_options.opt_p2p_mapping, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut sim_state.satnet_options.opt_low_footprint_top, "② Low Footprint Topology");
                            ui.label("ℹ").on_hover_text(
                                "Problem: OSPF trzyma pełną kopię LSDB (Link State Database) → RAM rośnie z czasem.\n\
                     Fix: Adjacency List zamiast pełnej macierzy → stały niski RAM (1.8 MB).\n\
                     Efekt: Zapobiega DB Overflow przy długich sesjach."
                            );
                        });
                        if !sim_state.satnet_options.opt_p2p_mapping {
                            sim_state.satnet_options.opt_low_footprint_top = false;
                        }
                    });

                    // KROK 3 — zależy od kroku 2
                    ui.add_enabled_ui(sim_state.satnet_options.opt_low_footprint_top, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut sim_state.satnet_options.opt_fast_link_lock, "③ Hardware Decoding Lock");
                            ui.label("ℹ").on_hover_text(
                                "Problem: Przy utracie linku OSPF czeka na Hello Timeout (6s) → ghost link.\n\
                     Fix: Hardware flag natychmiast wykrywa zerwanie łącza fizycznego.\n\
                     Efekt: Eliminuje ghost link state, szybszy failover."
                            );
                        });
                        if !sim_state.satnet_options.opt_low_footprint_top {
                            sim_state.satnet_options.opt_fast_link_lock = false;
                        }
                    });

                    // KROK 4 — pełny SATNET-OSPF, zależy od kroku 3
                    ui.add_enabled_ui(sim_state.satnet_options.opt_fast_link_lock, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut sim_state.satnet_options.opt_rfp_predictable, "④ Predictive RFP");
                            ui.label("ℹ").on_hover_text(
                                "Problem: Nawet z HW Lock, rerouting po utracie łącza zajmuje czas.\n\
                     Fix: Orbital mechanics pozwala przewidzieć kiedy link zniknie → pre-routing.\n\
                     Efekt: Zero downtime przy handover — pełny SATNET-OSPF."
                            );
                        });
                        if !sim_state.satnet_options.opt_fast_link_lock {
                            sim_state.satnet_options.opt_rfp_predictable = false;
                        }
                    });

                    // Status bar pokazujący aktualny "poziom"
                    ui.separator();
                    let level = [
                        sim_state.satnet_options.opt_p2p_mapping,
                        sim_state.satnet_options.opt_low_footprint_top,
                        sim_state.satnet_options.opt_fast_link_lock,
                        sim_state.satnet_options.opt_rfp_predictable,
                    ].iter().filter(|&&x| x).count();

                    let (label, color) = match level {
                        0 => ("● Vanilla OSPF (no optimizations)", egui::Color32::RED),
                        1 => ("● OSPF + P2P  [faster handshake]", egui::Color32::YELLOW),
                        2 => ("● OSPF + P2P + LF  [stable memory]", egui::Color32::from_rgb(255, 165, 0)),
                        3 => ("● OSPF + HW Lock  [no ghost links]", egui::Color32::from_rgb(100, 220, 100)),
                        4 => ("⚡ FULL SATNET-OSPF", egui::Color32::from_rgb(0, 200, 255)),
                        _ => ("", egui::Color32::WHITE),
                    };
                    ui.colored_label(color, label);
                });
            }


            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::ContactGraphRouting, "DTN / CGR");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::CentralizedSDN, "SDN Controller");

            ui.separator();
            ui.add(egui::Slider::new(&mut sim_state.sim_speed, 0.0..=5.0).text("Speed"));
            ui.add(egui::Slider::new(&mut sim_state.earth_radius, 1.0..=8.0).text("Earth Radius"));
            ui.add(egui::Slider::new(&mut sim_state.max_isl_distance, 0.0..=25.0).text("Max IS distance"));

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Snap back to reality").clicked() {
                    sim_state.sim_speed = 0.8;
                    sim_state.earth_radius = 2.5;
                    sim_state.max_isl_distance = 2.0;
                }
                if ui.button("Reset all RAM").clicked() {
                    sim_state.reset_ram = true;
                }
            });

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
                    ui.label(format!("Connection Time: {:.1}s", sat.connection_timer));
                    ui.label(format!("CPU load: {:.1}%", sat.cpu_load));

                    ui.label(format!("RAM: {:.1} / 25.0 MB", sat.ram_usage));
                    let ram_ratio = (sat.ram_usage / 25.0).clamp(0.0, 1.0);
                    let bar_color = if ram_ratio > 0.8 {
                        egui::Color32::RED
                    } else if ram_ratio > 0.5 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::GREEN
                    };
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(ui.available_width(), 12.0),
                        egui::Sense::hover()
                    );
                    ui.painter().rect_filled(rect, 2.0, egui::Color32::DARK_GRAY);
                    let mut fill_rect = rect;
                    fill_rect.max.x = rect.min.x + rect.width() * ram_ratio;
                    ui.painter().rect_filled(fill_rect, 2.0, bar_color);

                    if sat.ram_usage > 15.0 && !sim_state.satnet_options.opt_low_footprint_top {
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "⚠ LSDB growing — enable Low Footprint to prevent overflow"
                        );
                    }
                }
            }
        });
}