# Dev

## Packaging

* edit the _Cargo.toml_ files : version number
* compile release version : `make` 
* edit _flake.nix_ : 
  * version number
  * fake cargoSha256
* get correct cargoSha256 value by running `nix build .#webtarot`
* fix cargoSha256 in flake.nix and rerun `nix build .#webtarot`
* `cachix push mmai ./result`
* `nix copy  --to ssh://root@rhumbs.fr ./result`
* git flow release
* `make docker`
