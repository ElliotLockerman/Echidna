
use std::env::var;

fn main() {
    let build_mode = if var("DEBUG").expect("DEBUG not set") == "true" { 
        "debug" 
    } else {
        "release"
    };
    println!("cargo:rustc-env=BUILD_MODE={build_mode}");
}

