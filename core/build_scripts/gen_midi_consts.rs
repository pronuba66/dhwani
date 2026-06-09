use std::fmt::Write;

const TOTAL_NOTES: usize = 128;

pub fn generate() -> String {
    let mut s = String::new();
    s.push_str(
        r#"pub const MIDI_NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
"#,
    );
    s.push_str("\n#[allow(clippy::approx_constant)]\n");
    s.push_str("#[allow(clippy::unreadable_literal)]\n");
    let _ = writeln!(s, "pub const MIDI_NOTE_MULS: [f32; {TOTAL_NOTES}] = [");
    for i in 0..TOTAL_NOTES {
        #[allow(clippy::cast_precision_loss)]
        let mul = ((i.cast_signed() - 69) as f64 / 12f64).exp2();
        #[allow(clippy::cast_possible_truncation)]
        let _ = writeln!(s, "    {}f32, ", mul as f32);
    }
    s.push_str("];\n");
    s
}
