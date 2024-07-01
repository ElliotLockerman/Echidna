use crate::bundle_tmp_dir::BundleTmpDir;
use crate::config::Config;

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use rand::Rng;

const INFO_PLIST_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>

    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>

            {{#if utis}}
            <key>LSItemContentTypes</key>
            <array>
                {{#each utis}}
                <string>{{this}}</string>
                {{/each}}
            </array>
            {{else if exts}}
            <key>CFBundleTypeExtensions</key>
            <array>
                {{#each exts}}
                <string>{{this}}</string>
                {{/each}}
            </array>
            {{/if}}

            <key>CFBundleTypeRole</key>
            <string>Editor</string>

        </dict>
    </array>


    <key>CFBundleExecutable</key>
    <string>{{app_display_name}}</string>

    <key>CFBundleIconFile</key>
    <string>AppIcon.icns</string>

    <key>CFBundleIdentifier</key>
    <string>{{bundle_id}}</string>

    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>

    <key>CFBundleName</key>
    <string>{{app_display_name}}</string>

    <key>CFBundlePackageType</key>
    <string>APPL</string>

    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
</dict>
</plist>
"#;

const SHIM_APP_ICON: &[u8] = include_bytes!("../../app_files/ShimAppIcon.icns");

// UTIs and Exts are comma-delimited lists of Uniform Type Identifiers and Extensions,
// respectively.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DocTypes {
    #[default]
    TextFiles,
    AllDocs,
    UTIs(String),
    Exts(String),
}

impl DocTypes {
    pub fn to_info_kv(&self) -> (String, Vec<String>) {
        use DocTypes::*;
        match self {
            TextFiles => (
                "utis".to_owned(),
                vec!["public.text".to_owned(), "public.data".to_owned()],
            ),
            AllDocs => (
                "utis".to_owned(),
                vec!["public.content".to_owned(), "public.data".to_owned()],
            ),
            UTIs(s) => {
                let v = s
                    .split(',')
                    .map(|x| x.trim().to_owned())
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>();
                ("utis".to_owned(), v)
            }
            Exts(s) => {
                let v = s
                    .split(',')
                    .map(|x| x.trim().trim_start_matches('.').to_owned())
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>();
                ("exts".to_owned(), v)
            }
        }
    }
}

pub enum SaveErr {
    // AppAlreadyExists is separated out to give the user an opportunity to ovewrite.
    AppAlreadyExists,
    Other(String),
}

impl SaveErr {
    pub fn to_msg(&self, app_dst_path: &Path) -> String {
        match self {
            SaveErr::AppAlreadyExists => format!(
                "App already exists at '{}'. Run with [-f|--force] to overwrite.",
                app_dst_path.display()
            ),
            SaveErr::Other(msg) => msg.to_owned(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(target_os = "macos")]
fn write_shim_bin(mac_os: &Path, app_name: &OsStr, shim_bin: &Path) -> Result<(), String> {
    let bin_path = mac_os.join(app_name);
    fs::copy(shim_bin, bin_path).map_err(|e| {
        format!(
            "Error copying shim binary '{}' to temporary directory '{}': {e}",
            shim_bin.display(),
            mac_os.display()
        )
    })?;

    Ok(())
}

fn write_info_plist(
    contents: &Path,
    app_name: &str,
    doc_type: &DocTypes,
    bundle_id: &str,
) -> Result<(), String> {
    let (file_selectors_key, file_selectors) = doc_type.to_info_kv();

    let reg = handlebars::Handlebars::new();
    let rendered = reg
        .render_template(
            INFO_PLIST_TEMPLATE,
            &serde_json::json!({
                "app_display_name": app_name,
                file_selectors_key: file_selectors,
                "bundle_id": bundle_id,
            }),
        )
        .map_err(|e| format!("Error rendering Info.plist template: {e}"))?;

    let plist_dir = contents.join("Info.plist");

    fs::write(&plist_dir, rendered).map_err(|e| {
        format!(
            "Error writing Info.plist to temporary directory '{}': {e}",
            plist_dir.display()
        )
    })?;

    Ok(())
}

// Returns (app_name, bundle_name); app_name is without .app, bundle_* has it
fn get_names(mut app_path: PathBuf) -> Result<(OsString, OsString, PathBuf), String> {
    let file_name = || {
        app_path
            .file_name()
            .map(|x| x.to_owned())
            .ok_or_else(|| format!("Couldn't get file name from {}", app_path.display()))
    };

    let app_name = match app_path.extension() {
        Some(ext) => {
            if ext == "app" {
                app_path
                    .file_stem()
                    .ok_or_else(|| {
                        format!("Couldn't get app name from path '{}'", app_path.display())
                    })?
                    .to_owned()
            } else {
                file_name()?
            }
        }
        None => file_name()?,
    };

    let mut bundle_name = app_name.clone();
    bundle_name.push(".app");

    app_path.set_file_name(&bundle_name);

    Ok((app_name, bundle_name, app_path))
}

fn write_icon(icon_path: Option<&Path>, resources: &Path) -> Result<(), String> {
    let mut shim_icon = resources.to_owned();
    shim_icon.push("AppIcon.icns");
    let icon_path = match icon_path {
        Some(x) => x,
        None => {
            return fs::write(&shim_icon, SHIM_APP_ICON).map_err(|e| {
                format!(
                    "Error write shim's icon to temorary '{}': {e}",
                    shim_icon.display()
                )
            })
        }
    };

    let ext = icon_path.extension();
    if ext == Some(OsStr::new("png")) || ext == Some(OsStr::new("icns")) {
        return fs::copy(icon_path, &shim_icon).map(|_| ()).map_err(|e| {
            format!(
                "Error copying custom shim from '{}' to temorary '{}': {e}",
                icon_path.display(),
                shim_icon.display()
            )
        });
    }

    let image = image::open(icon_path)
        .map_err(|e| format!("Error loading icon from '{}': {e}", icon_path.display()))?;
    image
        .save_with_format(&shim_icon, image::ImageFormat::Png)
        .map_err(|e| {
            format!(
                "Error writing icon to temporary '{}': {e}",
                shim_icon.display()
            )
        })?;

    Ok(())
}

fn generate_bundle_id(app_name: &str) -> String {
    let hostname = gethostname::gethostname();
    let num = rand::thread_rng().gen_range(0..=999999);
    format!("local.{}.{app_name}{num}", hostname.to_string_lossy())
}

fn save_bundle<S: AsRef<Path>, D: AsRef<Path>>(
    tmp_bundle: S,
    dst_bundle: D,
    overwrite: bool,
) -> Result<(), SaveErr> {
    // Unfortunately not atomic, but an effort is made the protect existing data (unfortunately,
    // this effort isn't atomic either), but it should be ease to recreate shim apps.

    let (tmp_bundle, dst_bundle) = (tmp_bundle.as_ref(), dst_bundle.as_ref());

    // We'll get an error if we try to overwrite a non-empty bundle (the likely case). Instead,
    // try to move it to the tmp dir; if we get an error trying to move the new bundle, we can try to
    // move the original one back. This process is also failable, so its best-effort.
    let mut saved_bundle = None;
    if overwrite && dst_bundle.exists() && dst_bundle.file_name() == tmp_bundle.file_name() {
        let saved_bundle_inner = tmp_bundle.with_extension("bak");
        if let Ok(()) = fs::rename(dst_bundle, &saved_bundle_inner) {
            saved_bundle = Some(saved_bundle_inner);
        } // No else, best effort.
    }

    // The actual rename
    let res = fs::rename(tmp_bundle, dst_bundle).map_err(|e| {
        match e.raw_os_error() {
            Some(libc::ENOTEMPTY) => SaveErr::AppAlreadyExists, // ErrorKind::DirectoryNotEmpty not available on stable
            Some(_) | None => SaveErr::Other(format!(
                "Error moving temporary app '{}' to out_dir '{}': {e}",
                tmp_bundle.display(),
                dst_bundle.display(),
            )),
        }
    });

    if res.is_err() && saved_bundle.is_some() {
        if let Some(saved) = saved_bundle {
            // Best effort.
            let _ = fs::rename(saved, dst_bundle);
        }
    }

    res
}

////////////////////////////////////////////////////////////////////////////////

pub struct Generator {
    tmp_dir: BundleTmpDir,
    final_bundle_path: PathBuf,
    saved: bool,
}

impl Generator {
    // YOU MUST STILL CALL SAVE() AFTER.
    pub fn gen(
        config: &Config,
        doc_type: &DocTypes,
        shim_bin: &Path,
        bundle_id: Option<&str>,
        icon_path: Option<&Path>,
        app_path: PathBuf,
    ) -> Result<Generator, String> {
        let (app_name, bundle_name, final_bundle_path) = get_names(app_path)?;

        let tmp_dir = BundleTmpDir::new(&bundle_name)?;

        let default_bundle_id = generate_bundle_id(&app_name.to_string_lossy());

        write_info_plist(
            tmp_dir.contents(),
            &app_name.to_string_lossy(),
            doc_type,
            bundle_id.unwrap_or(&default_bundle_id),
        )?;
        write_shim_bin(tmp_dir.mac_os(), &app_name, shim_bin)?;
        config
            .write(tmp_dir.resources())
            .map_err(|e| e.to_string())?;
        write_icon(icon_path, tmp_dir.resources())?;

        Ok(Generator {
            tmp_dir,
            final_bundle_path,
            saved: false,
        })
    }

    // Safe to call again after an error, but not after a success.
    pub fn save(&mut self, overwrite: bool) -> Result<(), SaveErr> {
        assert!(!self.saved);
        let res = save_bundle(self.tmp_dir.app_root(), &self.final_bundle_path, overwrite);
        if res.is_ok() {
            self.saved = true;
        }
        res
    }

    pub fn final_bundle_path(&self) -> &Path {
        &self.final_bundle_path
    }
}
