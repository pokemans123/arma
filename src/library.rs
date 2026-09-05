use lofty::prelude::*;
use lofty::probe::Probe;
use reqwest;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub enum Source {
    File(PathBuf),
    Link(String),
}

#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub cover: Option<Vec<u8>>,
    pub duration: f64,
}

#[derive(Debug)]
pub struct Song {
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub cover: Option<Vec<u8>>,
    pub source: Source,
    pub duration: f64,
}

#[derive(Deserialize)]
struct YtDlpInfo {
    title: String,
    uploader: Option<String>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    album: Option<String>,
}

impl Song {
    ///Grab metadata for a song played locally on the system
    /// Parameters: path -> &str
    /// Returns Metadata struct
    pub fn read_local(path: &str) -> Result<Metadata, Box<dyn Error>> {
        let file = Probe::open(path)?.read()?;

        let tag = file.primary_tag().or_else(|| file.first_tag());

        let title = tag
            .and_then(|t| t.title())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown Title".to_string());
        let artist = tag.and_then(|t| t.artist()).map(|s| s.to_string());
        let album = tag.and_then(|t| t.album()).map(|s| s.to_string());

        let duration = file.properties().duration().as_secs_f64();

        let cover = tag
            .and_then(|t| t.pictures().first())
            .map(|pic| pic.data().to_vec());

        Ok(Metadata {
            title,
            album,
            artist,
            cover,
            duration,
        })
    }

    pub fn fetch_cover(thumb_url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let bytes = reqwest::blocking::get(thumb_url)?.bytes()?;
        Ok(bytes.to_vec())
    }

    ///Grab metadata for a song played froma
    /// Parameters: path -> &str
    /// Returns Metadata struct
    pub fn read_stream(url: &str) -> Result<Metadata, Box<dyn Error>> {
        let output = Command::new("yt-dlp")
            .arg("--no-playlist")
            .arg("-j")
            .arg(url)
            .output()?;

        if !output.status.success() {
            return Err(
                format!("yt-dlp failed: {}", String::from_utf8_lossy(&output.stderr)).into(),
            );
        }

        let info: YtDlpInfo = serde_json::from_slice(&output.stdout)?;

        let cover = info
            .thumbnail
            .as_deref()
            .and_then(|thumb_url| Self::fetch_cover(thumb_url).ok());

        Ok(Metadata {
            title: info.title,
            album: info.album,
            artist: info.uploader,
            duration: info.duration.unwrap_or(0.0),
            cover,
        })
    }

    ///Classifies if the current song being played is a URL or file
    /// Paramters: input -> &str
    /// Returns struct Source
    pub fn classify(input: &str) -> Source {
        if input.starts_with("http") {
            Source::Link(input.to_string())
        } else {
            Source::File(PathBuf::from(input))
        }
    }

    ///Enum matches to classify a song as either played locally or from a URL
    /// Parameters: source -> Source
    /// Returns struct Song
    pub fn read_metadata(source: Source) -> Result<Song, Box<dyn Error>> {
        match &source {
            Source::File(path) => {
                let data = Self::read_local(path.to_str().unwrap())?;
                Ok(Song {
                    duration: data.duration,
                    title: data.title,
                    album: data.album,
                    artist: data.artist,
                    cover: data.cover,
                    source,
                })
            }

            Source::Link(link) => {
                let data = Self::read_stream(link)?;
                Ok(Song {
                    duration: data.duration,
                    title: data.title,
                    album: data.album,
                    artist: data.artist,
                    cover: data.cover,
                    source,
                })
            }
        }
    }
}
