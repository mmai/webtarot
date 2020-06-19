# { callPackage, fetchFromGithub, stdenv }:
{ pkgs ? import <nixpkgs> {} }:

let 

mkRustPlatform = pkgs.callPackage ./mk-rust-platform.nix {};
rustPlatform   = mkRustPlatform { date = "2020-03-08"; channel = "nightly"; };

in 

rustPlatform.buildRustPackage rec {
  pname = "webtarot";
  version = "0.2.4";

  src = pkgs.fetchFromGitHub {
    owner = "mmai";
    repo = pname;
    rev = "v${version}";
    sha256 = "1v5pzg1arj2z9kwvkcnlzwgfghrjqcy6jh3vxvpd3gaylf4hrhz2";
  };
  # src = ./.;

  postInstall = ''
    mkdir -p $out
    cp -R ./webtarot_client/static $out/front
    cp ./webtarot_client/dist/*.{css,js,wasm} $out/front
    '';

  cargoSha256 = "15j4afwx5hqj770nyczcla0qf86g6fqcnd3hmj264pp9nw5j695w";

  meta = with pkgs.stdenv.lib; {
    description = "A online game of french tarot";
    homepage = "https://github.com/mmai/webtarot";
    license = licenses.gpl3;
    platforms = platforms.unix;
    maintainers = with maintainers; [ mmai ];
  };
}
