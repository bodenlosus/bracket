{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        # packages.default = pkgs.callPackage ./. {};
        devShells.default = pkgs.mkShell {
          shellHook = ''
            export XDG_DATA_DIRS=${pkgs.gtk4}/share/gsettings-schemas/gtk4-${pkgs.gtk4.version}:$XDG_DATA_DIRS
          '';
          packages = with pkgs; [
            gtk4
            rustToolchain
            libxml2
            blueprint-compiler
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
            libadwaita
            libpanel
            inspector
            # ffmpeg
            rustPlatform.bindgenHook
        ];
          env = {
            # BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.getVersion pkgs.clang}/include";
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
      });
}
