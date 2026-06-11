use std::{env, fs, path::PathBuf};

#[path = "build_scripts/gen_midi_consts.rs"]
mod gen_midi_consts;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // Generate notes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_scripts/gen_midi_consts.rs");
    let dest = out_dir.join("midi_consts.rs");
    fs::write(dest, gen_midi_consts::generate()).unwrap();
}
