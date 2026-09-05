use std::process::Command;

fn main() {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("quiet")
        .arg("-show_entries")
        .arg("format_tags=title,album,artist:format=duration")
        .arg("-of")
        .arg("json")
        .arg("/home/pranav/Music/chill/Sugar Sweet [mC7J8Rphv5E].mp3")
        .output()
        .expect("failed to run ffprobe");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Metadata: {stdout}");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("ffprobe failed: {stderr}");
    }
}
