# webtarot

A online game of tarot in rust / webassembly

Work in progress

Thanks
* the code for the base game server and yew (webassembly) stuff is taken from https://github.com/mitsuhiko/webgame
* the card game models are taken from https://github.com/gyscos/libcoinche

Development mode 

```
make client
make server
firefox http://127.0.0.1:8001/
```

Production

```
make
cd dist
./webgame_server
firefox http://127.0.0.1:8002/
```

