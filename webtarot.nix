# { callPackage, fetchFromGithub, stdenv }:
{ pkgs ? import <nixpkgs> {} }:

let 

mkRustPlatform = pkgs.callPackage ./mk-rust-platform.nix {};
rustPlatform   = mkRustPlatform { date = "2020-03-08"; channel = "nightly"; };

in 

rustPlatform.buildRustPackage rec {
  pname = "webtarot";
  version = "0.3.4";

  src = pkgs.fetchFromGitHub {
    owner = "mmai";
    repo = pname;
    rev = "v${version}";
    sha256 = "1ypyk9jvn1vjgnj64jydcksm0gmxc5km094qpafqys0npncng723";
  };
  # src = ./.;

  postInstall = ''
    mkdir -p $out
    cp -R ./webtarot_client/static $out/front
    cp ./webtarot_client/dist/*.{css,js,wasm} $out/front
    '';

  cargoSha256 = "0a4fwmxamqrcfxr7jmi1s649j91rnsch3gzxhqn75cf9cm9by00s";

  meta = with pkgs.stdenv.lib; {
    description = "A online game of french tarot";
    homepage = "https://github.com/mmai/webtarot";
    license = licenses.gpl3;
    platforms = platforms.unix;
    maintainers = with maintainers; [ mmai ];
  };
}
