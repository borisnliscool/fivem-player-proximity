use egui::{Color32, FontId, RichText};
use egui_overlay::{egui_backend, egui_window_glfw_passthrough, EguiOverlay};
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;

use crate::{MOVABLE, NEARBY_PLAYERS};

pub struct Overlay {}
impl EguiOverlay for Overlay {
    fn gui_run(
        &mut self,
        egui_context: &egui_backend::egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        let players = NEARBY_PLAYERS.lock().unwrap();
        let panel_frame = egui::Frame {
            inner_margin: 5.0.into(), // so the stroke is within the bounds
            ..Default::default()
        };

        let movable = *MOVABLE.lock().unwrap();

        glfw_backend.window.set_decorated(movable);

        // just some controls to show how you can use glfw_backend
        egui_backend::egui::CentralPanel::default()
            .frame(panel_frame)
            .show(egui_context, |ui| {
                ui.heading(
                    RichText::new(format!("Nearby Players ({})", players.len()))
                        .font(FontId::proportional(20.0))
                        .color(Color32::WHITE),
                );

                if movable {
                    ui.label("Press CTRL + SHIFT + HOME to remove borders.");
                } else {
                    // Convert players to a Vec and iterate over chunks of 25
                    let player_values: Vec<_> = players.values().collect();
                    ui.horizontal(|ui| {
                        for (index, chunk) in player_values.chunks(25).enumerate() {
                            ui.vertical(|ui| {
                                for player in chunk {
                                    ui.label(
                                        RichText::new(player.name.clone())
                                            .font(FontId::proportional(17.0))
                                            .color(Color32::RED),
                                    );
                                }
                            });
                        }
                    });
                }
            });

        glfw_backend.window.set_mouse_passthrough(!movable);
        egui_context.request_repaint();
    }
}
