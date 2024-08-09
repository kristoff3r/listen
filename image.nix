{ pkgs, buildImage, listen-api }:

buildImage {
  name = "listen-api";
  tag = "nix";

  copyToRoot = pkgs.buildEnv {
    name = "image-root";
    paths = [
      pkgs.procps
      pkgs.bashInteractive
      pkgs.coreutils
      pkgs.curl
    ];
    pathsToLink = [ "/bin" ];
  };

  config = {
    Entrypoint = [ "${listen-api}/bin/server" ];
    Env = [
      "RUST_LOG=info"
    ];
    ExposedPorts = {
      "3000" = {};
    };
    Volumes = {
      "/listen" = {};
    };
  };
}