{config, lib, pkgs, ...}:

with lib;

let
  cfg = config.services.webtarot;
  webtarot = (import ./webtarot.nix) { pkgs = pkgs; };
in 
  {

    options = {
      services.webtarot = {
        enable = mkEnableOption "webtarot";

        user = mkOption {
          type = types.str;
          default = "webtarot";
          description = "User under which Webtarot is ran.";
        };

        group = mkOption {
          type = types.str;
          default = "webtarot";
          description = "Group under which Webtarot is ran.";
        };

        protocol = mkOption {
          type = types.enum [ "http" "https" ];
          default = "https";
          description = ''
            Web server protocol.
          '';
        };


        hostname = mkOption {
          type = types.str;
          default = "tarot.localhost";
          description = "Public domain name of the webtarot web app.";
        };

        apiPort = mkOption {
          type = types.port;
          default = 8002;
          description = ''
            Webtarot API Port.
          '';
        };

      };
    };

    config = mkIf cfg.enable {
      users.users.webtarot = mkIf (cfg.user == "webtarot") { group = cfg.group; };

      users.groups.webtarot = mkIf (cfg.group == "webtarot") {};

      services.nginx = {
        enable = true;
        appendHttpConfig = ''
          upstream webtarot-api {
          server localhost:${toString cfg.apiPort};
          }
        '';
        virtualHosts = 
        let proxyConfig = ''
          # global proxy conf
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
          proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
          proxy_set_header X-Forwarded-Proto $scheme;
          proxy_set_header X-Forwarded-Host $host:$server_port;
          proxy_set_header X-Forwarded-Port $server_port;
          proxy_redirect off;

          # websocket support
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection $connection_upgrade;
        '';
        withSSL = cfg.protocol == "http";
        in {
          "${cfg.hostname}" = {
            enableACME = withSSL;
            forceSSL = withSSL;
            locations = {
              "/" = { 
                extraConfig = proxyConfig;
                proxyPass = "http://webtarot-api/";
              };
            };
          };
        };
      };

      systemd.targets.webtarot = {
        description = "Webtarot";
        wants = ["webtarot-server.service"];
      }; 
      systemd.services = 
      let serviceConfig = {
        User = "${cfg.user}";
        WorkingDirectory = "${webtarot}";
      };
      in {
        webtarot-server = {
          description = "Webtarot application server";
          partOf = [ "webtarot.target" ];

          serviceConfig = serviceConfig // { 
            ExecStart = ''${webtarot}/bin/webtarot_server -d ${webtarot}/front/ \
              -p ${toString cfg.apiPort}'';
          };

          wantedBy = [ "multi-user.target" ];
        };
      };

    };

    meta = {
      maintainers = with lib.maintainers; [ mmai ];
    };
  }
