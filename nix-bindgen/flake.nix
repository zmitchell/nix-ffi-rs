{
  description = "A project for experimenting with Rust bindings to Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # Latest commit on nix#8699: (Towards) stable C bindings
    nix-c-bindings.url = "github:NixOS/nix/70d5d8356d3365f4d89f2fcca0e4583bee3c1bf4";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, nix-c-bindings, ...}:
    flake-utils.lib.eachDefaultSystem (system: 
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        mainDeps = with pkgs; [
            boost
            openssl
            pkg-config
        ];
        darwinExtraDeps = with pkgs; [
          libiconv
          darwin.apple_sdk.frameworks.Security
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "nix-bindgen";
          version = "0.0.1";
          cargoLock.lockFile = ./Cargo.lock;

          src = builtins.path { path = ./.; name = "nix-bindgen"; };

          buildInputs = [nix-c-bindings.packages.${system}.nix] ++ mainDeps ++ pkgs.lib.optional pkgs.hostPlatform.isDarwin darwinExtraDeps;
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            self.packages.${system}.default
          ];
        };
      }
    );
}
