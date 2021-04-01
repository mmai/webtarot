# Dev

## Upgrading yew / webpack

Base template, à regarder régulièrement pour les mise à jour de dépendences et la config webpack : https://github.com/yewstack/yew-wasm-pack-template

Problem : dependabot found vulnerabilities on node-forge, needs 0.10.0 but webpack-dev-server needs 0.9 => we need to upgrade webpack-dev-server dependencies

```sh
cd webtarot_client
yarn upgrade --depth
```

## Test bots
```sh
nix develop
cd webtarot_bot
cargo run -- --join_code KSGWGW
```

## Packaging


* edit the _Cargo.toml_ files : version number
* compile release version : `make` 
  * attention, le build du front peut être cassé si lancé depuis le shell nix à cause de la version de rustc : vérifier les erreurs du debut. Il est peut-être nécessaire de lancer séparément le build du front et le build du serveur (celui-ci devant être lancé depuis le shell nix pour profiter des lib ssl nécessaires pour webtarot_bot)
* edit _flake.nix_ : 
  * version number
  * fake cargoSha256
* get correct cargoSha256 value by running `nix build .#webtarot`
* fix cargoSha256 in flake.nix 
* git flow release
* rerun `nix build .#webtarot`
* `cachix push mmai ./result`
* `nix copy  --to ssh://root@rhumbs.fr ./result`
* `make docker` :  does not work ?
