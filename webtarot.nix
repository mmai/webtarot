# { callPackage, fetchFromGithub, stdenv }:
{ pkgs ? import <nixpkgs> {} }:

let 

mkRustPlatform = pkgs.callPackage ./mk-rust-platform.nix {};
rustPlatform   = mkRustPlatform { date = "2020-03-08"; channel = "nightly"; };

in 

rustPlatform.buildRustPackage rec {
  pname = "webtarot";
  version = "0.1.0";

  src = pkgs.fetchFromGitHub {
    owner = "mmai";
    repo = pname;
    rev = version;
    sha256 = "117angn2l8w39wzj07xz435mi2xmdh8xywhjm55mavjnj0yc70p0";
  };
  # src = ./.;

  cargoSha256 = "17kmp4p0aqndvp4k4gzrnk7dq7i2iaz3mkpinki83bxac2y6micj";

  meta = with pkgs.stdenv.lib; {
    description = "A online game of french tarot";
    homepage = "https://github.com/mmai/webtarot";
    license = licenses.gpl3;
    platforms = platforms.unix;
    maintainers = with maintainers; [ mmai ];
  };
}
