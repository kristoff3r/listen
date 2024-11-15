{ lib
, rustPlatform
, fetchFromGitHub
}:

rustPlatform.buildRustPackage rec {
  pname = "leptosfmt";
  version = "0.1.32";

  src = fetchFromGitHub {
    owner = "bram209";
    repo = "leptosfmt";
    rev = "79f2fc81682e0070ce4fd36b86c7c63eba3f85e8";
    hash = "sha256-VcTHmbJoTGFux/L5jDOj440XYXV98J+1D7GXRIcFRV0=";
    fetchSubmodules = true;
  };

  cargoHash = "sha256-uLADmJrM+nWivTd1IgK31ThniO88iwp8ZgozBoggzb8=";

  meta = with lib; {
    description = "Formatter for the leptos view! macro";
    mainProgram = "leptosfmt";
    homepage = "https://github.com/bram209/leptosfmt";
    changelog = "https://github.com/bram209/leptosfmt/blob/${src.rev}/CHANGELOG.md";
    license = with licenses; [ asl20 mit ];
    maintainers = with maintainers; [ figsoda ];
  };
}