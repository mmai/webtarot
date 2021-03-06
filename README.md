# webtarot

A online game of [french tarot](https://en.wikipedia.org/wiki/French_tarot) in rust / webassembly

The project is in its early stages. It is playable but still lacks a lot of features and it may be unstable. 
(I am learning rust with this project, don't hesitate to make pull requests)

## Docker image

You can test the application using its docker image:

```
docker pull mmai/webtarot:0.5.2
docker run mmai/webtarot:0.5.2
```

## Manual installation

Compile application: 

```sh
make
```

Start server:

```sh
./dist/webtarot_server -p 8000 -d dist/public
```

then open [http://127.0.0.1:8000/](http://127.0.0.1:8000/).


If you want to see logs:
```sh
RUST_LOG=info ./dist/webtarot_server -p 8000 -d dist/public
```

## Development

Start server and client in developpement mode:

```sh
make client
make server
firefox http://127.0.0.1:8001/
```

## Internationalization

Requirements: 
- gettext
- xtr (`cargo install xtr`)
- cargo-i18n (`cargo install cargo-i18n`)

Edit target languages in _i18n.toml_ and _webtarot_client/i18n.toml_

Generate translation files: `cd webtarot_client && cargo i18n` (you can ignore error messages about parent crate)

Translate strings in webtarot_client/i18n/po/your_language/

Compile translations 

```sh
cd webtarot_client
cargo i18n
cargo build
```

## Thanks

* the initial code for the base game server and yew (webassembly) stuff was taken from https://github.com/mitsuhiko/webgame
* the card game models where inspired by https://github.com/gyscos/libcoinche
* the cards SVG images come from https://github.com/tarotclub/tarotclub

