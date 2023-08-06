{ pkgs, ... }:

{
  env.NODE_OPTIONS = "--openssl-legacy-provider";
  packages = with pkgs; [
    nodejs
    yarn
    python2 # for node-sass 
    wasm-pack
    # cargo
    pkg-config openssl_1_1 # fix the "failed to run custom build command for openssl-sys" error
  ];

  enterShell = ''
# cargo install cargo-i18n
  '';

# XXX : ne fonctionne pas, utiliser `make dev`

  # https://devenv.sh/languages/
  languages.rust.enable = true;
  languages.rust.toolchain.rustc = pkgs.rustc.override {
      stdenv = pkgs.stdenv.override{
        targetPlatform.isRedox = false;
        targetPlatform.isMusl = false;
        targetPlatform.isStatic = false;
        targetPlatform.parsed = {
          cpu = { name = "wasm32"; };
          vendor = {name = "unknown";};
          kernel = {name = "unknown";};
          abi = {name = "unknown";};
        };
      };
    };
}
