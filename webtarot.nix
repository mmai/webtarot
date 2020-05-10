# { callPackage, fetchFromGithub, stdenv }:
{ pkgs ? import <nixpkgs> {} }:

let 

mkRustPlatform = pkgs.callPackage ./mk-rust-platform.nix {};
rustPlatform   = mkRustPlatform { date = "2020-03-08"; channel = "nightly"; };

in 

rustPlatform.buildRustPackage rec {
  pname = "webtarot";
  version = "0.0.2";

  # src = pkgs.fetchFromGitHub {
  #   owner = "mmai";
  #   repo = pname;
  # # rev = version;
  #   rev = "9f2124947c932bb9a6b12ab7c4007283a846b6f4";
  #   sha256 = "0h9bfq6mp1hnmk7fjvcphn7kd9ngq84x0bjlcdnxpjmh4jcvjykn";
  # };
  src = ./.;

  cargoSha256 = "1c7p8vmfd1hdv7nr27rak0pis9an4ffgcrs6qjyk5nrrnrc5n25y";

  meta = with pkgs.stdenv.lib; {
    description = "A online game of french tarot";
    homepage = "https://github.com/mmai/webtarot";
    license = licenses.gpl3;
    platforms = platforms.unix;
    maintainers = with maintainers; [ mmai ];
  };
}
