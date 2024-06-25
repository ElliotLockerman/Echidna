
use echidna_lib::misc::{get_app_resources, DEFAULT_UTIS};
use echidna_lib::config::{Config, GroupBy, TerminalApp};
use echidna_lib::generate::{generate_shim_app, GenErr};
use echidna_lib::{term, bail, bailf};

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::ffi::OsString;
use std::path::PathBuf;

use eframe::egui;
use egui::viewport::IconData;
use rfd::FileDialog;
use egui_commonmark::{CommonMarkCache, commonmark_str};

const MIN_INNER_SIZE: (f32, f32) = (400.0, 180.0);
const MIN_HELP_INNER_SIZE: (f32, f32) = (400.0, 180.0);
const SECTION_PADDING: f32 = 15.0;

// Special value for terminal EchidnaApp::terminal indicating generic. Obviously, using the TerminalApp
// enum would be better, but it doesn't seem compatibel with egui.
const GENERIC: &str = "Generic";

#[derive(better_default::Default)]
struct EchidnaApp {
    cmd: String,

    #[default(DEFAULT_UTIS.to_string())]
    utis: String,

    group_by: GroupBy,

    #[default("com.example.YourAppName".to_owned())]
    bundle_id: String,
    ident_ever_changed: bool, // Disables setting default based on cmd

    default_file_name: String,
    previous_name: Option<OsString>, // Previous name chosen by Save As

    #[default(term::default_terminal().to_owned())]
    terminal: String,

    show_help: Arc<AtomicBool>,
    help_cache: Arc<Mutex<CommonMarkCache>>,

    #[default("".to_owned())]
    generic_terminal: String,
}

impl EchidnaApp {
    fn new() -> Self {
        EchidnaApp::default()
    }
    
    fn generate_inner(&mut self) -> Result<(), String> {
        if self.cmd.is_empty() {
            bail!("Command must not be empty".to_string());
        }

        if self.terminal == GENERIC && self.generic_terminal.is_empty() {
            return Err("Generic terminal must not be empty".to_string());
        }

        if self.bundle_id.is_empty() {
            bail!("Bundle Identifier must not be empty".to_string());
        }

        // Shame to have to use to_string_lossy(), everwhere else, the filename is
        // an OsStr(ing). At least here the user has the chance  to fix it if it
        // gets mangled.
        let dialog = FileDialog::new()
            .set_file_name(
                self.previous_name.as_ref()
                    .map(|x| x.to_string_lossy().to_string())
                    .unwrap_or(self.default_file_name.clone())
            );

        let Some(app_path) = dialog.save_file() else {
            return Ok(());
        };
        self.previous_name = app_path.file_name().map(|x| x.to_owned());

        let terminal = if self.terminal == GENERIC {
            TerminalApp::Generic(self.generic_terminal.clone())
        } else {
            TerminalApp::Supported(self.terminal.clone())
        };
        let config = Config{
            command: self.cmd.clone(),
            group_open_by: self.group_by,
            terminal,
        };
        let shim_path = get_shim_path()?;

        let res = generate_shim_app(
            &config,
            self.utis.clone(),
            &self.bundle_id,
            &shim_path,
            app_path.clone(),
            false,
        );

        match res {
            Ok(final_path) => {
                if let Err(e) = opener::reveal(final_path) {
                    eprintln!("{e}"); // No modal, failure here does no real harm.
                }
                return Ok(());
            },
            Err(err) => match err {
                GenErr::Other(msg) => {
                    bail!(msg);
                },
                GenErr::AppAlreadyExists => (),
            }
        }

        // Couldn't write app because one already exists. Give user a chance to ovewrite.
        let result = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Error: Destination already exists")
            .set_description(format!("Destination '{}' already exists. Overwrite?", app_path.display()))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        if result == rfd::MessageDialogResult::No {
            return Ok(());
        }

        let res = generate_shim_app(
            &config,
            self.utis.clone(),
            &self.bundle_id,
            &shim_path,
            app_path.clone(),
            true,
        );

        let final_path = match res {
            Ok(x) => x,
            Err(GenErr::Other(msg)) => bail!(msg),
            Err(GenErr::AppAlreadyExists) => {
                bailf!("Still couldn't write destination '{}'", app_path.display());
            },
        };

        if let Err(e) = opener::reveal(final_path) {
            eprintln!("{e}"); // No modal, failure here does no real harm.
        }

        Ok(())
    }

    fn generate(&mut self) {
        if let Err(e) = self.generate_inner() {
            modal(e);
        }
    }

    fn update_default_file_name(&mut self) {

        let word = self.cmd.split_whitespace().next();
        let word = match word {
            Some(x) => x,
            None => {
                self.bundle_id += "YourAppName";
                return;
            },
        };

        let mut chars = word.chars();
        let mut first = chars.next().expect("SplitWhitespace returned an empty string!?");
        first.make_ascii_uppercase();

        self.default_file_name.clear();
        self.default_file_name.push(first);
        self.default_file_name.extend(chars);
        self.default_file_name += "Opener";
    }

    fn update_default_ident(&mut self) {
        self.bundle_id.clear();
        self.bundle_id += "com.yourdomain.";
        self.bundle_id += &self.default_file_name;
    }

    fn draw_help(&self, ctx: &egui::Context) {
        if !self.show_help.load(Ordering::Relaxed) {
            return;
        }

        let vb = egui::viewport::ViewportBuilder::default()
            .with_title("Help")
            .with_min_inner_size(MIN_HELP_INNER_SIZE);
        let vid = egui::viewport::ViewportId::from_hash_of("help window");
        let show_help = self.show_help.clone();
        let help_cache = self.help_cache.clone();
        ctx.show_viewport_deferred(vid, vb, move |ctx, _| {
            egui::CentralPanel::default().show(ctx, |ui| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    show_help.store(false, Ordering::Relaxed);
                    return;
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    commonmark_str!("help", ui, &mut *help_cache.lock().unwrap(), "src/app/help.md");
                });
            });
        });
    }
}

impl eframe::App for EchidnaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("Form").num_columns(2).show(ui, |ui| {
                ui.label("Command:");
                ui.centered_and_justified(|ui| {
                    let cmd = egui::TextEdit::singleline(&mut self.cmd);
                    if ui.add(cmd).changed() {
                        self.update_default_file_name();
                    }
                });
                ui.end_row();

                ui.label("Uniform Type Identifiers:");
                ui.centered_and_justified(|ui| {
                    ui.text_edit_singleline(&mut self.utis);
                });
                ui.end_row();

                ui.label("Bundle Identifier:");
                ui.centered_and_justified(|ui| {
                    if !self.ident_ever_changed {
                        self.update_default_ident();
                    }
                    if ui.text_edit_singleline(&mut self.bundle_id).changed() {
                        self.ident_ever_changed = true;
                    }
                });
                ui.end_row();

                ui.end_row(); // Spacer

                ui.label("Terminal:");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_source("Terminal Combo Box")
                        .selected_text(&self.terminal)
                        .show_ui(ui, |ui| {
                            for terminal in term::supported_terminals() {
                                assert!(terminal != GENERIC);
                                if ui.selectable_label(self.terminal == terminal, terminal).clicked() {
                                     terminal.clone_into(&mut self.terminal);
                                }
                            }

                            if ui.selectable_label(self.terminal == GENERIC, GENERIC).clicked() {
                                GENERIC.clone_into(&mut self.terminal);
                            }
                    });


                    if self.terminal == GENERIC {
                        let generic = egui::TextEdit::singleline(&mut self.generic_terminal)
                            .hint_text("Terminal App Name");
                        ui.add(generic);
                    }
                });
                ui.end_row();

                ui.label("Open Files:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.group_by, GroupBy::All, "Together");
                    ui.radio_value(&mut self.group_by, GroupBy::None, "Individually");
                });
                ui.end_row();

            });

            ui.add_space(SECTION_PADDING);

            ui.horizontal(|ui| {
                if ui.button("Save As..").clicked() {
                    self.generate();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("?").clicked() {
                        self.show_help.store(true, Ordering::Relaxed);
                    }
                });
            });

            self.draw_help(ctx);
        });
    }
}

////////////////////////////////////////////////////////////////////////////////

fn get_shim_path() -> Result<PathBuf, String> {
    // Standard location: the app bundle's Resources folder.
    let shim_path = {
        let mut rsc = get_app_resources()?;
        rsc.push("echidna-shim");
        rsc
    };

    if shim_path.exists() {
        return Ok(shim_path);
    }

    if shim_path.exists() {
        return Ok(shim_path);
    }

    // Maybe they're running the executable directly from the targets/ directory? Then echidna-shim
    // will be right there.
    let shim_path = {
        let mut path = std::env::current_exe()
            .map_err(|e| format!("Failed to get current exe: {e}"))?;

        if !path.pop() {
            bailf!("Couldn't pop binary filename from path '{}' !?", shim_path.display());
        }
        path.push("echidna-shim");
        path
    };

    if shim_path.exists() {
        return Ok(shim_path);
    }

    // TODO: argument or environment variable for non-standard uses.
    Err("Can't find echidna-shim executable".to_owned())
}


fn modal<S: Into<String>>(msg: S) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Error")
        .set_description(msg)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

fn load_icon() -> Arc<IconData> {
    let image_ret = image::load_from_memory(include_bytes!("../../app_files/icon.png"))
        .map(|x| x.into_rgb8());

    let image = match image_ret {
        Ok(x) => x,
        Err(_) => {
            // TODO: logging
            return std::sync::Arc::new(egui::viewport::IconData::default());
        },
    };

    let (width, height) = image.dimensions();
    let data = IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    Arc::new(data)
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(MIN_INNER_SIZE)
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Echidna",
        options,
        Box::new(|_cc| {
            Box::new(EchidnaApp::new())
        }),
    )
}

