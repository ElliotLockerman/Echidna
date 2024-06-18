
use echidna_util::get_app_resources;
use echidna_util::config::{Config, GroupBy};

use std::path::PathBuf;
use std::sync::Arc;

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
}

impl EchidnaApp {
    fn new() -> Self {
        EchidnaApp {
            ..Default::default()
        }
    }
    
    fn generate(&self, app_path: PathBuf) -> Result<(), String> {
        let config = Config::new(self.cmd.clone(), self.group_by);
        let shim_path = get_app_resources()?.join("echidna-shim");

        echidna_lib::generate_shim_app(
            &config,
            self.exts.clone(),
            &shim_path,
            app_path,
        )?;

        Ok(())
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
                    show_modal("Command may not be empty".to_string());
                } else {
                    if let Some(path) = FileDialog::new().save_file() {
                        match self.generate(path) {
                            Ok(()) => (),
                            Err(e) => show_modal(e),
                        }
                    }
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

