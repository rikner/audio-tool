const MUSICAL_NOTES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub fn frequency_to_note(freq: f32) -> String {
    let note: i32 = (12.0 * (freq / 440.0).log2()).round() as i32;
    let note_index = (note + 69) % 12;
    MUSICAL_NOTES[note_index as usize].to_string()
}

pub fn note_to_frequency(note: &str) -> f32 {
    let note_index = MUSICAL_NOTES.iter().position(|&x| x == note).unwrap() as i32;
    440.0 * 2.0_f32.powf((note_index - 9) as f32 / 12.0)
}
