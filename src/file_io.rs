use std::fs::{File, OpenOptions, self};
use std::io::{self, BufRead, Write, BufWriter};
use std::path::Path;

use crate::items;

const SETTINGS: &str = "settings.txt";
const CAPACITY: usize = 1 << 25; // 32 MiB

/// Returns an iterator for the lines of text in a text file.
/// Used to avoid reading entire file at once.
/// # Errors
/// Errors if file cannot be opened.
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Writes the debug text of `obj` to `file`.
/// # Panics
/// Panics if an error occurs when writing to `file`.
pub fn write<T>(file: &mut BufWriter<File>, obj: &T) where T: items::Print {
    file.write_all(obj.format().as_bytes()).expect("write failed");
}

pub fn pretty_write(file: &mut BufWriter<File>, text: &str) {
    file.write_all(text.as_bytes()).expect("write failed");
    // file.write_all("\n".as_bytes()).expect("write failed");
}

/// Returns a file to write to after creating it (if necessary) and truncating it.
/// # Panics
/// Panics if an error occurs when opening file.
pub fn open_file_truncate(filename: &str) -> BufWriter<File> {
    BufWriter::with_capacity(CAPACITY, OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(filename).unwrap_or_else(|_| panic!("Error opening {}", filename)))
}

#[derive(Default, Clone)]
pub struct Settings {
    pub file: String,
    pub reuse: bool,
    pub verbose: bool,
    pub enable_io: bool,
    pub timeout: f32,
}


/// Read settings from `SETTINGS`, saving them to a `Settings`
pub fn get_settings() -> Settings {
    let settings_text = fs::read_to_string(SETTINGS).expect("settings file should exist");
    let mut settings = Settings::default();
    for line in settings_text.lines() {
        let line_split: Vec<&str> = line.split(' ').collect();
        let (setting_type, value) = (line_split[0], line_split[1]);
        match setting_type {
            "FILE" => {
                settings.file = value.to_string();
            },
            "REUSE" => {
                settings.reuse = value.parse().unwrap();
            },
            "VERBOSE" => {
                settings.verbose = value.parse().unwrap();
            },
            "ENABLE_IO" => {
                settings.enable_io = value.parse().unwrap();
            },
            "TIMEOUT" => {
                settings.timeout = value.parse().unwrap();
            }
            _ => {}
        }
    }
    settings
}