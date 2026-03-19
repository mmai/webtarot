{ pkgs, ... }:

{
  packages = with pkgs; [
    openssl # fix the "failed to run custom build command for openssl-sys" error
    lld # needed by trunk for webtarot-client
    trunk # wasm builder
    dart-sass
  ];

  languages.rust.enable = true;
}
