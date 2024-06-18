
use echidna_util::get_app_resources;
use echidna_util::config::{Config, GroupBy};
use echidna_lib::{generate_shim_app, GenErr};

use std::sync::Arc;
use std::ffi::OsString;
use std::path::PathBuf;

use eframe::egui;
use egui::viewport::IconData;
use rfd::FileDialog;

const MIN_INNER_SIZE: (f32, f32) = (400.0, 200.0);
const GENERATE_PADDING: f32 = 15.0;

#[derive(Default)]
struct EchidnaApp {
    cmd: String,
    exts: String,
    group_by: GroupBy,
    previous_name: Option<OsString>,
}

impl EchidnaApp {
    fn new() -> Self {
        EchidnaApp {
            ..Default::default()
        }
    }
    
    fn generate_inner(&mut self) -> Result<(), String> {
        let mut dialog = FileDialog::new();
        if let Some(name) = &self.previous_name {
            // Shame to have to use to_string_lossy(), everwhere else, the filename is
            // an OsStr(ing). At least here the user has the chance  to fix it if it
            // gets mangled.
            dialog = dialog.set_file_name(name.to_string_lossy());
        }
        let Some(app_path) = dialog.save_file() else {
            return Ok(());
        };
        self.previous_name = app_path.file_name().map(|x| x.to_owned());

        let config = Config::new(self.cmd.clone(), self.group_by);
        let shim_path = get_shim_path()?;

        let res = generate_shim_app(
            &config,
            self.exts.clone(),
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
                    return Err(msg);
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
            self.exts.clone(),
            &shim_path,
            app_path.clone(),
            true,
        );

        let final_path = match res {
            Ok(x) => x,
            Err(GenErr::Other(msg)) => {
                return Err(msg);
            },
            Err(GenErr::AppAlreadyExists) => {
                return Err(format!("Still couldn't write destination '{}'", app_path.display()));
            },
        };

        if let Err(e) = opener::reveal(final_path) {
            eprintln!("{e}"); // No modal, failure here does no real harm.
        }

        Ok(())
    }

    fn generate(&mut self) {
        if let Err(e) = self.generate_inner() {
            show_modal(e);
        }
    }

}

impl eframe::App for EchidnaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new(0).num_columns(2).show(ui, |ui| {
                ui.label("Command:");
                let cmd = egui::TextEdit::singleline(&mut self.cmd);
                ui.add(cmd);
                ui.end_row();

                ui.label("Extensions:");
                let exts = egui::TextEdit::singleline(&mut self.exts)
                    .hint_text("Optional; see Readme");
                ui.add(exts);
                ui.end_row();

                ui.label("Open Files:");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.group_by, GroupBy::All, "Together");
                    ui.radio_value(&mut self.group_by, GroupBy::None, "Individually");
                });
            });

            ui.add_space(GENERATE_PADDING);

            if ui.button("Generate!").clicked() {
                if self.cmd.is_empty() {
                    show_modal("Command must not be empty".to_string());
                } else {
                    self.generate();
                }
            }
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
            .map_err(|e| format!("Failed to get current ext: {e}"))?;

        if !path.pop() {
            return Err(format!("Couldn't pop binary filename from path '{}' !?", shim_path.display()));
        }
        path.push("echidna-shim");
        path
    };

    if shim_path.exists() {
        return Ok(shim_path);
    }

    // TODO: argument or environment variable for non-standard uses.
    return Err("Can't find echidna-shim executable".to_owned());
}


fn show_modal(msg: String) {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Error")
        .set_description(msg)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

fn load_icon() -> Arc<IconData> {
    let image_ret = image::load_from_memory(include_bytes!("../app_files/icon.png"))
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

