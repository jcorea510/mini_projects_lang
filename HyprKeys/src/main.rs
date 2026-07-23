mod theme;
use eframe::egui::RichText;
use theme::catppuccin::*;
use eframe::egui;
use egui::{Visuals, style::{WidgetVisuals, Widgets}, Stroke};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("HyprKeys")
            .with_app_id("HyprKeys"),
        ..Default::default()
    };
    eframe::run_native(
        "HyprKeys",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc))))
    )
}


#[derive(Default)]
struct BindRow {
    combo: String,
    action: String,
}

#[derive(Default)]
struct MyEguiApp {
    commands_by_submap: HashMap<String, Vec<BindRow>>,
    search_text: String,
}

impl MyEguiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        set_catppuccin_theme(&cc.egui_ctx);
        let commands = load_binds();

        Self {
            commands_by_submap: commands,
            search_text: String::new(),
        }
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(
                RichText::new("Hyprland keybindings")
                    .heading()
                    .strong()
                    .color(GREEN)
            );

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Search keybindings: ")
                        .strong()
                        .color(GREEN)
                );
                ui.text_edit_singleline(&mut self.search_text);
                if ui.button("Refresh").clicked() {
                    self.commands_by_submap = load_binds();
                }
            });

            let search = self.search_text.to_lowercase();
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut submaps: Vec<_> = self.commands_by_submap.keys().collect();
                submaps.sort();

                for submap in submaps {
                    let rows = &self.commands_by_submap[submap];
                    let filtered: Vec<_> = rows
                        .iter()
                        .filter(|row| {
                            search.is_empty()
                                || row.combo.to_lowercase().contains(&search)
                                || row.action.to_lowercase().contains(&search)
                        })
                        .collect();

                    if !filtered.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("Submap {submap}"))
                                .strong()
                                .color(YELLOW)
                        );

                        egui::Grid::new(format!("grid_{submap}"))
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .min_col_width(ui.available_width() * 0.3)
                            .show(ui, |ui| {
                                for row in filtered {
                                    ui.label(RichText::new(&row.combo).color(BLUE).strong());
                                    ui.label(&row.action);
                                    ui.end_row();
                                }
                            });

                        ui.add_space(10.0);
                    }
                }
            });
        });
    }
}

/// Mirrors the fields hyprctl -j binds emits per bind.
/// Not every field is used for display, but they're kept so
/// serde has somewhere to put them (and so this is easy to extend).
#[derive(Debug, Deserialize)]
struct HyprBind {
    #[serde(default)]
    mouse: bool,
    #[serde(default)]
    release: bool,
    #[serde(default)]
    repeat: bool,
    #[serde(default)]
    has_description: bool,
    #[serde(default)]
    modmask: u32,
    #[serde(default)]
    submap: String,
    #[serde(default)]
    key: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    dispatcher: String,
    #[serde(default)]
    arg: String,
}

/// Decodes Hyprland's modmask bitfield into readable modifier names.
/// Bit values match wlroots/xkb: SHIFT=1, CAPS=2, CTRL=4, ALT=8,
/// MOD2=16, MOD3=32, SUPER=64, MOD5=128.
fn decode_modmask(modmask: u32) -> String {
    let mods = [
        (1u32, "Shift"),
        (2, "Caps"),
        (4, "Ctrl"),
        (8, "Alt"),
        (16, "Mod2"),
        (32, "Mod3"),
        (64, "Super"),
        (128, "Mod5"),
    ];

    mods.iter()
        .filter(|(bit, _)| modmask & bit != 0)
        .map(|(_, name)| *name)
        .collect::<Vec<_>>()
        .join(" + ")
}

/// Builds a human-readable line for a bind that has no description,
/// e.g. "Super + Shift + Q  ->  killactive ()"
fn fallback_line(bind: &HyprBind) -> String {
    let mods = decode_modmask(bind.modmask);
    let combo = if mods.is_empty() {
        bind.key.clone()
    } else {
        format!("{mods} + {}", bind.key)
    };

    let action = if bind.arg.is_empty() {
        bind.dispatcher.clone()
    } else {
        format!("{} ({})", bind.dispatcher, bind.arg)
    };

    let flags = [
        (bind.mouse, "mouse"),
        (bind.release, "on release"),
        (bind.repeat, "repeat"),
    ]
    .iter()
    .filter(|(active, _)| *active)
    .map(|(_, name)| *name)
    .collect::<Vec<_>>()
    .join(", ");

    if flags.is_empty() {
        format!("{combo}  ->  {action}")
    } else {
        format!("{combo}  ->  {action}  [{flags}]")
    }
}

/// Runs `hyprctl -j binds`, parses the JSON, and groups the results
/// by submap. This replaces the old .conf-file parser: it works
/// regardless of whether your keybinds are defined in Hyprland's
/// native config syntax or in Lua, since it reads Hyprland's own
/// runtime bind table instead of scraping your config files.
fn load_binds() -> HashMap<String, Vec<BindRow>> {
    let mut commands_by_submap: HashMap<String, Vec<BindRow>> = HashMap::new();

    let output = match Command::new("hyprctl").args(["-j", "binds"]).output() {
        Ok(out) => out,
        Err(_) => return commands_by_submap,
    };

    if !output.status.success() {
        return commands_by_submap;
    }

    let binds: Vec<HyprBind> = match serde_json::from_slice(&output.stdout) {
        Ok(b) => b,
        Err(_) => return commands_by_submap,
    };

    for bind in binds {
        let submap = if bind.submap.is_empty() {
            "global".to_string()
        } else {
            bind.submap.clone()
        };

        let mods = decode_modmask(bind.modmask);
        let combo = if mods.is_empty() {
            bind.key.clone()
        } else {
            format!("{mods} + {}", bind.key)
        };

        let action = if bind.has_description && !bind.description.is_empty() {
            bind.description.clone()
        } else if bind.arg.is_empty() {
            bind.dispatcher.clone()
        } else {
            format!("{} ({})", bind.dispatcher, bind.arg)
        };

        commands_by_submap
            .entry(submap)
            .or_default()
            .push(BindRow { combo, action });
    }

    commands_by_submap
}

fn set_catppuccin_theme(ctx: &egui::Context) {
    let visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(TEXT),
        window_fill: BASE,
        panel_fill: MANTLE,
        faint_bg_color: CRUST,
        extreme_bg_color: CRUST,
        ..Visuals::dark()
    };

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;

    style.visuals.widgets = Widgets {
        noninteractive: WidgetVisuals {
            bg_fill: BASE,
            weak_bg_fill: BASE,
            bg_stroke: Stroke::new(0.0, BASE),
            fg_stroke: Stroke::new(1.0, SUBTEXT),
            corner_radius: egui::CornerRadius::from(0),
            expansion: 0.0,
        },
        inactive: WidgetVisuals {
            bg_fill: SURFACE,
            weak_bg_fill: SURFACE,
            bg_stroke: Stroke::new(1.0, OVERLAY),
            fg_stroke: Stroke::new(1.0, TEXT),
            corner_radius: egui::CornerRadius::from(4),
            expansion: 0.0,
        },
        hovered: WidgetVisuals {
            bg_fill: OVERLAY,
            weak_bg_fill: OVERLAY,
            bg_stroke: Stroke::new(1.0, PEACH),
            fg_stroke: Stroke::new(1.5, PEACH),
            corner_radius: egui::CornerRadius::from(6),
            expansion: 1.0,
        },
        active: WidgetVisuals {
            bg_fill: MAUVE,
            weak_bg_fill: MAUVE,
            bg_stroke: Stroke::new(1.5, BLUE),
            fg_stroke: Stroke::new(2.0, CRUST),
            corner_radius: egui::CornerRadius::from(6),
            expansion: 1.0,
        },
        open: WidgetVisuals {
            bg_fill: SURFACE,
            weak_bg_fill: SURFACE,
            bg_stroke: Stroke::new(1.0, LAVENDER),
            fg_stroke: Stroke::new(1.0, TEXT),
            corner_radius: egui::CornerRadius::from(6),
            expansion: 1.0,
        },
    };
    ctx.set_style(style);
}
