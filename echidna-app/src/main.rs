
use echidna_helpers::config::{Config, GroupBy};

use std::path::PathBuf;

use eframe::egui;
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
    
    fn generate(&self, mut path: PathBuf) -> Result<(), String> {

        let file_name = match path.file_stem() {
            Some(x) => x.to_owned(),
            None => return Err("No file name given".to_owned()),
        };
        let file_name = match file_name.to_str() {
            Some(x) => x.to_owned(),
            None => return Err("File name must be valid unicode".to_string()),
        };

        path.pop();

        let config = Config::new(self.cmd.clone(), self.group_by);

        echidna_lib::generate_shim_app(
            file_name,
            &config,
            self.exts.clone(),
            path.to_owned(),
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

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size(MIN_INNER_SIZE),
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

