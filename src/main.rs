#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::thread;
use std::sync::mpsc::{channel, Sender};
use std::path::Path;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;

mod models;

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
        handle.write_all(log_config);
    }

    Ok(())
}

fn tail_log(tx: Sender<models::Play>) -> io::Result<()> {
    let hearthstone_path = Path::new(r"C:\Program Files (x86)\Hearthstone\Logs\Power.log");
    let mut handle = io::BufReader::new(File::open(hearthstone_path)?);
    handle.seek(SeekFrom::End(0))?;

    loop {
        let mut b = String::new();
        match handle.read_line(&mut b) {
            Ok(0) => {
                continue;
            }
            Ok(n) => {
                println!("{:?}", parse_log_line(&b));
            }
            Err(err) => println!("Error!, {}", err),
        }
    }

    Ok(())
}

fn parse_log_line(line: &str) -> Option<models::Play> {
    lazy_static! {
        static ref card_update_pattern: regex::Regex = regex::Regex::new(r"^.*id=(?P<id>\d*) .*cardId=(?P<card_id>[a-zA-Z0-9_]*) .*player=(?P<player>\d*)").unwrap();
        static ref game_complete_pattern: regex::Regex = regex::Regex::new(r"^.*TAG_CHANGE Entity=GameEntity tag=STATE value=COMPLETE.*$").unwrap();
    }

    if game_complete_pattern.is_match(line) {
        return None;
    }

    card_update_pattern
        .captures(line)
        .and_then(|group| {
            let id = group.name("id").map(|m| m.as_str());
            let card_id = group.name("card_id").map(|m| m.as_str());
            let player = group.name("player").map(|m| m.as_str());

            match (id, card_id, player) {
                (Some(id), Some(card_id), Some(player)) if card_id != "" => {
                    Some(models::Play {
                             id: id.to_string(),
                             card_id: card_id.to_string(),
                             player: player.to_string(),
                         })
                }
                _ => None,
            };

            None
        })
}

fn main() {
    println!("Initializing log config");
    init_log().unwrap();
    println!("Initialized log config");

    let (tx, rx) = channel();

    println!("Spawning log thread");
    let log_thread = thread::spawn(|| tail_log(tx));
    println!("Spawned log thread");

    println!("Start receiving events");
    while let Ok(play) = rx.recv() {
        println!("{:?}", play);
    }
}
