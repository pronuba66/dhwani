use std::fmt::Write;

const TOTAL_NOTES: usize = 128;
const NOTES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub fn generate() -> String {
    let mut s = String::new();
    s.push_str("pub const MIDI_NOTE_NAMES: [&str; 12] = [\n");
    s.push_str("    ");
    write!(s, r#""{}","#, NOTES[0]).ok();
    for note in &NOTES[1..] {
        write!(s, r#" "{note}","#).ok();
    }
    s.push_str("\n];\n");
    s.push('\n');
    s.push_str("// In rad/s\n");
    s.push_str("#[allow(clippy::approx_constant)]\n");
    s.push_str("#[allow(clippy::unreadable_literal)]\n");
    writeln!(s, "pub const MIDI_NOTE_MULS: [f32; {TOTAL_NOTES}] = [").ok();
    for i in 0..TOTAL_NOTES {
        #[allow(clippy::cast_precision_loss)]
        let mul = ((i.cast_signed() - 69) as f64 / 12f64).exp2();
        writeln!(
            s,
            "    // {}, {}{},",
            i,
            NOTES[i % 12],
            (i / 12).cast_signed() - 1
        )
        .ok();
        #[allow(clippy::cast_possible_truncation)]
        writeln!(s, "    {}f32,", mul as f32).ok();
    }
    s.push_str("];\n");
    s
}
