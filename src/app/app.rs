use echidna_lib::config::{Config, GroupBy, TerminalApp};
use echidna_lib::generate;
use echidna_lib::generate::{Generator, SaveErr};
use echidna_lib::misc::get_app_resources;
use echidna_lib::{bail, bailf, term};

use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use eframe::egui;
use egui::load::Bytes;
use egui::viewport::IconData;
use egui::widgets::ImageSource;
use egui::Grid;
use egui_commonmark::{commonmark_str, CommonMarkCache};
use icns::IconFamily;
use lazy_static::lazy_static;

// All eyeballed.
const INNER_HEIGHT: f32 = 230.0;
const MIN_INNER_SIZE: (f32, f32) = (500.0, INNER_HEIGHT);
const MAX_INNER_SIZE: (f32, f32) = (650.0, INNER_HEIGHT);
const MIN_HELP_INNER_SIZE: (f32, f32) = (400.0, 180.0);
const WINDOW_PADDING: f32 = 20.0;
const SECTION_SPACING: f32 = 20.0;
const THUMBNAIL_SIZE: (f32, f32) = (128.0, 128.0);

// Special value for terminal EchidnaApp::terminal indicating generic. Obviously, using the TerminalApp
// enum would be better, but it doesn't seem compatibel with egui.
const GENERIC: &str = "Generic";

const TERM_KEY: &str = "TERM_KEY";
const GENERIC_TERM_KEY: &str = "GENERIC_TERM_KEY";
const GROUP_BY_KEY: &str = "GROUP_BY_KEY";

const DEFAULT_APP_NAME: &str = "YourAppName";

const DEFAULT_SHIM_ICON_THUMB: ImageSource<'static> =
    egui::include_image!("../../app_files/shim_icon_256.png");

#[derive(Default, Clone, PartialEq, Eq)]
enum DocTypes {
    #[default]
    TextFiles,
    AllDocs,
    UTIs,
    Exts,
}

impl DocTypes {
    fn display_name(&self) -> &str {
        match self {
            Self::TextFiles => "Text Files",
            Self::AllDocs => "All Documents",
            Self::UTIs => "Specific UTIs:",
            Self::Exts => "Specific Extensions:",
        }
    }
}

lazy_static! {
    pub static ref SUPPORTED_EXTS: &'static [&'static str] = &[
        "jpg", "jpeg", "avif", "avif", "bmp", "dds", "exr", "gif", "hdr", "ico", "png", "pnm",
        "qoi", "tga", "tif", "tiff", "webp", "icns",
    ];
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Image {
    path: PathBuf,
    buffer: Bytes,
}

impl Image {
    const PNG_MAGIC: &'static [u8] = &[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

    pub fn new(path: PathBuf, buffer: Vec<u8>) -> Image {
        Image {
            path,
            buffer: Bytes::from(buffer),
        }
    }

    fn is_png(path: &Path) -> Result<bool, String> {
        let mut file =
            File::open(path).map_err(|e| format!("Error opening '{}': {e}", path.display()))?;

        let buf = &mut [0u8; Self::PNG_MAGIC.len()];
        let num = file
            .read(&mut buf[..])
            .map_err(|e| format!("Error reading '{}': {e}", path.display()))?;
        if num != Self::PNG_MAGIC.len() {
            bailf!("Unexpected EOF reading '{}'", path.display());
        }

        Ok(buf == Self::PNG_MAGIC)
    }

    fn load_icns(path: PathBuf) -> Result<Image, String> {
        assert_eq!(path.extension(), Some(OsStr::new("icns")));
        let file =
            File::open(&path).map_err(|e| format!("Error opening '{}': {e}", path.display()))?;
        let file = BufReader::new(file);
        let icon_family = IconFamily::read(file)
            .map_err(|e| format!("Error parsing file '{}': {e}", path.display()))?;

        // Somewhat arbitarily choose the one with the closest width.
        let width = THUMBNAIL_SIZE.0.round() as i64;
        let mut best_error = i64::MAX;
        let mut best_type = None;
        for icon_type in icon_family.available_icons() {
            let error = (width - icon_type.pixel_width() as i64).abs();
            if error <= best_error {
                best_error = error;
                best_type = Some(icon_type);
            }
        }

        if best_type.is_none() {
            return Err(format!("Unable to find an icon in '{}'", path.display()));
        }

        let icon = icon_family.get_icon_with_type(best_type.unwrap()).unwrap();

        let mut buffer = vec![];
        icon.write_png(&mut buffer)
            .expect("Error writing data from icns to buffer");

        Ok(Image::new(path, buffer))
    }

    pub fn load(path: PathBuf) -> Result<Image, String> {
        if path.extension() == Some(OsStr::new("icns")) && !Self::is_png(&path)? {
            return Self::load_icns(path);
        }

        // Manually loading the image and passing it as bytes is the only way I
        // could get it to handle URIs with spaces
        let mut buffer = vec![];
        let mut file = std::fs::File::open(path.clone())
            .map_err(|e| format!("Error opening {}: {e}", path.display()))?;

        file.read_to_end(&mut buffer)
            .map_err(|e| format!("Error reading {}: {e}", path.display()))?;

        Ok(Image::new(path, buffer))
    }

    pub fn to_egui_image(&self) -> egui::Image<'static> {
        egui::Image::from_bytes(self.path.display().to_string(), self.buffer.clone())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct EchidnaApp {
    cmd: String,

    doc_type: DocTypes,
    utis: String,
    exts: String,

    group_by: GroupBy,

    default_file_name: String,
    previous_name: Option<OsString>, // Previous name chosen by Save As

    terminal: String,
    generic_terminal: String,

    custom_shim_icon: Option<Image>,

    show_help: Arc<AtomicBool>,
    help_cache: Arc<Mutex<CommonMarkCache>>,
}

impl EchidnaApp {
    fn new_with_cc(cc: &eframe::CreationContext) -> Self {
        let mut app = EchidnaApp::default();

        let gets = |key, default: &str| {
            cc.storage
                .and_then(|x| x.get_string(key))
                .unwrap_or_else(|| default.into())
        };

        app.terminal = gets(TERM_KEY, term::default_terminal());
        app.generic_terminal = gets(GENERIC_TERM_KEY, "");
        app.group_by = cc
            .storage
            .and_then(|x| x.get_string(GROUP_BY_KEY))
            .map(|x| {
                serde_json::from_str::<GroupBy>(&x).expect("Error deserializing default GroupBy")
            })
            .unwrap_or_default();

        DEFAULT_APP_NAME.clone_into(&mut app.default_file_name);

        app
    }

    fn generate_inner(&mut self) -> Result<(), String> {
        if self.cmd.is_empty() {
            bail!("Command must not be empty");
        }

        let doc_type = match self.doc_type {
            DocTypes::TextFiles => generate::DocTypes::TextFiles,
            DocTypes::AllDocs => generate::DocTypes::AllDocs,
            DocTypes::UTIs => {
                if self.utis.is_empty() {
                    bailf!("UTIs must not be empty");
                }
                generate::DocTypes::UTIs(self.utis.clone())
            }
            DocTypes::Exts => {
                if self.exts.is_empty() {
                    bailf!("Extensions must not be empty");
                }
                generate::DocTypes::Exts(self.exts.clone())
            }
        };

        if self.terminal == GENERIC && self.generic_terminal.is_empty() {
            return Err("Generic terminal must not be empty".to_string());
        }

        // Shame to have to use to_string_lossy(), everwhere else, the filename is
        // an OsStr(ing). At least here the user has the chance  to fix it if it
        // gets mangled.
        let dialog = rfd::FileDialog::new().set_file_name(
            self.previous_name
                .as_ref()
                .map(|x| x.to_string_lossy().to_string())
                .unwrap_or(self.default_file_name.clone()),
        );

        let Some(app_path) = dialog.save_file() else {
            return Ok(());
        };

        if let Some(name) = app_path.file_name() {
            if name != OsStr::new(&self.default_file_name) {
                self.previous_name = Some(name.to_owned());
            }
        }

        let terminal = if self.terminal == GENERIC {
            TerminalApp::Generic(self.generic_terminal.clone())
        } else {
            TerminalApp::Supported(self.terminal.clone())
        };
        let config = Config {
            command: self.cmd.clone(),
            group_open_by: self.group_by,
            terminal,
        };
        let shim_path = get_shim_path()?;

        let mut gen = Generator::gen(
            &config,
            &doc_type,
            &shim_path,
            None,
            self.custom_shim_icon.as_ref().map(|x| &*x.path),
            app_path.clone(),
        )?;
        let res = gen.save(false);

        match res {
            Ok(()) => {
                if let Err(e) = opener::reveal(gen.final_bundle_path()) {
                    eprintln!("{e}"); // No modal, failure here does no real harm.
                }
                return Ok(());
            }
            Err(err) => match err {
                SaveErr::Other(msg) => {
                    bail!(msg);
                }
                SaveErr::AppAlreadyExists => (),
            },
        }

        // Couldn't write app because one already exists. Give user a chance to ovewrite.
        let result = rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Error: Destination already exists")
            .set_description(format!(
                "Destination '{}' already exists. Overwrite?",
                app_path.display()
            ))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show();

        if result == rfd::MessageDialogResult::No {
            return Ok(());
        }

        let res = gen.save(true);

        match res {
            Ok(()) => (),
            Err(SaveErr::Other(msg)) => bail!(msg),
            Err(SaveErr::AppAlreadyExists) => {
                bailf!("Still couldn't write destination '{}'", app_path.display());
            }
        };

        if let Err(e) = opener::reveal(gen.final_bundle_path()) {
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
                DEFAULT_APP_NAME.clone_into(&mut self.default_file_name);
                return;
            }
        };

        let mut chars = word.chars();
        let mut first = chars
            .next()
            .expect("SplitWhitespace returned an empty string!?");
        first.make_ascii_uppercase();

        self.default_file_name.clear();
        self.default_file_name.push(first);
        self.default_file_name.extend(chars);
        self.default_file_name += "Opener";
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
                    commonmark_str!(
                        "help",
                        ui,
                        &mut *help_cache.lock().unwrap(),
                        "app_files/help.md"
                    );
                });
            });
        });
    }

    fn draw_form(&mut self, ui: &mut egui::Ui) {
        Grid::new("Form").num_columns(2).show(ui, |ui| {
            ui.label("Command:")
                .on_hover_text("Shell command to run to open files.");
            ui.centered_and_justified(|ui| {
                let cmd = egui::TextEdit::singleline(&mut self.cmd);
                if ui.add(cmd).changed() {
                    self.update_default_file_name();
                }
            });
            ui.end_row();

            ui.label("Documents:")
                .on_hover_text("Documents to support opening.");
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("Document Type Combo Box")
                    .selected_text(self.doc_type.display_name().to_owned())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.doc_type,
                            DocTypes::TextFiles,
                            DocTypes::TextFiles.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.doc_type,
                            DocTypes::AllDocs,
                            DocTypes::AllDocs.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.doc_type,
                            DocTypes::UTIs,
                            DocTypes::UTIs.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.doc_type,
                            DocTypes::Exts,
                            DocTypes::Exts.display_name(),
                        );
                    });

                if self.doc_type == DocTypes::UTIs {
                    ui.add(egui::TextEdit::singleline(&mut self.utis).hint_text("Comma-delimited"));
                }

                if self.doc_type == DocTypes::Exts {
                    ui.add(egui::TextEdit::singleline(&mut self.exts).hint_text("Comma-delimited"));
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
                            if ui
                                .selectable_label(self.terminal == terminal, terminal)
                                .clicked()
                            {
                                terminal.clone_into(&mut self.terminal);
                            }
                        }

                        if ui
                            .selectable_label(self.terminal == GENERIC, GENERIC)
                            .clicked()
                        {
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

            ui.label("Open Files:")
                .on_hover_text("Open files in single window or one window per file?");
            ui.horizontal(|ui| {
                ui.radio_value(&mut self.group_by, GroupBy::All, "Together");
                ui.radio_value(&mut self.group_by, GroupBy::None, "Individually");
            });
            ui.end_row();
        });
    }

    fn change_shim_icon(&mut self, path: Option<PathBuf>) {
        let path = path.or_else(|| {
            rfd::FileDialog::new()
                .add_filter("image", *SUPPORTED_EXTS)
                .pick_file()
        });

        let Some(path) = path else {
            // Pressed cancel.
            return;
        };

        self.custom_shim_icon = match Image::load(path.clone()) {
            Ok(x) => Some(x),
            Err(e) => {
                modal(format!("Error loading icon from '{}': {e}", path.display()));
                None
            }
        };
    }

    fn draw_icon_column(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            let img_bg = ui.visuals().widgets.inactive.weak_bg_fill;
            ui.add(if let Some(img) = &self.custom_shim_icon {
                img.to_egui_image()
                    .fit_to_exact_size(THUMBNAIL_SIZE.into())
                    .bg_fill(img_bg)
            } else {
                egui::Image::new(DEFAULT_SHIM_ICON_THUMB)
                    .fit_to_exact_size(THUMBNAIL_SIZE.into())
                    .bg_fill(img_bg)
            });

            if ui.button("Select…").clicked() {
                self.change_shim_icon(None);
            }

            let reset_button = egui::Button::new("Default Icon");
            if ui
                .add_enabled(self.custom_shim_icon.is_some(), reset_button)
                .clicked()
            {
                self.custom_shim_icon = None;
            }
        });
    }

    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                let file = &i.raw.dropped_files[0];
                assert!(file.path.is_some());
                self.change_shim_icon(file.path.clone());
            }
        });
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        Grid::new("Root")
            .num_columns(2)
            .spacing((SECTION_SPACING, 0.0))
            .show(ui, |ui| {
                self.draw_icon_column(ui);
                self.draw_form(ui);
            });

        ui.add_space(SECTION_SPACING);

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                if ui.button("?").clicked() {
                    self.show_help.store(true, Ordering::Relaxed);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Save As…").clicked() {
                        self.generate();
                    }
                });
            });
        });
    }
}

impl eframe::App for EchidnaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let frame = egui::Frame::central_panel(&ctx.style()).inner_margin(WINDOW_PADDING);
        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            self.draw(ui);
            self.draw_help(ctx);
        });

        self.handle_dropped_files(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(TERM_KEY, self.terminal.clone());
        storage.set_string(GENERIC_TERM_KEY, self.generic_terminal.clone());
        storage.set_string(GROUP_BY_KEY, serde_json::to_string(&self.group_by).unwrap());
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

    // Maybe they're running the executable directly from the targets/ directory? Then echidna-shim
    // will be right there.
    let shim_path = {
        let mut path =
            std::env::current_exe().map_err(|e| format!("Failed to get current exe: {e}"))?;

        if !path.pop() {
            bailf!(
                "Couldn't pop binary filename from path '{}' !?",
                shim_path.display()
            );
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
    let image_ret =
        image::load_from_memory(include_bytes!("../../app_files/icon.png")).map(|x| x.into_rgb8());

    let image = match image_ret {
        Ok(x) => x,
        Err(_) => {
            // TODO: logging
            return std::sync::Arc::new(egui::viewport::IconData::default());
        }
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
            .with_max_inner_size(MAX_INNER_SIZE)
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Echidna",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(EchidnaApp::new_with_cc(cc))
        }),
    )
}
