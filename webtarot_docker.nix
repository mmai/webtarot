{ pkgs ? import <nixpkgs> {} }:

with pkgs;
let
  port = "8080";
  webtarot = (import ./webtarot.nix) { pkgs = pkgs; };
  entrypoint = writeScript "entrypoint.sh" ''
    #!${stdenv.shell}
    IP=$(ip route get 1 | awk '{print $NF;exit}')
    echo "Starting server. Open your client on http://$IP:${port}"
    ${webtarot}/bin/webtarot_server -d ${webtarot}/front/ -a $IP -p ${port}
  '';
in
  dockerTools.buildImage {
    name = "mmai/webtarot";
    tag = "0.3.4";
    contents = [ busybox ];
    config = {
      Entrypoint = [ entrypoint ];
      ExposedPorts = {
        "${port}/tcp" = {};
      };
    };
  }
