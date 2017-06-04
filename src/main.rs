#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::fs::{File, metadata};
use std::io::SeekFrom;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

mod models;
mod parsers;

fn init_log() -> io::Result<()> {
    let log_config = b"[Achievements]
LogLevel=1
FilePrinting=true
ConsolePrinting=true
ScreenPrinting=false

[Power]
LogLevel=1
FilePrinting=true
ConsolePrinting=true
ScreenPrinting=false";

    let hs_dir = Path::new(r"C:\Program Files (x86)\Hearthstone\log.config");
    if !hs_dir.exists() {
        let mut handle = File::create(hs_dir)?;
        handle.write_all(log_config)?;
    }

    Ok(())
}

fn tail_log(tx: Sender<parsers::LogEvent>) -> io::Result<()> {
    let hearthstone_path = Path::new(r"C:\Program Files (x86)\Hearthstone\Logs\Power.log");
    let mut handle = io::BufReader::new(File::open(&hearthstone_path)?);
    handle.seek(SeekFrom::End(0))?;

    let mut last_known_file_size = metadata(&hearthstone_path).unwrap().len();

    loop {
        let mut buffer = String::new();
        match handle.read_line(&mut buffer) {
            Ok(0) => {
                let current_file_size = metadata(&hearthstone_path).unwrap().len();
                if current_file_size < last_known_file_size {
                    last_known_file_size = current_file_size;
                    handle.seek(SeekFrom::Start(0)).unwrap();
                    tx.send(parsers::LogEvent::PowerLogRecreated).unwrap();
                }

                thread::sleep(Duration::from_millis(250));
            }
            Ok(_) => {
                parsers::parse_log_line(&buffer).map(|play| tx.send(play).unwrap());
            }
            Err(err) => println!("Error!, {}", err),
        }
    }
}

fn main() {
    println!("Initializing log config");
    init_log().unwrap();
    println!("Initialized log config");

    let (tx, rx) = channel();

    println!("Spawning log thread");
    thread::spawn(|| tail_log(tx));
    println!("Spawned log thread");

    println!("Start receiving events");
    let mut game_state = models::GameState::default();
    while let Ok(play) = rx.recv() {
        match play {
            parsers::LogEvent::GameComplete => {
                println!("Game completed");
                game_state = models::GameState::default();
            }
            parsers::LogEvent::PowerLogRecreated => {
                println!("PowerLog recreated");
                game_state = models::GameState::default();
            }
            parsers::LogEvent::Play(play) => {
                let updated = game_state.handle_play(play);
                if updated {
                    println!("New state: {:?}", game_state);
                    println!();
                }
            }
        }
    }
}
