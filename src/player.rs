use crate::library;
use crate::library::Song;
use mpvipc::{Error, Mpv};
use std::env::{home_dir, var_os};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct Player {
    paused: bool,
    current_song: Option<Song>,
    socket: String,
    mpv_process: Child,
    mpv: Mpv,
    queue: PathBuf,
}

impl Player {
    /// Create a new Player instance by connecting mpv to a socket
    /// Parameters: socket_path -> str
    pub fn new(socket_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let _ = std::fs::remove_file(socket_path);
        let mut mpv_process = Command::new("mpv")
            .arg("--idle")
            .arg("--no-video")
            .arg(format!("--input-ipc-server={socket_path}"))
            .spawn()
            .expect("failed to spawn mpv; ensure it is on $PATH");

        thread::sleep(Duration::from_millis(500));

        let mpv = Mpv::connect(socket_path)?;

        let home_dir = var_os("HOME")
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "$HOME is not set"))?;

        let data_dir = PathBuf::from(home_dir)
            .join(".local")
            .join("state")
            .join("tuimusic");

        fs::create_dir_all(&data_dir)?;

        let playlist_file = data_dir.join("queue.m3u");

        match fs::File::create_new(&playlist_file) {
            Ok(_) => println!(
                "Previous queue not found. Created one at {}",
                playlist_file.display()
            ),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                println!("Previous queue found at {}", playlist_file.display())
            }
            Err(e) => panic!("Failed to create queue file: {e}"),
        }

        Ok(Player {
            paused: true,
            current_song: None,
            socket: socket_path.to_string(),
            mpv_process,
            queue: playlist_file,
            mpv,
        })
    }

    ///Gets the current song; Returns struct Song
    pub fn get_current_song(&self) -> Result<Song, Error> {
        let path: String = self.mpv.get_property("path")?;
        let source = library::Song::classify(&path);
        let current_song = library::Song::read_metadata(source).unwrap();
        Ok(current_song)
    }

    ///Moves on to the next song; will poll until the song changes
    pub fn next_song(&mut self) -> Result<(), Error> {
        let current_index: usize = self.mpv.get_property("playlist-playing-pos")?;
        self.mpv.run_command(mpvipc::MpvCommand::PlaylistNext)?;
        while current_index == self.mpv.get_property::<usize>("playlist-playing-pos")? {
            println!("Waiting for song to change...");
            thread::sleep(Duration::from_millis(50));
        }
        self.current_song = Some(self.get_current_song()?);
        Ok(())
    }

    ///Moves back to the previous song; will poll until song changes
    pub fn prev_song(&mut self) -> Result<(), Error> {
        let current_index: usize = self.mpv.get_property("playlist-playing-pos")?;
        self.mpv.run_command(mpvipc::MpvCommand::PlaylistPrev)?;
        while current_index == self.mpv.get_property::<usize>("playlist-playing-pos")? {
            println!("Waiting for song to change...");
            thread::sleep(Duration::from_millis(50));
        }
        self.current_song = Some(self.get_current_song()?);
        Ok(())
    }

    pub fn get_pos(&self) -> Result<f64, Error> {
        let pos = self.mpv.get_property("time-pos")?;
        Ok(pos)
    }

    pub fn toggle_shuffle(&mut self) -> Result<(), Error> {
        let is_shuffled: bool = self.mpv.get_property("shuffle")?;
        self.mpv.set_property("shuffle", !is_shuffled)?;
        Ok(())
    }

    ///Initalize the player by loading the queue file
    pub fn start(&mut self) -> Result<(), Error> {
        self.mpv
            .run_command(mpvipc::MpvCommand::LoadList {
                file: (self.queue.display().to_string()),
                option: (mpvipc::PlaylistAddOptions::Replace),
            })
            .unwrap();

        self.paused = self.mpv.get_property("pause")?;

        self.current_song = Some(self.get_current_song()?);

        Ok(())
    }

    pub fn play_pause(&mut self) -> Result<(), Error> {
        self.mpv.set_property("pause", !self.paused).unwrap();
        self.paused = self.mpv.get_property("pause")?;
        Ok(())
    }

    // ///Safely closes the player
    // pub fn drop(&mut self) {
    //     let _ = self.mpv.run_command(mpvipc::MpvCommand::Quit);
    //     self.mpv_process.wait().ok();
    // }
}

impl Drop for Player {
    fn drop(&mut self) {
        let _ = self.mpv.run_command(mpvipc::MpvCommand::Quit);
        self.mpv_process.wait().ok();
    }
}
