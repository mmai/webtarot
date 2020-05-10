# webtarot

A online game of tarot in rust / webassembly

Work in progress (not playable)

## Usage

Development mode 

```
make client
make server
firefox http://127.0.0.1:8001/
```

Production

```sh
make
RUST_LOG=info ./dist/webgame_server -p 8000 -d dist/public
firefox http://127.0.0.1:8000/
```
Or, if you want logs on production :
```sh
RUST_LOG=info ./dist/webgame_server -p 8000 -d dist/public
```

## Thanks

* the code for the base game server and yew (webassembly) stuff is taken from https://github.com/mitsuhiko/webgame
* the card game models are taken from https://github.com/gyscos/libcoinche
* the cards SVG images come from https://github.com/tarotclub/tarotclub

