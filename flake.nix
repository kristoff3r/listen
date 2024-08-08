{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      my-rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      devShell."${system}" = pkgs.mkShell {
        DATABASE_URL= "postgres://postgres:postgres@localhost/listen";
        buildInputs = with pkgs; [
          my-rust
          nodejs
          postgresql_16
          diesel-cli
          cargo-watch
          cargo-leptos
          cargo-generate
          wasm-bindgen-cli
          leptosfmt
          binaryen
          dart-sass
        ];
      };
    };
}
