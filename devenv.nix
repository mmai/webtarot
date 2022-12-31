{ pkgs, ... }:

{
  env.NODE_OPTIONS = "--openssl-legacy-provider"
  packages = with pkgs; [
    nodejs
    yarn
    python2 # for node-sass 
    wasm-pack
    # (rust-bin.stable.latest.default.override {
    #  extensions = [ "rust-src" ];
    #  targets = [ "wasm32-unknown-unknown" ];
    #  })
    # rustc 
    # cargo
    pkg-config openssl_1_1
  ];

  enterShell = ''
  '';

  # https://devenv.sh/languages/
  languages.rust.enable = true;

  # https://devenv.sh/scripts/
  scripts.hello.exec = "echo hello from $GREET";

  # https://devenv.sh/pre-commit-hooks/
  pre-commit.hooks.shellcheck.enable = true;
}
