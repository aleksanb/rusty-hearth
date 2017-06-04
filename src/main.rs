#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs::{File, metadata};
use std::io::SeekFrom;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::sync::{Mutex, Arc};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

use rocket::State;
use rocket_contrib::JSON;

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

fn tail_log(hearthstone_path: &Path, tx: Sender<parsers::LogEvent>) -> io::Result<()> {
    let mut handle = io::BufReader::new(File::open(&hearthstone_path)?);
    handle.seek(SeekFrom::End(0))?;

    let mut last_known_file_size = metadata(&hearthstone_path)?.len();

    loop {
        let mut buffer = String::new();
        match handle.read_line(&mut buffer)? {
            0 => {
                let current_file_size = metadata(&hearthstone_path)?.len();
                if current_file_size < last_known_file_size {
                    last_known_file_size = current_file_size;
                    handle.seek(SeekFrom::Start(0))?;
                    tx.send(parsers::LogEvent::PowerLogRecreated).unwrap();
                } else {
                    thread::sleep(Duration::from_millis(250));
                }
            }
            _ => {
                parsers::parse_log_line(&buffer).map(|play| tx.send(play).unwrap());
            }
        }
    }
}

#[get("/")]
fn root(game_state: State<Arc<Mutex<models::GameState>>>) -> JSON<models::GameState> {
    let game_state = game_state.lock().unwrap();
    JSON(game_state.clone())
}

fn main() {
    init_log().expect("log.config should be initialized");

    let hearthstone_path = Path::new(r"C:\Program Files (x86)\Hearthstone\Logs\Power.log");

    let mut game_state = Arc::new(Mutex::new(models::GameState::default()));
    let mut consumer_game_state = game_state.clone();

    let (tx, rx) = channel();

    thread::spawn(move || {
                      println!("Producer thread started");
                      tail_log(hearthstone_path, tx).expect("Log should be tailed");
                      println!("Producerr thread stopped");
                  });

    thread::spawn(move || {
        println!("Consumer thread started");
        while let Ok(play) = rx.recv() {
            match play {
                parsers::LogEvent::GameComplete |
                parsers::LogEvent::PowerLogRecreated => {
                    println!("Reseting GameState: {:?}", play);

                    *consumer_game_state.lock().unwrap() = models::GameState::default();
                }
                parsers::LogEvent::Play(play) => {
                    if consumer_game_state.lock().unwrap().handle_play(play) {
                        println!("New state: {:?}", *consumer_game_state.lock().unwrap());
                        println!();
                    }
                }
            }
        }

        println!("Consumer thread stopped");
    });

    rocket::ignite()
        .mount("/", routes![root])
        .manage(game_state.clone())
        .launch();
}
