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
        python = pkgs.python313;
        package = python.pkgs.callPackage ./package.nix {};
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        defaultPackage.${system} = package;
        # packages.default = pkgs.callPackage ./. {};
        devShells.default = pkgs.mkShell {
          packages = [package] ++ (with pkgs; [
            pkg-config
            rustToolchain
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
            rustPlatform.bindgenHook
            maturin
        ]) ++ ( with python.pkgs; [
          setuptools-rust
          pip
          wheel
        ]);
          env = {
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
      });
}
