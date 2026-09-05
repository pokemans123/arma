use crate::library::{Song, Source};
use crate::player;
use std::fs;
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "ogg", "m4a", "wav", "opus"];

pub fn scan_directory(root: &str) -> Vec<Song> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
                .unwrap_or(false)
        })
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            match Song::read_metadata(Source::File(path.clone())) {
                Ok(song) => Some(song),
                Err(e) => {
                    eprintln!("skipping {path:?}: {e}");
                    None
                }
            }
        })
        .collect()
}
