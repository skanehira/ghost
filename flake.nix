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
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        packageName = cargoToml.package.name;
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        naersk-lib = pkgs.callPackage naersk {
          cargo = toolchain;
          rustc = toolchain;
          clippy = toolchain;
        };
        darwinFrameworks = pkgs.lib.optionals pkgs.stdenv.isDarwin (
          with pkgs.darwin.apple_sdk.frameworks;
            [ CoreServices CoreFoundation ]
        );
        commonNativeInputs = [ pkgs.pkg-config ];
        ghost = naersk-lib.buildPackage {
          pname = "ghost";
          version = "git";
          src = ./.;
          nativeBuildInputs = commonNativeInputs;
          buildInputs = darwinFrameworks;
          meta = with pkgs.lib; {
            description = "Simple background process manager with a TUI for Unix-like systems.";
            homepage = "https://github.com/skanehira/ghost";
            license = licenses.mit;
            mainProgram = packageName;
            platforms = platforms.unix;
          };
        };
      in {
        packages.default = ghost;

        apps.default = {
          type = "app";
          program = "${ghost}/bin/ghost";
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ toolchain ] ++ commonNativeInputs;
          buildInputs = darwinFrameworks;
          RUST_BACKTRACE = "1";
        };
      });
}
