{
  description = "Webtarot";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    # flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay }:
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
    in
    {
      overlay = final: prev: {

        webtarot-front =
          let
            rustPlatform = final.rustPlatform;
            # Use the same version as in Cargo.lock
            wasm-bindgen-version = "0.2.114";
            wasm-bindgen-cli = final.buildWasmBindgenCli rec {
              version = wasm-bindgen-version;
              src = final.fetchCrate {
                pname = "wasm-bindgen-cli";
                inherit version;
                hash = "sha256-xrCym+rFY6EUQFWyWl6OPA+LtftpUAE5pIaElAIVqW0=";
              };
              cargoDeps = rustPlatform.fetchCargoVendor {
                inherit src;
                name = "wasm-bindgen-cli-vendor";
                hash = "sha256-Z8+dUXPQq7S+Q7DWNr2Y9d8GMuEdSnq00quUR0wDNPM=";
              };
            };

            frontendCargoDeps = rustPlatform.fetchCargoVendor {
              src = ./.;
              name = "webtarot-frontend-vendor";
              hash = "sha256-J00yzaK80YHAO60aXWWgsHiGCpfXG0u3vAlhD3JY74s=";
            };
          in
          final.stdenv.mkDerivation {
            name = "webtarot-front";
            src = ./.;

            nativeBuildInputs = with final; [
              cargo
              rustc
              lld
              rustPlatform.cargoSetupHook
              wasm-bindgen-cli
              trunk
              dart-sass
              binaryen
            ];

            cargoDeps = frontendCargoDeps;

            buildPhase = ''
              runHook preBuild
              export HOME=$TMPDIR

              # Tell trunk to use the system-installed tool versions from PATH
              cat >> webtarot_client/Trunk.toml << 'EOF'

              [tools]
              wasm-bindgen = { version = "${wasm-bindgen-version}" }
              wasm-opt = { version = "version_124" }
              EOF

              pushd webtarot_client
              sass --style compressed scss/webtarot.scss > static/webtarot.css
              trunk build --release --offline
              popd

              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p $out
              cp -R webtarot_client/dist/. $out/
              runHook postInstall
            '';
          };

        webtarot = with final; (rustPlatform.buildRustPackage rec {
          name = "webtarot";
          version = "0.8.0";
          src = ./.;

          nativeBuildInputs = [ pkg-config ];
          buildInputs = [ openssl ];

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "webgame_protocol-0.8.0" = "sha256-Igu2w3OJBoYhSJ5LdoWIbt8taSD8VnNiKl8uB4zHeb0=";
            };
          };

          meta = with pkgs.lib; {
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
            entrypoint = writeScript "entrypoint.sh" ''
              #!${stdenv.shell}
              IP=$(ip route get 1 | awk '{print $NF;exit}')
              echo "Starting server. Open your client on http://$IP:${port}"
              mkdir -p /tmp
              ${webtarot}/bin/webtarot_server -d ${webtarot-front}/ -a $IP -p ${port}
            '';
          in
          dockerTools.buildImage {
            name = "mmai/webtarot";
            tag = "latest";
            # contents = [ busybox ];
            copyToRoot = buildEnv {
              name = "busybox";
              paths = [ busybox ];
            };
            config = {
              Entrypoint = [ entrypoint ];
              ExposedPorts = {
                "${port}/tcp" = { };
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


      devShell =
        let
          inherit (nixpkgs) lib;
          legacyPackages = lib.attrsets.mapAttrs (system: pkgs: pkgs.extend rust-overlay.overlays.default) nixpkgs.legacyPackages;
        in
        forAllSystems (system: (import ./shell.nix {
          pkgs = legacyPackages.${system};
        }));

      # webtarot service module
      nixosModule = (import ./module.nix);

    };
}
