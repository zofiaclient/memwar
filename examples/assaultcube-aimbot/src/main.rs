#![windows_subsystem = "windows"]

use std::sync::mpsc::TryRecvError;

use eframe::egui::{CentralPanel, Color32, RichText, Stroke};
use eframe::emath::Vec2;
use notan::app::{App, Graphics, Plugins};
use notan::AppState;
use notan::draw::DrawConfig;
use notan::egui::{EguiConfig, EguiPluginSugar, Frame, Margin, menu};
use notan::prelude::WindowConfig;

use memwar::tasks::ReceiverTask;

mod entity;
mod game;
mod offsets;
mod tasks;

const PALETTE_TEXT: Color32 = Color32::from_rgb(255, 82, 125);
const PALETTE_BACKGROUND: Color32 = Color32::from_rgb(40, 20, 30);
const PALETTE_DARK_BACKGROUND: Color32 = Color32::from_rgb(28, 13, 24);
const PALETTE_IMPORTANT: Color32 = Color32::from_rgb(255, 0, 98);

#[derive(AppState)]
struct State {
    aimbot_task: ReceiverTask<String, String>,
    last_error: Option<String>,
}

impl State {
    fn new() -> Self {
        Self {
            aimbot_task: tasks::new_aimbot_task(),
            last_error: None,
        }
    }
}

fn draw(app: &mut App, gfx: &mut Graphics, plugins: &mut Plugins, state: &mut State) {
    let frame = Frame {
        fill: PALETTE_DARK_BACKGROUND,
        inner_margin: Margin::same(15.0),
        ..Default::default()
    };

    let output = plugins.egui(|ctx| {
        CentralPanel::default().frame(frame).show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(30.0, 20.0);
            ui.spacing_mut().button_padding = Vec2::new(15.0, 15.0);
            
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.menu_button("Quit", |ui| {
                        ui.label("Do you really wish to quit?");

                        if ui.button("Confirm").clicked() {
                            app.exit();
                        }
                        
                        if ui.button("Cancel").clicked() {
                            ui.close_menu();
                        }
                    });
                });

                ui.menu_button("Help", |ui| {
                    ui.heading(RichText::new("AssaultCube Aimbot").color(PALETTE_TEXT));

                    ui.label("Press Toggle Aimbot and enter AssaultCube.");
                    ui.label("Hold F ingame to use the aimbot.");

                    ui.separator();
                    ui.hyperlink_to("GitHub", "https://github.com/zofiaclient/memwar")
                        .on_hover_text("This program was written with memwar.");
                });
            });

            Frame::default().fill(PALETTE_BACKGROUND).inner_margin(30.0).stroke(Stroke::new(1.0, PALETTE_IMPORTANT))
                .show(ui, |ui| {
                    ui.set_min_height(ui.available_height());
                    
                    ui.vertical_centered(|ui| {
                        ui.heading(RichText::new("AssaultCube Aimbot").color(PALETTE_TEXT));
                        ui.label("Written by Zofia");

                        ui.separator();

                        if let Some(err) = &state.last_error {
                            ui.colored_label(PALETTE_IMPORTANT, err);
                        }

                        let mut enabled = state.aimbot_task.is_enabled();

                        ui.checkbox(&mut enabled, "");
                        
                        state.aimbot_task.set_enabled(enabled);

                        if state.aimbot_task.is_enabled() {
                            ui.label("Aimbot is enabled. Hold F ingame to use the aimbot.");
                        } else {
                            ui.label("Aimbot is disabled. While enabled, hold F ingame to use the aimbot.");
                        }

                        match state.aimbot_task.try_recv_data() {
                            Ok(entity_name) => {
                                ui.colored_label(
                                    Color32::LIGHT_GREEN,
                                    format!("Aimed at player `{}`", entity_name),
                                );
                            }
                            Err(TryRecvError::Empty) => {
                                ui.label("Awaiting aimbot response..");
                            }
                            Err(TryRecvError::Disconnected) => {
                                state.last_error = Some("Aimbot thread disconnected".to_string());

                                if ui.button("Restart thread").clicked() {
                                    state.aimbot_task = tasks::new_aimbot_task();
                                }
                            }
                        };

                        match state.aimbot_task.read_error() {
                            Ok(err) => {
                                state.last_error = Some(err);
                            }
                            Err(TryRecvError::Disconnected) => {
                                ui.colored_label(Color32::RED, "Thread disconnected");
                            }
                            Err(TryRecvError::Empty) => (),
                        }
                    });
                });
        });
    });
    gfx.render(&output);
}

fn main() -> Result<(), String> {
    let wnd_cfg = WindowConfig::new()
        .set_title("AssaultCube Aimbot");

    notan::init_with(State::new)
        .add_config(wnd_cfg)
        .add_config(EguiConfig)
        .add_config(DrawConfig)
        .draw(draw)
        .build()
}
