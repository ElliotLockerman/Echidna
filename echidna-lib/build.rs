
use std::env::var;

fn main() {
    std::fs::write("build_log.txt", format!("{:?}", var("DEBUG"))).unwrap();

    let build_mode = if var("DEBUG").expect("DEBUG not set") == "true" { 
        "debug" 
    } else {
        "release"
    };
    println!("cargo:rustc-env=BUILD_MODE={build_mode}");
}

