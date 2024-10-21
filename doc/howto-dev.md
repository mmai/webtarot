# Dev

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
* git push && git push --tags
* rerun `nix build .#webtarot`
* `cachix push mmai ./result`
* `nix copy  --to ssh://root@rhumbs.fr ./result`
* `make docker` :  does not work ?

## Upgrading yew / webpack

Base template, à regarder régulièrement pour les mise à jour de dépendences et la config webpack : https://github.com/yewstack/yew-wasm-pack-template

Problem : dependabot found vulnerabilities on node-forge, needs 0.10.0 but webpack-dev-server needs 0.9 => we need to upgrade webpack-dev-server dependencies

```sh
cd webtarot_client
yarn upgrade --depth
```

## Diagnostics

Display games list
```sh
(echo '{"cmd": "show_server_status"}';sleep 0.1 )| websocat ws://127.0.0.1:8001/ws/new_new | jq 
(echo '{"cmd": "show_server_status"}';sleep 0.1 )| websocat ws://tarot.rhumbs.fr/ws/new_new | jq 
``` 

Print a specific game_state in server logs :

```sh
echo '{"cmd": "debug_game", "game_id":"'$GUID'" , "operation": "ShowState"}' | websocat  ws://127.0.0.1:8001/ws/new_new | jq '.data | fromjson'

```

## Bots

Launch bots from command line:
```sh
nix develop
cd webtarot_bot
cargo run -- --join_code KSGWGW --count 4
```

Only bots plays against each others: 

```sh
cargo run
```

## Compilation fixes

Demande un `use strum::IntoEnumIterator` alors qu'il y est déjà puis se plaint qu'il y soit...

=> différence de version entre le strum utilisé par webtarot_client et celui de tarotgame => edit Cargo.toml pour uniformiser

```
error[E0599]: no variant or associated item named `iter` found for enum `tarotgame::bid::Target` in the current scope
  --> webtarot_client/src/components/bidding_actions.rs:80:38
   |
80 |                     for bid::Target::iter()
   |                                      ^^^^ variant or associated item not found in `tarotgame::bid::Target`
   |
   = help: items from traits can only be used if the trait is in scope
   = note: the following trait is implemented but not in scope; perhaps add a `use` for it:
           `use strum::IntoEnumIterator;`
   = note: this error originates in a macro (in Nightly builds, run with -Z macro-backtrace for more info)

warning: unused import: `strum::IntoEnumIterator`
 --> webtarot_client/src/components/bidding_actions.rs:3:5
  |
3 | use strum::IntoEnumIterator;
  |     ^^^^^^^^^^^^^^^^^^^^^^^
```
