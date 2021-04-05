{ pkgs ? import <nixpkgs> {} }:
# let
#   rust_channel = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;
# in 
with pkgs;
mkShell {
  # Set Environment Variables
  RUST_BACKTRACE = 1;

  # nativeBuildInputs = [
  #   rust_channel # Full rust from overlay, includes cargo
  #   nodePackages.npm # For all node packages
  #   wasm-pack # Compiling to WASM and packing with web-stuff
  # ];

  buildInputs = [
    rustc cargo
    pkgconfig openssl
  ];

}
