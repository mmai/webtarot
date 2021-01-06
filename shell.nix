{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell {
  # Set Environment Variables
  RUST_BACKTRACE = 1;

  buildInputs = [
    rustc cargo pkgconfig openssl
  ];

}
