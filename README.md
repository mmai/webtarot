# webtarot

A online game of [french tarot](https://en.wikipedia.org/wiki/French_tarot) in rust / webassembly

The project is in its very early stages. It is somewhat playable but has an ugly UI, lacks a lot of features and is probably full of bugs.

Currently only the 5 players variant (the most fun) is implemented.

## Usage

Development mode 

```sh
make client
make server
firefox http://127.0.0.1:8001/
```

Production

```sh
make
./dist/webtarot_server -p 8000 -d dist/public
firefox http://127.0.0.1:8000/
```
Or, if you want logs on production :
```sh
RUST_LOG=info ./dist/webtarot_server -p 8000 -d dist/public
```

## Thanks

* the code for the base game server and yew (webassembly) stuff is taken from https://github.com/mitsuhiko/webgame
* the card game models where inspired by https://github.com/gyscos/libcoinche
* the cards SVG images come from https://github.com/tarotclub/tarotclub

