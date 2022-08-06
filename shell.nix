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
    nodejs
    yarn
    python2 # for node-sass 
    wasm-pack
    (rust-bin.stable.latest.default.override {
     extensions = [ "rust-src" ];
     targets = [ "wasm32-unknown-unknown" ];
     })
    # rustc 
    cargo
    pkgconfig openssl
  ];

}
