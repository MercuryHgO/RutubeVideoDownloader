let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { 
    overlays = [ rust_overlay];
  };
  rustVersion = "latest";
  # rustVersion = "1.62.0";
  rust = pkgs.rust-bin.stable.${rustVersion}.default.override {
    extensions = [
      "rust-src" # for rust-analyzer
      "rust-analyzer"
    ];
    targets = [
      "x86_64-pc-windows-gnu"
      "x86_64-pc-windows-gnullvm"
    ];
  };

  pkgConfigDeps = with pkgs; [
    openssl
  ];

in
(pkgs.buildFHSEnv {
  name = "money_counter_dev";
  
  targetPkgs = pkgs: [
    rust
  ] ++ (with pkgs; [
    zlib
    gcc
    pkgsCross.mingwW64.stdenv.cc
    wayland
    xorg.libX11
    libxkbcommon
    pkg-config
    ffmpeg
  ]) ++ (with pkgs.xorg; [
    libxcb
    libXcursor
    libXrandr
    libXi
  ]);

  profile = with pkgs;
  ''
    export RUST_BACKTRACE=1
    export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L native=${pkgsCross.mingwW64.windows.pthreads}/lib"
    export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${lib.makeLibraryPath ([libGL libGLU])}

    export PKG_CONFIG_PATH="${
      builtins.concatStringsSep ":" (map (pkg: "${pkg.dev}/lib/pkgconfig") pkgConfigDeps)
    }"

  '';
  

  runScript = "${pkgs.writeShellScriptBin "dev_env" ''
    tmux new-session -d -t egor-govno-rust

    tmux split-window -h -t egor-govno-rust
    tmux resize-pane -t egor-govno-rust:0.1 -x 20%

    tmux send-keys -t egor-govno-rust:0 'bash' C-m

    tmux send-keys -t egor-govno-rust:0.0 'hx' C-m

    tmux attach-session -t egor-govno-rust

    while tmux has-session -t egor-govno-rust; do sleep 1; done
    exit
  ''}/bin/dev_env";
}).env
