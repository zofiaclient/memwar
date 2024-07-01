#![windows_subsystem = "windows"]

use std::sync::mpsc::TryRecvError;

use eframe::egui::{CentralPanel, Color32, RichText, ScrollArea, Ui};
use eframe::emath::Vec2;
use eframe::epaint::Shadow;
use notan::app::{App, Graphics, Plugins};
use notan::AppState;
use notan::draw::DrawConfig;
use notan::egui::{Button, EguiConfig, EguiPluginSugar, Frame, Margin, menu, Rounding, Stroke};
use notan::prelude::WindowConfig;

use memwar::tasks::ReceiverTask;

mod entity;
mod game;
mod pointers;
mod tasks;

const PALETTE_TEXT: Color32 = Color32::from_rgb(255, 82, 125);
const PALETTE_BACKGROUND: Color32 = Color32::from_rgb(40, 20, 30);
const PALETTE_DARK_BACKGROUND: Color32 = Color32::from_rgb(28, 13, 24);
const PALETTE_IMPORTANT: Color32 = Color32::from_rgb(255, 0, 98);

#[derive(Default)]
struct AimbotConsole {
    messages: Vec<String>,
}

impl AimbotConsole {
    fn to_ui(&self, ui: &mut Ui) {
        let frame = Frame {
            fill: PALETTE_BACKGROUND,
            inner_margin: Margin::same(15.0),
            stroke: Stroke::new(1.0, PALETTE_IMPORTANT),
            rounding: Rounding::same(5.0),
            shadow: Shadow::default(),
            ..Default::default()
        };
        
        frame.show(ui, |ui| {
            ScrollArea::both().max_height(100.0).show(ui, |ui| {
                ui.heading(RichText::new("Console Output").color(PALETTE_TEXT));

                if self.messages.is_empty() {
                    ui.label("Awaiting console output..");
                }
                
                for message in &self.messages {
                    ui.code(RichText::new(message).color(PALETTE_IMPORTANT));
                }
            });
        });
    }

    fn add(&mut self, message: String) {
        if self.messages.len() > 10 {
            self.messages = Vec::new();
        }
        self.messages.push(message);
    }
}

#[derive(AppState)]
struct State {
    aimbot_task: ReceiverTask<String, String>,
    console: AimbotConsole,
}

impl State {
    fn new() -> Self {
        Self {
            aimbot_task: tasks::new_aimbot_task(),
            console: AimbotConsole::default(),
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

            ui.vertical_centered(|ui| {
                ui.heading(RichText::new("AssaultCube Aimbot").color(PALETTE_TEXT));
                ui.label("Written by Zofia");
                
                ui.separator();
                state.console.to_ui(ui);
                
                if ui
                    .add(Button::new("Toggle Aimbot").fill(PALETTE_BACKGROUND))
                    .on_hover_ui(|ui| {
                        ui.heading(RichText::new("Aimbot").color(PALETTE_TEXT));
                        ui.label("When enabled, hold F to use the aimbot.");
                    })
                    .clicked()
                {
                    state.aimbot_task.toggle_enabled();
                }

                if state.aimbot_task.is_enabled() {
                    ui.label("Aimbot is enabled. Hold F ingame to use the aimbot.");
                } else {
                    ui.label("Aimbot is disabled. Hold F ingame to use the aimbot.");
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
                        state.console.add("Thread disconnected".to_string());

                        if ui.button("Restart thread").clicked() {
                            state.aimbot_task = tasks::new_aimbot_task();
                        }
                    }
                };

                match state.aimbot_task.read_error() {
                    Ok(err) => {
                        state.console.add(format!("Aimbot error: {err}"));
                    }
                    Err(TryRecvError::Disconnected) => {
                        ui.colored_label(Color32::RED, "Thread disconnected");
                    }
                    Err(TryRecvError::Empty) => (),
                }
            });
        });
    });
    gfx.render(&output);
}

fn main() -> Result<(), String> {
    let wnd_cfg = WindowConfig::new()
        .set_title("AssaultCube Aimbot")
        .set_vsync(true);

    notan::init_with(State::new)
        .add_config(wnd_cfg)
        .add_config(EguiConfig)
        .add_config(DrawConfig)
        .draw(draw)
        .build()
}
