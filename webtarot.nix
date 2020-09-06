# { callPackage, fetchFromGithub, stdenv }:
{ pkgs ? import <nixpkgs> {} }:

let 

mkRustPlatform = pkgs.callPackage ./mk-rust-platform.nix {};
rustPlatform   = mkRustPlatform { date = "2020-03-08"; channel = "nightly"; };

in 

rustPlatform.buildRustPackage rec {
  pname = "webtarot";
  version = "0.4.1";

  src = pkgs.fetchFromGitHub {
    owner = "mmai";
    repo = pname;
    rev = "v${version}";
    sha256 = "0h5llm636a01h8l70hp88z6pzkvzgy6imblwbvhwszzi2dx87iy4";
  };
  # src = ./.;

  postInstall = ''
    mkdir -p $out
    cp -R ./webtarot_client/static $out/front
    cp ./webtarot_client/dist/*.{css,js,wasm} $out/front
    '';

  cargoSha256 = "0qvk0d1v920rapwskj58kcc1dpz2j01lwcywj9krvc3mh42b5rv6";

  meta = with pkgs.stdenv.lib; {
    description = "A online game of french tarot";
    homepage = "https://github.com/mmai/webtarot";
    license = licenses.gpl3;
    platforms = platforms.unix;
    maintainers = with maintainers; [ mmai ];
  };
}
