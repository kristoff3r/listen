{ darwin
, fetchFromGitHub
, openssl
, pkg-config
, lib
, rustPlatform
, stdenv
}:
let
  inherit (darwin.apple_sdk.frameworks)
    CoreServices
    SystemConfiguration
    Security;
  inherit (lib) optionals;
  inherit (stdenv.hostPlatform) isDarwin;
in
rustPlatform.buildRustPackage rec {
  pname = "cargo-leptos";
  version = "0.2.29";

  src = fetchFromGitHub {
    owner = "leptos-rs";
    repo = pname;
    rev = "v${version}";
    hash = "sha256-z8oF/FFA/sZcjl5M2RTxNJvgvdK00jmWHJDc16kQGJQ=";
  };

  cargoHash = "sha256-Vce/qeKmw6tv9L+OLA3KUDBFgm1zkcRVo/C37FuACOI=";

  buildInputs = optionals isDarwin [
    SystemConfiguration
    Security
    CoreServices
    openssl
    pkg-config
  ];

  # https://github.com/leptos-rs/cargo-leptos#dependencies
  buildFeatures = [ "no_downloads" ]; # cargo-leptos will try to install missing dependencies on its own otherwise
  doCheck = false; # Check phase tries to query crates.io

  meta = with lib; {
    description = "Build tool for the Leptos web framework";
    mainProgram = "cargo-leptos";
    homepage = "https://github.com/leptos-rs/cargo-leptos";
    changelog = "https://github.com/leptos-rs/cargo-leptos/releases/tag/v${version}";
    license = with licenses; [ mit ];
    maintainers = with maintainers; [ benwis ];
  };
}
