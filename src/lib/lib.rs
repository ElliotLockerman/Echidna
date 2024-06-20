
pub mod config;
pub mod misc;

use crate::config::Config;

use std::path::{Path, PathBuf};
use std::fs;
use std::ffi::{OsStr, OsString};

const INFO_PLIST_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>

    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            {{#if exts}}
            <key>CFBundleTypeExtensions</key>
            <array>
            {{#each exts}}
                <string>{{this}}</string>
            {{/each}}
            </array>
            {{/if}}
            <key>LSItemContentTypes</key>
            <array>
                <string>public.item</string>
            </array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
        </dict>
    </array>


    <key>CFBundleExecutable</key>
    <string>{{app_display_name}}</string>

    <key>CFBundleIconFile</key>
    <string>AppIcon.icns</string>

    <key>CFBundleIdentifier</key>
    <string>{{identifier}}</string>

    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>

    <key>CFBundleName</key>
    <string>{{app_display_name}}</string>

    <key>CFBundlePackageType</key>
    <string>APPL</string>

    <key>LSUIElement</key>
    <false/>

    <key>NSAppTransportSecurity</key>
    <dict>
        <key>NSAllowsArbitraryLoads</key>
        <true/>
    </dict>

    <key>NSMainNibFile</key>
    <string>MainMenu</string>

    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
"#;

const SHIM_APP_ICON: &[u8] = include_bytes!("../../app_files/ShimAppIcon.icns");


pub enum GenErr {
    // AppAlreadyExists is separated out to give the user an opportunity to ovewrite.
    AppAlreadyExists,
    Other(String),
}

impl GenErr {
    pub fn to_msg(&self, app_dst_path: &Path) -> String {
        match self {
            GenErr::AppAlreadyExists =>
                    format!("App already exists at '{}'. Run with [-f|--force] to overwrite.",
                    app_dst_path.display()
                ),
            GenErr::Other(msg) => format!("{msg}"),
        }
    }
}

macro_rules! gen_err_other {
    ($($arg:tt)+) => {{
        GenErr::Other(format!($($arg)*))
    }}
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(target_os = "macos")]
fn write_shim_bin(mac_os: &Path, app_name: &OsStr, shim_bin: &Path) -> Result<(), GenErr> {
    let bin_path = mac_os.join(app_name);
    fs::copy(shim_bin, bin_path).map_err(|e|
        gen_err_other!("Error copying shim binary '{}' to temporary directory '{}': {e}", shim_bin.display(), mac_os.display())
    )?;

    Ok(())
}

fn parse_exts(exts: &str) -> Vec<String> {
    exts.split(',')
        .map(|x| x.trim().trim_start_matches('.').to_owned())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
}

fn write_info_plist(
    contents: &Path,
    app_name: &str,
    exts: &str,
    identifier: &str,
) -> Result<(), GenErr> {

    let extsv = parse_exts(exts);

    let reg = handlebars::Handlebars::new();
    let rendered = reg.render_template(
        INFO_PLIST_TEMPLATE,
        &serde_json::json!({
            "app_display_name": app_name,
            "exts": extsv,
            "identifier": identifier,
        }),
    ).map_err(|e| gen_err_other!("Error rendering Info.plist template: {e}"))?;

    let plist_dir = contents.join("Info.plist");

    fs::write(&plist_dir, rendered).map_err(|e|
        gen_err_other!("Error writing Info.plist to temporary directory '{}': {e}", plist_dir.display())
    )?;

    Ok(())
}

// Returns (app_name, bundle_name); app_name is without .app, bundle_* has it
fn get_names(mut app_path: PathBuf) -> Result<(OsString, OsString, PathBuf), GenErr> {
    let app_name = || app_path.file_name()
        .map(|x| x.to_owned())
        .ok_or_else(|| gen_err_other!("Couldn't get file name from {}", app_path.display()));

    let app_name = match app_path.extension() {
        Some(ext) =>
            if ext == "app" {
                app_path.file_stem()
                    .ok_or_else(|| gen_err_other!("Couldn't get app name from path '{}'", app_path.display()))?
                    .to_owned()
            } else {
                app_name()?
            }
        None => app_name()?
    };

    let mut bundle_name = app_name.clone();
    bundle_name.push(".app");

    app_path.set_file_name(&bundle_name);

    Ok((app_name, bundle_name, app_path))
}

fn write_icon(resources: &Path) -> Result<(), GenErr> {
    let mut shim_icon = resources.to_owned();
    shim_icon.push("AppIcon.icns");
    fs::write(shim_icon, SHIM_APP_ICON)
        .map_err(|e| gen_err_other!(
            "Error write shim's icon to temorary {}: {e}",
            resources.display()
        ))?;

    Ok(())
}

fn move_bundle<S: AsRef<Path>, D: AsRef<Path>>(
    tmp_bundle: S,
    dst_bundle: D,
    overwrite: bool) -> Result<(), GenErr> {

    // Unfortunately not atomic, but an effort is made the protect existing data (unfortunately,
    // this effort isn't atomic either), but it should be ease to recreate shim apps.

    let (tmp_bundle, dst_bundle) = (tmp_bundle.as_ref(), dst_bundle.as_ref());


    // We'll get an error if we try to overwrite a non-empty bundle (the likely case). Instead,
    // try to move it to the tmp dir; if we get an error trying to move the new bundle, we can try to
    // move the original one back. This process is also failable, so its best-effort.
    let mut saved_bundle = None;
    if overwrite && dst_bundle.exists() && dst_bundle.file_name() == tmp_bundle.file_name() {
        let saved_bundle_inner = tmp_bundle.with_extension("bak");
        match fs::rename(&dst_bundle, &saved_bundle_inner) {
            Ok(()) => { saved_bundle = Some(saved_bundle_inner); },
            Err(_) =>  (),
        }
    }

    // The actual rename
    let res = fs::rename(&tmp_bundle, &dst_bundle).map_err(|e| {
        match e.raw_os_error() {
            Some(libc::ENOTEMPTY) => GenErr::AppAlreadyExists, // ErrorKind::DirectoryNotEmpty not available on stable
            Some(_) | None => gen_err_other!(
                "Error moving temporary app '{}' to out_dir '{}': {e}",
                tmp_bundle.display(),
                dst_bundle.display(),
            ),
        }
    });

    if res.is_err() && saved_bundle.is_some() {
        // Best effort.
        let _ = fs::rename(&saved_bundle.unwrap(), &dst_bundle);
    }

    res
}

////////////////////////////////////////////////////////////////////////////////

// exts is a comma-delimited list of extensions to support.
// Returns path to app bundle on success.
pub fn generate_shim_app(
    config: &Config,
    exts: String,
    identifier: &str,
    shim_bin: &Path,
    app_path: PathBuf,
    overwrite: bool
) -> Result<PathBuf, GenErr> {

    let (app_name, bundle_name, final_bundle_path) = get_names(app_path)?;

    let tmp_dir = tempdir::TempDir::new("echidna-lib")
        .map_err(|e| gen_err_other!("Error creating temporary directory: {e}"))?;

    let pretty_create_dir = |path: &Path| {
        fs::create_dir(path).map_err(|e| {
            let (prefix, relative) = {
                match path.strip_prefix(tmp_dir.path()) {
                    Ok(striped) => (tmp_dir.path(), striped),
                    Err(_) => {
                        // Just ignore it, we're formatting an error, its more important to get the
                        // original error out than to figure this out.
                        (path, path)
                    }
                }
            };
            gen_err_other!("Error creating directory '{}' in temp dir {}: {e}", relative.display(), prefix.display())
        })
    };

    // All in a temporary directory.
    let app_root = tmp_dir.path().join(bundle_name);
    let contents = app_root.join("Contents");
    let mac_os = contents.join("MacOS");
    let resources = contents.join("Resources");

    pretty_create_dir(&app_root)?;
    pretty_create_dir(&contents)?;
    pretty_create_dir(&mac_os)?;
    pretty_create_dir(&resources)?;

    write_info_plist(
        &contents,
        &app_name.to_string_lossy(),
        &exts,
        identifier,
    )?;
    write_shim_bin(&mac_os, &app_name, shim_bin)?;
    config.write(&resources).map_err(|e| gen_err_other!("{e}"))?;
    write_icon(&resources)?;

    move_bundle(app_root, &final_bundle_path, overwrite)?;

    Ok(final_bundle_path)
}

