use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub struct BundleTmpDir {
    _tmp_dir: tempdir::TempDir,
    app_root: PathBuf,
    contents: PathBuf,
    mac_os: PathBuf,
    resources: PathBuf,
}

impl BundleTmpDir {
    pub fn new(bundle_name: &OsStr) -> Result<BundleTmpDir, String> {
        let tmp_dir = tempdir::TempDir::new("echidna-lib")
            .map_err(|e| format!("Error creating temporary directory: {e}"))?;

        // All in a temporary directory.
        let app_root = tmp_dir.path().join(bundle_name);
        let contents = app_root.join("Contents");
        let mac_os = contents.join("MacOS");
        let resources = contents.join("Resources");

        let pretty_create_dir = |path: &Path| pretty_create_dir_inner(path, tmp_dir.path());
        pretty_create_dir(&app_root)?;
        pretty_create_dir(&contents)?;
        pretty_create_dir(&mac_os)?;
        pretty_create_dir(&resources)?;

        Ok(BundleTmpDir {
            _tmp_dir: tmp_dir,
            app_root,
            contents,
            mac_os,
            resources,
        })
    }

    pub fn app_root(&self) -> &Path {
        &self.app_root
    }

    pub fn contents(&self) -> &Path {
        &self.contents
    }

    pub fn mac_os(&self) -> &Path {
        &self.mac_os
    }

    pub fn resources(&self) -> &Path {
        &self.resources
    }
}

fn pretty_create_dir_inner(path: &Path, tmp_root: &Path) -> Result<(), String> {
    fs::create_dir(path).map_err(|e| {
        let (prefix, relative) = {
            match path.strip_prefix(tmp_root) {
                Ok(striped) => (tmp_root, striped),
                Err(_) => {
                    // Just ignore it, we're formatting an error, its more important to get the
                    // original error out than to figure this out.
                    (path, path)
                }
            }
        };
        format!(
            "Error creating directory '{}' in temp dir {}: {e}",
            relative.display(),
            prefix.display()
        )
    })
}
