# Dev

## Packaging

* remove webtarot-bot from the root Cargo.toml file if necessary (error openssl / pkg-config)

* edit the _Cargo.toml_ files : version number
* compile release version : `make` 
* edit _flake.nix_ : 
  * version number
  * fake cargoSha256
* get correct cargoSha256 value by running `nix build .#webtarot`
* fix cargoSha256 in flake.nix 
* git flow release
* rerun `nix build .#webtarot`
* `cachix push mmai ./result`
* `nix copy  --to ssh://root@rhumbs.fr ./result`
* `make docker`

## Upgrading yew / webpack

Base template : https://github.com/yewstack/yew-wasm-pack-template

Problem : dependabot found vulnerabilities on node-forge, needs 0.10.0 but webpack-dev-server needs 0.9 => we need to upgrade webpack-dev-server dependencies

```sh
cd webtarot_client
yarn upgrade --depth
```
