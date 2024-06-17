
use echidna_util::config::Config;

use std::path::{Path, PathBuf};
use std::fs;

const INFO_PLIST_TEMPLATE: &str = r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>

    <key>CFBundleDisplayName</key>
    <string>{{app_display_name}}</string>

    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array/>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.item</string>
            </array>
        </dict>
    </array>

    {{#if exts}}
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeExtensions</key>
            <array>
            {{#each exts}}
                <string>{{this}}</string>
            {{/each}}
            </array>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
        </dict>
    </array>
    {{/if}}


    <key>CFBundleExecutable</key>
    <string>{{app_display_name}}</string>

    <key>CFBundleIdentifier</key>
    <string>com.lockerman.EchidnaShim</string>

    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>

    <key>CFBundleName</key>
    <string>{{app_display_name}}</string>

    <key>CFBundlePackageType</key>
    <string>APPL</string>

    <key>CFBundleShortVersionString</key>
    <string>1.0</string>

    <key>LSMinimumSystemVersion</key>
    <string>10.11.0</string>

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

////////////////////////////////////////////////////////////////////////////////

#[cfg(target_os = "macos")]
fn write_shim_bin(app_name: &String, mac_os: &Path, shim_bin: &Path) -> Result<(), String> {
    let bin_path = mac_os.join(app_name);
    fs::copy(shim_bin, &bin_path).map_err(|e|
        format!("Error copying shim binary '{}' to temporary directory '{}': {e}", shim_bin.display(), mac_os.display())
    )?;

    Ok(())
}

fn parse_exts(exts: &str) -> Vec<String> {
    exts.split(',')
        .map(|x| x.trim().trim_start_matches('.').to_owned())
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
}

fn write_info_plist(app_name: &str, exts: &str, contents: &Path) -> Result<(), String> {
    let extsv = parse_exts(exts);

    let reg = handlebars::Handlebars::new();
    let rendered = reg.render_template(
        INFO_PLIST_TEMPLATE,
        &serde_json::json!({
            "app_display_name": app_name,
            "exts": extsv
        }),
    ).map_err(|e| format!("Error rendering Info.plist template: {e}"))?;

    let plist_dir = contents.join("Info.plist");

    fs::write(&plist_dir, rendered).map_err(|e|
        format!("Error writing Info.plist to temporary directory '{}': {e}", plist_dir.display())
    )?;

    Ok(())
}

fn write_config(config: &Config, resources: &Path) -> Result<(), String> {
    let config_dir = resources.join("config.json5");

    let config_json = serde_json::to_string(config).map_err(|e|
        format!("Error serializing config {config:?}: {e}")
    )?;

    fs::write(&config_dir, config_json).map_err(|e|
        format!("Error writing config to temporary directory '{}': {e}", config_dir.display())
    )?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////

// exts is a comma-delimited list of extensions to support
pub fn generate_shim_app(
    app_name: String,
    config: &Config,
    exts: String,
    shim_bin: &Path,
    out_dir: PathBuf
) -> Result<(), String> {

    let app_dir_name = app_name.clone() + ".app";

    if !out_dir.exists() {
        return Err(format!("Output directory '{}' doesn't exist", out_dir.display()));
    }

    let tmp_dir = tempdir::TempDir::new("echidna-lib")
        .map_err(|e| format!("Error creating temporary directory: {e}"))?;

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
            format!("Error creating directory '{}' in temp dir {}: {e}", relative.display(), prefix.display())
        })
    };

    let app_root = tmp_dir.path().join(&app_dir_name);
    let contents = app_root.join("Contents");
    let mac_os = contents.join("MacOS");
    let resources = contents.join("Resources");

    pretty_create_dir(&app_root)?;
    pretty_create_dir(&contents)?;
    pretty_create_dir(&mac_os)?;
    pretty_create_dir(&resources)?;

    write_info_plist(&app_name, &exts, &contents)?;
    write_shim_bin(&app_name, &mac_os, shim_bin)?;
    write_config(config, &resources)?;

    let app_dst = out_dir.join(&app_dir_name);
    if app_dst.exists() {
        return Err(format!("'{}' already exists", app_dst.display()));
    }
    fs::rename(&app_root, &app_dst).map_err(|e|
        format!(
            "Error moving temporary app '{}' to out_dir '{}': {e}",
            app_root.display(),
            app_dst.display(),
        )
    )?;

    Ok(())
}

