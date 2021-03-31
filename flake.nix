{
  description = "Webtarot";

  inputs.nixpkgs.url = github:NixOS/nixpkgs/nixos-20.09;

  outputs = { self, nixpkgs }:
  let
    systems = [ "x86_64-linux" "i686-linux" "aarch64-linux" ];
    forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system); 
    # Memoize nixpkgs for different platforms for efficiency.
    nixpkgsFor = forAllSystems (system:
      import nixpkgs {
        inherit system;
        overlays = [ self.overlay ];
      }
    );
  in {
    overlay = final: prev: {

      webtarot-front = final.stdenv.mkDerivation {
        name = "webtarot-front";
        src = ./webtarot_client;
        installPhase = ''
          mkdir -p $out
          cp -R ./static/* $out
          cp ./dist/*.{css,js,wasm} $out
        '';
      };

      webtarot = with final; ( rustPlatform.buildRustPackage rec {
          name = "webtarot";
          version = "0.7.0";
          src = ./.;

          nativeBuildInputs = [ pkgconfig ];
          buildInputs = [ openssl ];

          cargoSha256 = "sha256-zyB/tRzqsn0oURYiiIodLMO6h+436wV4uQtSPz9aGVM=";

          meta = with pkgs.stdenv.lib; {
            description = "A online game of french tarot";
            homepage = "https://github.com/mmai/webtarot";
            license = licenses.gpl3;
            platforms = platforms.unix;
            maintainers = with maintainers; [ mmai ];
          };
        });

      webtarot-docker = with final;
        let
          port = "8080";
          data_path = "/var/webtarot";
          db_uri = "${data_path}/webtarot_db";
          runAsRoot = ''
            mkdir -p ${data_path}/archives
          '';
          entrypoint = writeScript "entrypoint.sh" ''
            #!${stdenv.shell}
            IP=$(ip route get 1 | awk '{print $NF;exit}')
            echo "Starting server. Open your client on http://$IP:${port}"
            ${webtarot}/bin/webtarot_server -d ${webtarot-front}/ -a $IP -p ${port} -u ${db_uri} --archives-directory ${data_path}/archives
          '';
        in 
          dockerTools.buildImage {
            name = "mmai/webtarot";
            tag = "0.7.0";
            contents = [ busybox ];
            config = {
              Entrypoint = [ entrypoint ];
              ExposedPorts = {
                "${port}/tcp" = {};
              };
            };
          };

    };

    packages = forAllSystems (system: {
      inherit (nixpkgsFor.${system}) webtarot;
      inherit (nixpkgsFor.${system}) webtarot-front;
      inherit (nixpkgsFor.${system}) webtarot-docker;
    });

    defaultPackage = forAllSystems (system: self.packages.${system}.webtarot);

    devShell = forAllSystems (system: (import ./shell.nix { pkgs = nixpkgs.legacyPackages.${system}; }));

    # webtarot service module
    nixosModule = (import ./module.nix);

  };
}
