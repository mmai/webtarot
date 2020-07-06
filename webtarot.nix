# { callPackage, fetchFromGithub, stdenv }:
{ pkgs ? import <nixpkgs> {} }:

let 

mkRustPlatform = pkgs.callPackage ./mk-rust-platform.nix {};
rustPlatform   = mkRustPlatform { date = "2020-03-08"; channel = "nightly"; };

in 

rustPlatform.buildRustPackage rec {
  pname = "webtarot";
  version = "0.2.6";

  src = pkgs.fetchFromGitHub {
    owner = "mmai";
    repo = pname;
    rev = "v${version}";
    sha256 = "15fjj7wbvb9i04dclvhm9ynhlnw91wk4yln0ccra5bnpw0jamacr";
  };
  # src = ./.;

  postInstall = ''
    mkdir -p $out
    cp -R ./webtarot_client/static $out/front
    cp ./webtarot_client/dist/*.{css,js,wasm} $out/front
    '';

  cargoSha256 = "0x6x0hzgn7x2agw3ah3363pklhmiyacjaxk6fqs9dcan5zz7cs0f";

  meta = with pkgs.stdenv.lib; {
    description = "A online game of french tarot";
    homepage = "https://github.com/mmai/webtarot";
    license = licenses.gpl3;
    platforms = platforms.unix;
    maintainers = with maintainers; [ mmai ];
  };
}
