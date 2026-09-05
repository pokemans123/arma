use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use mpvipc::{Error, Mpv, MpvCommand, PlaylistAddOptions};
mod player;
use player::Player;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
mod index;

mod library;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
