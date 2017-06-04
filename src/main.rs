#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::fs::File;
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
    let mut handle = io::BufReader::new(File::open(hearthstone_path)?);
    handle.seek(SeekFrom::End(0))?;

    loop {
        let mut buffer = String::new();
        match handle.read_line(&mut buffer) {
            Ok(0) => {
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
                print!("Game completed");
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
