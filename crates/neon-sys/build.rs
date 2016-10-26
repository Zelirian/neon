extern crate gcc;
extern crate regex;

use std::process::{Command, Stdio};
use std::env;
use regex::Regex;

fn main() {
    // 1. Build the object file from source using node-gyp.
    build_object_file();

    // 2. Link the library from the object file using gcc.
    link_library();
}

fn build_object_file() {
    let npm_command = if cfg!(unix) { "npm" } else { "npm.cmd" };
    let node_gyp_command = if cfg!(unix) { "node-gyp" } else { "node-gyp.cmd" };

    // Ensure that all package.json dependencies and dev dependencies are installed.
    Command::new(npm_command).args(&["install", "--silent"]).status().ok().expect("Failed to run \"npm install\" for neon-sys!");

    // Run `node-gyp configure` (appending -d in debug mode).
    let configure_args = if debug() { vec!["configure", "-d", "--arch=ia32"] } else { vec!["configure", "--arch=ia32"] };
    let output = Command::new(node_gyp_command)
        .args(&configure_args)
        .output()
        .expect("Failed to run \"node-gyp configure\" for neon-sys!");

    if cfg!(windows) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let node_root_dir = Regex::new(r"'-Dnode_root_dir=(.+)'").unwrap()
            .captures(&stderr)
            .and_then(|captures| captures.at(1))
            .expect("Couldn't find node_root_dir in node-gyp output.");
        println!("cargo:node_root_dir={}", node_root_dir);
    }

    // Run `node-gyp build` (appending -d in debug mode).
    let build_args = if debug() { vec!["build", "-d"] } else { vec!["build"] };
    Command::new(node_gyp_command)
        .stderr(Stdio::null()) // Prevent cargo build from hanging on Windows.
        .args(&build_args)
        .status()
        .ok()
        .expect("Failed to run \"node-gyp build\" for neon-sys!");
}

// Link the built object file into a static library.
fn link_library() {
    let configuration = if debug() { "Debug" } else { "Release" };
    let object_path = if cfg!(unix) {
        format!("build/{}/obj.target/neon/src/neon.o", configuration)
    } else {
        format!("build\\{}\\obj\\neon\\neon.obj", configuration)
    };

    gcc::Config::new().object(object_path).compile("libneon.a");
}

fn debug() -> bool {
    match env::var("DEBUG") {
        Ok(s) => s == "true",
        Err(_) => false
    }
}
