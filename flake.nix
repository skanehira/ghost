{
  description = "Ghost application flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, naersk, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system;
          overlays = overlays;
        };
        rustToolchain = pkgs.rust-bin.stable."1.88.0".default;
        naersk-lib = (naersk.lib.${system}).override {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
        darwinFrameworks = if pkgs.stdenv.isDarwin then
          with pkgs.darwin.apple_sdk.frameworks; [ CoreServices CoreFoundation ]
        else
          [];
        ghost = naersk-lib.buildPackage {
          pname = "ghost";
          version = "0.1.0";
          src = ./.;
          cargoBuildOptions = [ "--release" ];
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = darwinFrameworks;
        };
      in {
        packages.default = ghost;

        apps.default = {
          type = "app";
          program = "${ghost}/bin/ghost";
        };

        devShells.default = pkgs.mkShell {
          packages =
            [ rustToolchain pkgs.pkg-config ]
            ++ darwinFrameworks;
          env.RUST_BACKTRACE = "1";
        };
      });
}
