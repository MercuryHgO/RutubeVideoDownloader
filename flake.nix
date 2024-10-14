{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };

        pkg-config-deps = with pkgs; [
          openssl
        ];

        pkg-config-path = builtins.concatStringsSep ":" (
          map (pkg: "${pkg.dev}/lib/pkgconfig") pkg-config-deps
        );
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;

          nativeBuildInputs = with pkgs; [ ffmpeg ];
          buildInputs = with pkgs; [ pkg-config ];

          PKG_CONFIG_PATH = pkg-config-path;
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs;[
            pkg-config
            ffmpeg
          ];
          
          buildInputs = with pkgs;[
            cargo
            rustc
            rustfmt
            pre-commit
            rustPackages.clippy
            rust-analyzer
          ];

          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          PKG_CONFIG_PATH = pkg-config-path;
        };
      }
    );
}
