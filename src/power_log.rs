use std::fs::{self, File};
use std::io::SeekFrom;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use parsers;

pub fn init_log(hearthstone_dir: &Path) -> io::Result<()> {
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

    if !hearthstone_dir.exists() {
        let mut handle = File::create(hearthstone_dir)?;
        handle.write_all(log_config)?;
    }

    Ok(())
}

pub fn tail_log(hearthstone_path: &Path, tx: Sender<parsers::LogEvent>) -> io::Result<()> {
    let mut handle = io::BufReader::new(File::open(&hearthstone_path)?);
    handle.seek(SeekFrom::End(0))?;

    let mut last_known_file_size = fs::metadata(&hearthstone_path)?.len();

    loop {
        let mut buffer = String::new();
        match handle.read_line(&mut buffer)? {
            0 => {
                let current_file_size = fs::metadata(&hearthstone_path)?.len();
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
