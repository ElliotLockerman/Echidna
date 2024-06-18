
use echidna_util::get_app_resources;
use echidna_util::config::{Config, GroupBy};
use echidna_lib::{generate_shim_app, GenErr};

use std::sync::Arc;
use std::ffi::OsString;

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
    
    fn generate(&mut self) {
        let mut dialog = FileDialog::new();
        if let Some(name) = &self.previous_name {
            // Shame to have to use to_string_lossy(), everwhere else, the filename is
            // an OsStr(ing). At least here the user has the chance  to fix it if it
            // gets mangled.
            dialog = dialog.set_file_name(name.to_string_lossy());
        }
        let Some(app_path) = dialog.save_file() else {
            return;
        };
        self.previous_name = app_path.file_name().map(|x| x.to_owned());

        let config = Config::new(self.cmd.clone(), self.group_by);
        let shim_path = match get_app_resources() {
            Ok(x) => x.join("echidna-shim"),
            Err(e) => {
                show_modal(e);
                return;
            }
        };

        let res = generate_shim_app(
            &config,
            self.exts.clone(),
            &shim_path,
            app_path.clone(),
            false,
        );

        match res {
            Ok(()) => return,
            Err(err) => match err {
                GenErr::Other(msg) => {
                    show_modal(msg);
                    return;
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
            return;
        }

        let res = generate_shim_app(
            &config,
            self.exts.clone(),
            &shim_path,
            app_path.clone(),
            true,
        );

        match res {
            Ok(()) => return,
            Err(GenErr::Other(msg)) => {
                show_modal(msg);
                return;
            },
            Err(GenErr::AppAlreadyExists) =>
                show_modal(format!("Still couldn't write destination '{}'", app_path.display())),
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

