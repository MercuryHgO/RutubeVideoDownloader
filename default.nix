{ pkgs ? import <nixpkgs> {} }:

let 
  pkgConfigDeps = with pkgs; [
    openssl    
  ];
in
pkgs.rustPlatform.buildRustPackage {
  pname = "ruteube-video-downloader";
  version = "0.1.0";

  buildInputs = with pkgs; [ pkg-config ];
  nativeBuildInputs = with pkgs; [ ffmpeg pkg-config ];

  cargoSha256 = "sha256-vYt4yQITuoTNckbdcJmYFHzL+w85v25KOYqhbGTseJU=";

  buildPhase = ''
    export PKG_CONFIG_PATH="${
      builtins.concatStringsSep ":" (map (pkg: "${pkg.dev}/lib/pkgconfig") pkgConfigDeps)
    }"

    cargo build --release --target x86_64-unknown-linux-gnu
  '';

  src = ./.;
}

# let
#   rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
#   pkgs = import <nixpkgs> { 
#     overlays = [ rust_overlay];
#   };
#   rustVersion = "latest";
#   # rustVersion = "1.62.0";
#   rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
#     extensions = [
#       "rust-src" # for rust-analyzer
#       "rust-analyzer"
#     ];
#     targets = [
#       "x86_64-pc-windows-gnu"
#       "x86_64-pc-windows-gnullvm"
#     ];
#   };

#   pkgConfigDeps = with pkgs; [
#     openssl
#   ];

#   buildInputs = pkgs: [
#     rust
#   ] ++ (with pkgs; [
#     # zlib
#     # gcc
#     # pkgsCross.mingwW64.stdenv.cc
#     # wayland
#     # xorg.libX11
#     # libxkbcommon
#     pkg-config
#     ffmpeg
#   ]);
# in
# pkgs.stdenv.
# (pkgs.buildFHSEnv {
#   name = "money_counter_dev";

#   profile = with pkgs;
#   ''
#     export RUST_BACKTRACE=1
#     export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L native=${pkgsCross.mingwW64.windows.pthreads}/lib"
#     export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${lib.makeLibraryPath ([libGL libGLU])}

    # export PKG_CONFIG_PATH="${
    #   builtins.concatStringsSep ":" (map (pkg: "${pkg.dev}/lib/pkgconfig") pkgConfigDeps)
    # }"

#   '';
# }).env

