fn main() {
    println!("Hello, world!");
}
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::f32::consts::PI;

// --- 1. ENUM DLA WYBORU ALGORYTMU ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum RoutingProtocol {
    #[default]
    TerrestrialOSPF,
    SatnetOSPF,
    ContactGraphRouting,
    CentralizedSDN,
}

// --- 2. ZASOBY GLOBALNE (RESOURCES) ---
#[derive(Resource, Default)]
struct SimulationState {
    current_protocol: RoutingProtocol,
    sim_speed: f32,
    is_paused: bool,
}

// --- 3. KOMPONENTY (COMPONENTS) ---
#[derive(Component)]
struct Earth;

#[derive(Component)]
struct Satellite {
    id: u32,
    orbit_radius: f32,
    current_angle: f32,
    orbit_speed: f32,
    ram_usage: f32,    // w MB
    cpu_load: f32,     // w %
    status_msg: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "SATNET-OSPF Hardware-in-the-Loop Simulation".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        // Inicjalizacja zasobów
        .init_resource::<SimulationState>()
        // System startowy
        .add_systems(Startup, setup_scene)
        // Systemy uruchamiane co klatkę
        .add_systems(Update, (
            update_simulation_speed,
            move_satellites,
            update_protocol_logic,
            draw_gui,
        ))
        .run();
}

// --- 4. SYSTEM STARTOWY (SETUP) ---
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut sim_state: ResMut<SimulationState>,
) {
    // Ustawienia początkowe symulacji
    sim_state.sim_speed = 1.0;

    // Kamera 3D skierowana na środek układu
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 12.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Oświetlenie otoczenia (Space ambient)
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.2, 0.2, 0.4),
        brightness: 200.0,
    });

    // Główne światło kierunkowe (np. Słońce)
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 10.0),
        ..default()
    });

    // Ziemia (Sfera w centrum)
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(3.0).mesh().ico(5).unwrap()),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.1, 0.3, 0.6),
                roughness: 0.8,
                metallic: 0.1,
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Earth,
    ));

    // Generowanie konstelacji satelitów (np. 4 satelity na jednej orbicie)
    let num_satellites = 4;
    let radius = 6.0;
    for i in 0..num_satellites {
        let starting_angle = (i as f32) * (2.0 * PI / num_satellites as f32);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.2).mesh().ico(3).unwrap()),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.0, 1.0, 0.0), // Startowo zielone
                    emissive: Color::rgb(0.0, 0.2, 0.0),   // Delikatny efekt glow
                    ..default()
                }),
                ..default()
            },
            Satellite {
                id: i,
                orbit_radius: radius,
                current_angle: starting_angle,
                orbit_speed: 0.2, // prędkość kątowa
                ram_usage: 1.2,   // Bazowe LwIP zużycie
                cpu_load: 5.0,
                status_msg: "OPERATIONAL".to_string(),
            },
        ));
    }
}

// --- 5. SYSTEMY LOGIKI I RUCHU ---

fn update_simulation_speed(mut sim_state: ResMut<SimulationState>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Space) {
        sim_state.is_paused = !sim_state.is_paused;
    }
}

fn move_satellites(
    time: Res<Time>,
    sim_state: Res<SimulationState>,
    mut query: Query<(&mut Transform, &mut Satellite)>,
) {
    if sim_state.is_paused { return; }

    for (mut transform, mut satellite) in query.iter_mut() {
        // Aktualizacja kąta orbity na podstawie czasu delty i suwaka prędkości
        satellite.current_angle += satellite.orbit_speed * time.delta_seconds() * sim_state.sim_speed;
        if satellite.current_angle > 2.0 * PI {
            satellite.current_angle -= 2.0 * PI;
        }

        // Przeliczenie trygonometryczne pozycji na płaszczyźnie XZ (ruch wokół Ziemi)
        let x = satellite.orbit_radius * satellite.current_angle.cos();
        let z = satellite.orbit_radius * satellite.current_angle.sin();

        transform.translation = Vec3::new(x, 0.0, z);
    }
}

// Miejsce na Twoje zadanie domowe – modelowanie zachowania protokołów
fn update_protocol_logic(
    sim_state: Res<SimulationState>,
    mut query: Query<(&mut Satellite, &mut Handle<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut sat, mat_handle) in query.iter_mut() {
        if let Some(material) = materials.get_mut(mat_handle.id()) {
            match sim_state.current_protocol {
                RoutingProtocol::TerrestrialOSPF => {
                    // Symulacja błędu standardowego OSPF na orbicie
                    sat.ram_usage = 48.5; // OSPF LSDB puchnie
                    sat.cpu_load = 92.0;  // Ciągłe przeliczanie SPF na wolnym CPU
                    sat.status_msg = "CONVERGENCE TIMEOUT / OOM".to_string();
                    material.base_color = Color::rgb(1.0, 0.0, 0.0); // Czerwony alert
                }
                RoutingProtocol::SatnetOSPF => {
                    // Satnet działa stabilnie
                    sat.ram_usage = 2.1;
                    sat.cpu_load = 12.0;
                    sat.status_msg = "SATNET-OSPF OPTIMIZED".to_string();
                    material.base_color = Color::rgb(0.0, 1.0, 0.0); // Stabilny zielony
                }
                RoutingProtocol::ContactGraphRouting => {
                    sat.ram_usage = 12.4; // Buforowanie pakietów DTN
                    sat.cpu_load = 25.0;
                    sat.status_msg = "STORE & FORWARD ACTIVE".to_string();
                    material.base_color = Color::rgb(1.0, 0.5, 0.0); // Pomarańczowy tryb DTN
                }
                RoutingProtocol::CentralizedSDN => {
                    sat.ram_usage = 4.0;
                    sat.cpu_load = 8.0;
                    sat.status_msg = "SDN CONTROLLER DELAY".to_string();
                    material.base_color = Color::rgb(0.2, 0.6, 1.0); // Niebieski sygnał SDN
                }
            }
        }
    }
}

// --- 6. INTERFEJS UŻYTKOWNIKA (GUI VIA EGUI) ---
fn draw_gui(
    mut contexts: EguiContexts,
    mut sim_state: ResMut<SimulationState>,
    sat_query: Query<&Satellite>,
) {
    egui::SidePanel::left("control_panel")
        .default_width(300.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical_central(|ui| {
                ui.heading("🛰️ SPACE-ROUTER SIMULATOR");
            });
            ui.separator();

            ui.label("Wybierz protokół routingu:");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::TerrestrialOSPF, "Terrestrial OSPFv3 (RFC 5340)");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::SatnetOSPF, "⚡ SATNET-OSPF Framework");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::ContactGraphRouting, "DTN / Contact Graph Routing");
            ui.selectable_value(&mut sim_state.current_protocol, RoutingProtocol::CentralizedSDN, "Software-Defined Networking (SDN)");

            ui.separator();
            ui.label("Kontrola czasu:");
            ui.add(egui::Slider::new(&mut sim_state.sim_speed, 0.0..=5.0).text("Prędkość"));

            let pause_text = if sim_state.is_paused { "Wznów (Space)" } else { "Pauza (Space)" };
            if ui.button(pause_text).clicked() {
                sim_state.is_paused = !sim_state.is_paused;
            }

            ui.separator();
            ui.heading("Telemetria węzłów (Nodes)");

            // Wyświetlamy statystyki każdego satelity w panelu bocznym
            for sat in sat_query.iter() {
                ui.collapsing(format!("Satelita #{}", sat.id), |ui| {
                    ui.label(format!("Status: {}", sat.status_msg));
                    ui.label(format!("CPU Load: {:.1}%", sat.cpu_load));
                    ui.add(egui::ProgressBar::new(sat.cpu_load / 100.0).text("CPU"));
                    ui.label(format!("RAM Footprint: {:.2} MB / 64 MB", sat.ram_usage));
                });
            }
        });
}