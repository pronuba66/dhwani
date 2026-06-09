use std::{fs, path::Path};

#[path = "build_scripts/gen_midi_consts.rs"]
mod gen_midi_consts;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    // Generate notes
    println!("cargo:rerun-if-changed=build_scripts/gen_midi_consts.rs");
    let dest = Path::new("src/midi_consts.rs");
    fs::write(dest, gen_midi_consts::generate()).unwrap();
}
