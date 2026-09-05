use mpvipc::{Error, Mpv};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Error> {
    let socket_path = "/tmp/rust-player-test.sock";

    // Spawn mpv exactly the way your real player eventually will:
    // idle mode, no video, our own IPC socket.
    // This relies on the nix-wrapped `mpv` (with mpris) being first on $PATH,
    // which the devShell's shellHook already ensures.
    let mut mpv_process = Command::new("mpv")
        .arg("--idle")
        .arg("--no-video")
        .arg(format!("--input-ipc-server={socket_path}"))
        .spawn()
        .expect("failed to spawn mpv - is it on $PATH?");

    // Give mpv a moment to create the socket before we try to connect.
    // (A real player should retry/poll instead of a fixed sleep - fine for a smoke test.)
    sleep(Duration::from_millis(500));

    let mpv = Mpv::connect(socket_path)?;

    // Replace with a real audio file path on your machine.
    mpv.run_command(mpvipc::MpvCommand::LoadFile {
        file: "/home/pranav/Music/chill/test.webm".to_string(),
        option: mpvipc::PlaylistAddOptions::Replace,
    })?;

    println!("Loaded track. Check `playerctl -p mpv metadata` now.");
    sleep(Duration::from_secs(5));

    println!("Pausing via Rust...");
    mpv.set_property("pause", true)?;
    println!("Check `playerctl -p mpv status` -> should say Paused");
    sleep(Duration::from_secs(5));

    println!("Resuming via Rust...");
    mpv.set_property("pause", false)?;
    sleep(Duration::from_secs(5));

    mpv.run_command(mpvipc::MpvCommand::Quit)?;
    mpv_process.wait().ok();

    Ok(())
}
