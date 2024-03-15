const MUSICAL_NOTES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub fn frequency_to_note(freq: f32) -> String {
    let note: i32 = (12.0 * (freq / 440.0).log2()).round() as i32;
    let note_index = (note + 69) % 12;
    MUSICAL_NOTES[note_index as usize].to_string()
}

pub struct NoteAndDeviation {
    pub musical_note: String,
    pub deviation: f32,
}

pub fn frequency_to_note_with_deviation(freq: f32) -> NoteAndDeviation {
    let musical_note = frequency_to_note(freq);
    let note_frequency = note_to_frequency(&musical_note);
    let deviation = (freq / note_frequency).log2();

    NoteAndDeviation {
        musical_note,
        deviation,
    }
}

pub fn note_to_frequency(note: &str) -> f32 {
    let note_index = MUSICAL_NOTES.iter().position(|&x| x == note).unwrap() as i32;
    440.0 * 2.0_f32.powf((note_index - 9) as f32 / 12.0)
}
