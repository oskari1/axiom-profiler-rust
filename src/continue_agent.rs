use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time;
use crate::LogParser;

#[derive(Clone)]
pub struct ParserContinueAgent {
    continue_parsing: Arc<Mutex<bool>>
}

impl ParserContinueAgent {
    pub fn new(parser: &impl LogParser) -> ParserContinueAgent {
        ParserContinueAgent{ continue_parsing: parser.get_continue_mutex() }
    }

    pub fn stop_parsing(&mut self) {
        match self.continue_parsing.lock() {
            Ok(mut guard) => {
                *guard = false;
            },
            Err(_poisoned) => {}    // don't need to do anything, parser panicked
        }
        println!("Interrupted parsing");
    }

    pub fn stop_on_timer(&mut self, time: f32) {
        // stub
        let mut agent = self.clone();
        if let Err(e) = thread::spawn(move || {
            thread::sleep(time::Duration::from_secs_f32(time));
            agent.stop_parsing();
        }).join() { // can't join here! Spawn thread in main??
            println!("Error: {:?}", e);
        }

    }
}
