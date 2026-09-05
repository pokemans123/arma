use lofty::config::ParseOptions;
use lofty::file::AudioFile;
use lofty::mpeg::MpegFile;
use lofty::tag::TagType;
use std::fs::File;

fn main() {
    let path = "/home/pranav/Music/chill/Sugar Sweet [mC7J8Rphv5E].mp3";
    let mut file_content = File::open(path)?;

}
