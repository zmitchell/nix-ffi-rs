{
  description = "A project for experimenting with Rust bindings to Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ...}:
    flake-utils.lib.eachDefaultSystem (system: 
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        darwinExtraDeps = with pkgs; [
          libiconv
          darwin.apple_sdk.frameworks.Security
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "nix-ffi-rs";
          version = "0.0.1";
          cargoLock.lockFile = ./Cargo.lock;

          src = builtins.path { path = ./.; name = "nix-ffi-rs"; };

          buildInputs = with pkgs; [
            openssl
            pkg-config
          ] ++ lib.optional hostPlatform.isDarwin darwinExtraDeps;
        };
      }
    );
}
