{
  description = "A flake using Oxalica's rust-overlay wrapped with bevy-flake.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    bevy-flake = {
      url = "github:swagtop/bevy-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      bevy-flake,
      rust-overlay,
      ...
    }:
    bevy-flake.lib.mkFlake {
      perSystem =
        {
          pkgs,
          system,
          packages,
          formatter,
          ...
        }:
        {
          inherit packages formatter;

          devShells.default = pkgs.mkShell {
            name = "bevy-flake-rust-overlay";
            packages = [
              packages.rust-toolchain
              packages.dioxus-cli.develop
              packages.bevy-cli

              (pkgs.callPackage ./nix/wgsl-analyzer.nix {
                rustPlatform = pkgs.makeRustPlatform {
                  cargo = packages.rust-toolchain;
                  rustc = packages.rust-toolchain;
                };
              })
              pkgs.inotify-tools
            ];
          };
        };

      config =
        {
          pkgs,
          system,
          default,
          ...
        }:
        let
          inherit (pkgs.lib)
            genAttrs
            recursiveUpdate
            ;
        in

        {
          src = builtins.path {
            name = "src";
            path = ./.;

            # Ignore files that aren't needed in compilation of Bevy project.
            filter =
              path: type:
              !(builtins.elem (baseNameOf path) [
                "flake.lock"
                "flake.nix"
                "assets.sh"
                "wesl.toml"
                "nix/wgsl-analyzer.nix"
                "Makefile"
                ".vscode"
              ]);
          };

          targetEnvironments = recursiveUpdate default.targetEnvironments {
            x86_64-pc-windows-msvc.RUSTFLAGS =
              (default.targetEnvironments.x86_64-pc-windows-msvc.RUSTFLAGS or "")
              + " -C target-feature=+crt-static";
            aarch64-pc-windows-msvc.RUSTFLAGS =
              (default.targetEnvironments.aarch64-pc-windows-msvc.RUSTFLAGS or "")
              + " -C target-feature=+crt-static";
          };

          rustToolchain =
            targets:
            let
              channel = "stable"; # For nightly, use "nightly".
            in
            pkgs.rust-bin.${channel}.latest.default.override {
              inherit targets;
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
            };

          withPkgs = import nixpkgs {
            inherit system;
            overlays = [
              (import rust-overlay)

              (final: prev: {
                wasm-bindgen-cli = prev.buildWasmBindgenCli rec {
                  src = prev.fetchCrate {
                    pname = "wasm-bindgen-cli";
                    version = "0.2.130";
                    hash = "sha256-9Og1O/6/VC3sc5hBjoIWBErmLW2UHGMsTQO+6+m/dng=";
                  };

                  cargoDeps = prev.rustPlatform.fetchCargoVendor {
                    inherit src;
                    inherit (src) pname version;
                    hash = "sha256-j81sUf1m5yXU4FJfGj7GV+hdqVCV8mRmNb0QIsMCjqw=";
                  };
                };
              })
            ];
            config = {
              allowUnfree = true;
              microsoftVisualStudioLicenseAccepted = true;
            };
          };
        };
    };
}
