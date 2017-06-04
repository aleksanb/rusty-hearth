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

mod models;
mod parsers;
mod power_log;

use models::{ThreadsafeGameState, GameState};
use power_log::{init_log, tail_log};
use rocket::State;
use rocket_contrib::JSON;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

#[get("/")]
fn root(game_state: State<ThreadsafeGameState>) -> JSON<GameState> {
    let game_state = game_state.lock().unwrap();
    JSON(game_state.clone())
}

fn main() {
    let hearthstone_log_config = Path::new(r"C:\Program Files (x86)\Hearthstone\log.config");
    let hearthstone_path = Path::new(r"C:\Program Files (x86)\Hearthstone\Logs\Power.log");

    init_log(&hearthstone_log_config).expect("log.config should initialize correctly");

    let (tx, rx) = mpsc::channel();
    let game_state = ThreadsafeGameState::default();
    let consumer_game_state = game_state.clone();

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

                    *consumer_game_state.lock().unwrap() = GameState::default();
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
