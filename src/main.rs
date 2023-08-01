use std::env;
use prototype::{LogParser, Z3Parser1};
use prototype::file_io::*;


fn main() {
    let args: Vec<String> = env::args().collect();
    let settings = get_settings();
    let filename = if args.len() < 2 {
        &settings.file
    } else {
        &args[1]
    };
    let mut parser = Z3Parser1::default();
    if let Err(e) = parser.read_and_parse_file(filename, &settings) {
        println!("{}", e);
    }
}