{ pkgs, buildImage, listen-api }:

buildImage {
  name = "listen-api";
  tag = "nix";

  copyToRoot = pkgs.buildEnv {
    name = "image-root";
    paths = with pkgs; [
      procps
      bashInteractive
      coreutils
      curl

      yt-dlp
      ffmpeg
    ];
    pathsToLink = [ "/bin" ];
  };

  config = {
    Entrypoint = [ "${listen-api}/bin/backend" ];
    Env = [
      "RUST_LOG=info"
      "RUST_BACKTRACE=1"
    ];
    ExposedPorts = {
      "3000" = {};
    };
    Volumes = {
      "/videos" = {};
    };
  };
}
