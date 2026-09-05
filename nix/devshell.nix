{ pkgs, crane }:

let
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    extensions = [ "rust-src" "rust-analyzer" ];
  };

  mpvWithMpris = pkgs.mpv.override {
    scripts = [ pkgs.mpvScripts.mpris ];
  };
in
pkgs.mkShell {
  packages = [
    rustToolchain
    pkgs.clippy
    pkgs.cargo-watch
    mpvWithMpris
    pkgs.yt-dlp
    pkgs.playerctl
  ];

  shellHook = ''
    export PATH=${mpvWithMpris}/bin:$PATH
  '';
}
