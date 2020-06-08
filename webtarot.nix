# { callPackage, fetchFromGithub, stdenv }:
{ pkgs ? import <nixpkgs> {} }:

let 

mkRustPlatform = pkgs.callPackage ./mk-rust-platform.nix {};
rustPlatform   = mkRustPlatform { date = "2020-03-08"; channel = "nightly"; };

in 

rustPlatform.buildRustPackage rec {
  pname = "webtarot";
  version = "0.2.1";

  src = pkgs.fetchFromGitHub {
    owner = "mmai";
    repo = pname;
    rev = "v${version}";
    sha256 = "1nfcxc6kjmyk0j2krkr2bycfqw825xy4bcag3f9f18iv3zvqgz4d";
  };
  # src = ./.;

  postInstall = ''
    mkdir -p $out
    cp -R ./webtarot_client/static $out/front
    cp ./webtarot_client/dist/*.{js,wasm} $out/front
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
