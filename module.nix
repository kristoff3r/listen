{ config, lib, ... }:

let
  cfg = config.services.listen;
in

with lib;

{
  options = {
    services.listen = {
      enable = mkEnableOption "enable the listen service";
      version = mkOption {
        default = "master";
        type = with lib.types; str;
        description = ''
          Which docker tag to use for the main container
        '';
      };
      domain = mkOption {
        type = with lib.types; str;
        description = ''
          Hostname the service is being served on
        '';
      };
    };
  };

  config = lib.mkIf (cfg.enable) {
    systemd.services.create-listen-pod = {
      serviceConfig.Type = "oneshot";
      wantedBy = [
        "podman-db.service"
        "podman-listen-api.service"
      ];
      script = ''
        ${pkgs.podman}/bin/podman pod exists listen || \
          ${pkgs.podman}/bin/podman pod create -n listen -p '127.0.0.1:3000:3000'
      '';
    };
    virtualisation.oci-containers.backend = "podman";
    virtualisation.oci-containers.containers = {
      db = {
        image = "postgres:16.4";
        environment = {
          POSTGRES_USER="postgres";
          POSTGRES_PASSWORD="postgres";
          POSTGRES_DB="listen";
        };

        extraOptions = [
          "--pod" "listen"
        ];
      };
      listen-api = {
        image = "listen-api:nix";
        ports = [
          "127.0.0.1:3000:3000"
        ];
        environment = {
          RUST_LOG="info";
        };
        volumes = [
          "/var/lib/listen:/listen"
        ];
      };
    };

    services.nginx = import ../nginx.nix {
      logError = "stderr debug";
      appendHttpConfig = ''
        chunked_transfer_encoding on;
      '';

      virtualHosts."${cfg.domain}" =
        {
          # locations."/" = {
          #   basicAuth."${cfg.username}" = cfg.password;
          #   root = "/var/lib/live-frontend";
          #   extraConfig = ''
          #     try_files $uri $uri/ /index.html;
          #   '';
          # };
          locations."/" = {
            proxyPass = "http://127.0.0.1:3000";
            proxyWebsockets = true;
          };
        };
    };
  };
}