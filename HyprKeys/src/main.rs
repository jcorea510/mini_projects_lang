use std::fs;
use std::env;
use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "HyprKeys",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc))))
    )
}

#[derive(Default)]
struct MyEguiApp {
    commands_by_submap: HashMap<String, Vec<String>>,
    search_text: String,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        set_catppuccin_theme(&cc.egui_ctx);
        let commands = load_config_commands();

        Self {
            commands_by_submap: commands,
            search_text: String::new(),
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hyprland keybindings");
           
            ui.label("Search keybindings: ");
            ui.text_edit_singleline(&mut self.search_text);
            
            let search = self.search_text.to_lowercase();
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (submap, command) in &self.commands_by_submap {
                    let filtered: Vec<_> = command
                        .iter()
                        .filter(|cmd| {
                            search.is_empty() || cmd.to_lowercase().contains(&search)
                        })
                        .collect();

                    if !filtered.is_empty() {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new(format!("Submap {submap}")).strong());

                            for cmd in filtered {
                                ui.label(cmd);
                            }
                        });
                        ui.add_space(10.0);
                    }
                }
            });
        });
    }
}

fn load_config_commands()->HashMap<String, Vec<String>> {
    let mut commands_by_sumap: HashMap<String, Vec<String>> = HashMap::new();
    let config_dir = env::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".config/hypr/keybindings");

    let mut current_submap = "default".to_string();

    for entry in WalkDir::new(&config_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "conf")
                .unwrap_or(false)
        })
    {
        if let Ok(contents) = fs::read_to_string(entry.path()) {
            for line in contents.lines() {
                let trimmed_line = line.trim();
                if trimmed_line.starts_with("submap =") {
                    let name = trimmed_line.strip_prefix("submap =").unwrap().trim();
                    if name != "reset" {
                        current_submap = name.to_string();
                    }
                    continue;
                }
                if trimmed_line.contains("escape") || trimmed_line.contains("submap, reset") {
                    continue;
                }
                if trimmed_line.starts_with("bind =") {
                    commands_by_sumap
                        .entry(current_submap.clone())
                        .or_default()
                        .push(trimmed_line.to_string());
                }
            }
        }
    }
    commands_by_sumap
}

fn set_catppuccin_theme(ctx: &egui::Context) {
    use egui::{Color32};

    // Catppuccin Mocha (puedes ajustar con Macchiato o Latte si deseas)
    // let rosewater = Color32::from_rgb(245, 224, 220);
    // let flamingo = Color32::from_rgb(242, 205, 205);
    // let pink = Color32::from_rgb(245, 194, 231);
    // let green = Color32::from_rgb(166, 227, 161);
    // let teal = Color32::from_rgb(148, 226, 213);
    // let sky = Color32::from_rgb(137, 220, 235);
    // let sapphire = Color32::from_rgb(116, 199, 236);
    // let red = Color32::from_rgb(243, 139, 168);
    // let maroon = Color32::from_rgb(235, 160, 172);
    // let yellow = Color32::from_rgb(249, 226, 175);
    let mauve = Color32::from_rgb(203, 166, 247);
    let peach = Color32::from_rgb(250, 179, 135);
    let blue = Color32::from_rgb(137, 180, 250);
    let lavender = Color32::from_rgb(180, 190, 254);

    let text = Color32::from_rgb(205, 214, 244);
    let subtext = Color32::from_rgb(166, 173, 200);
    let base = Color32::from_rgb(30, 30, 46);
    let mantle = Color32::from_rgb(24, 24, 37);
    let crust = Color32::from_rgb(17, 17, 27);
    let surface = Color32::from_rgb(49, 50, 68);
    let overlay = Color32::from_rgb(76, 79, 105);

    use egui::{Visuals, style::{WidgetVisuals, Widgets}, Stroke};
    let visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(text),
        window_fill: base,
        panel_fill: mantle,
        faint_bg_color: crust,
        extreme_bg_color: crust,
        ..Visuals::dark()
    };

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;

    style.visuals.widgets = Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: base,
            weak_bg_fill: base,
            bg_stroke: Stroke::new(0.0, base),
            fg_stroke: Stroke::new(1.0, subtext),
            corner_radius: egui::CornerRadius::from(0),
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            bg_fill: surface,
            weak_bg_fill: surface,
            bg_stroke: Stroke::new(1.0, overlay),
            fg_stroke: Stroke::new(1.0, text),
            corner_radius: egui::CornerRadius::from(4),
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            bg_fill: overlay,
            weak_bg_fill: overlay,
            bg_stroke: Stroke::new(1.0, peach),
            fg_stroke: Stroke::new(1.5, peach),
            corner_radius: egui::CornerRadius::from(6),
            expansion: 1.0,
        },
        active: WidgetVisuals {
            bg_fill: mauve,
            weak_bg_fill: mauve,
            bg_stroke: Stroke::new(1.5, blue),
            fg_stroke: Stroke::new(2.0, crust),
            corner_radius: egui::CornerRadius::from(6),
            expansion: 1.0,
        },
        open: WidgetVisuals {
            bg_fill: surface,
            weak_bg_fill: surface,
            bg_stroke: Stroke::new(1.0, lavender),
            fg_stroke: Stroke::new(1.0, text),
            corner_radius: egui::CornerRadius::from(6),
            expansion: 1.0,
        },
    };
    ctx.set_style(style);
}
