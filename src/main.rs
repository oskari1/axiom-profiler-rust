use prototype::continue_agent::ParserContinueAgent;
use prototype::file_io::*;
use prototype::{LogParser, Z3Parser1};
use std::env;
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    let settings = get_settings();
    let filename = if args.len() < 2 {
        settings.file.to_string()
    } else {
        args[1].to_string()
    };
    let mut parser = Z3Parser1::default();
    let mut continue_agent = ParserContinueAgent::new(&parser);
    // let timer_async = async {
    //     println!("Starting time");
    //     tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
    //     println!("ending time");
    //     continue_agent.stop_parsing();
    // };
    // let read_parse_async = async {
    //     println!("Starting parse");
    //     if let Err(e) = parser.read_and_parse_file(filename, &settings) {
    //         println!("{}", e);
    //     }
    // };
    if settings.timeout > 0.0 {
        let _timer = thread::spawn(move || {
            thread::sleep(Duration::from_secs_f32(10.0));
            continue_agent.stop_parsing();
        });
        let read_parse = thread::spawn(move || {
            let settings = settings.clone();
            if let Err(e) = parser.read_and_parse_file(&filename, &settings) {
                println!("{}", e);
            }
        });
        if let Err(e) = read_parse.join() {
            println!("{:?}", e);
        };
    } else if let Err(e) = parser.read_and_parse_file(&filename, &settings) {
        println!("{}", e);
    }
    // let rt = tokio::runtime::Runtime::new().unwrap();
    // rt.block_on(async {futures::join!(timer_async, read_parse_async)});
}
