{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    # nix2container.url = "github:nlewo/nix2container";
    # nix2container.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs:
    let
      system = "x86_64-linux";
      overlays = [ (import inputs.rust-overlay) ];
      pkgs = import inputs.nixpkgs { inherit system overlays; };
      # nix2containerPkgs = inputs.nix2container.packages.${system};
      lib = pkgs.lib;
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

      craneBuild = rec {
        pname = "listen";
        version = "0.1.0";

        src = lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = path: type:
            (lib.hasSuffix "tailwind.config.js" path) ||
            (lib.hasSuffix ".css" path) ||
            (lib.hasInfix "/public/" path) ||
            (lib.hasInfix "/migrations/" path) ||
            # Default filter from crane (allow .rs files)
            (craneLib.filterCargoSources path type)
          ;
        };
        args = {
          inherit src pname version;
          strictDeps = true;
          buildInputs = with pkgs; [
            postgresql_16
          ];
          nativeBuildInputs = with pkgs; [
            postgresql_16
            makeWrapper
            ourCargoLeptos
            binaryen
            tailwindcss
          ];
        };
        cargoArtifacts = craneLib.buildDepsOnly args;
        buildArgs = args // {
          inherit cargoArtifacts;
          buildPhaseCargoCommand = "cargo leptos build --release -vvv";
          cargoTestCommand = "cargo leptos test --release -vvv";
          cargoExtraArgs = "";

          installPhaseCommand = ''
            mkdir -p $out/bin
            cp target/release/backend $out/bin/
            cp -r target/site $out/bin/
            wrapProgram $out/bin/backend \
              --set LEPTOS_SITE_ROOT $out/bin/site
          '';
        };
        package = craneLib.buildPackage buildArgs;

        clippy = craneLib.cargoClippy (args // {
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets --all-features -- --deny warnings";
        });

        fmt = craneLib.cargoFmt {
          inherit pname src;
        };
      };

      listen-api = craneBuild.package;
      listen-api-image = pkgs.callPackage ./image.nix {
        # inherit (nix2containerPkgs.nix2container) buildImage;
        inherit (pkgs.dockerTools) buildImage;
        inherit listen-api;
      };

      # Version overrides
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
      ourLeptosfmt = pkgs.callPackage ./nix/leptosfmt.nix { inherit rustPlatform; };
      ourCargoLeptos = pkgs.callPackage ./nix/cargo-leptos.nix { inherit rustPlatform; };
      ourWasmBindgen = pkgs.wasm-bindgen-cli.override {
        inherit rustPlatform;
        version = "0.2.95";
        hash = "sha256-prMIreQeAcbJ8/g3+pMp1Wp9H5u+xLqxRxL+34hICss=";
        cargoHash = "sha256-6iMebkD7FQvixlmghGGIvpdGwFNLfnUcFke/Rg8nPK4=";
      };
      ourDieselCli = pkgs.callPackage ./nix/diesel-cli.nix { inherit rustPlatform; };
    in
    {
      nixosModules.default = import ./module.nix { inherit listen-api-image; };

      packages.${system} = {
        inherit listen-api;
        image = listen-api-image;
        leptosfmt = ourLeptosfmt;
      };

      checks.${system} = {
        listen-clippy = craneBuild.clippy;
        listen-fmt = craneBuild.fmt;
      };

      devShells."${system}".default = craneLib.devShell {
        DATABASE_URL= "postgres://postgres:postgres@localhost:5433/listen";
        inputsFrom = [ listen-api ];

        packages = with pkgs; [
          bashInteractive
          devenv

          ourLeptosfmt
          ourWasmBindgen
          ourDieselCli

          yt-dlp
          ffmpeg
        ];
      };
    };
}
