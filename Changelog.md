# Changelog

## 0.7.6
- front: chat box button
- fix points calculation for poignée & petit + 21

## 0.7.5
- cards translations
- translate chat messages
- players chatbox replace taker indicator
- enhance bot play

## 0.7.4
- fix bots
- add screenshot to readme

## 0.7.3
- fix bots
- abstract bot io
- maj doc packaging
- don't build client from nix

## 0.7.2
- fix front build
- nix module : fix nginx proxy headers

## 0.7.1
─ fix archives option
─ fix nix ssl dependencies

## 0.7.0
- bots

## 0.6.3
- UX : connexions / deconnexions

## 0.6.1
- invitation link
- some french translations
- basic command line bot player

## 0.6.0

- added slam
- added announces (poignées)
- upgraded confirmation handling to an events system

## 0.5.3

- removed cards played in chat
- show played card names next to players names
- fix score for 3 & 4 players games
- fix dog on garde sans / garde contre for 3 & 4 players games

## 0.5.2

- petit au bout 
- check dog cards
- UI larger dog

## 0.5.1

- fix excuse not mandatory after trump
- fix bidding buttons
- remove erroneous points on players (on mouse over)
- fix dog cards dealing at 3 or 4 players games

## 0.5.0

- variants support

## 0.4.0

- webgame_server & webgame_protocol extracted to an external repository

## 0.3.4

- added ip address parameter to server command

## 0.3.0

- allow player to disconnect and reconnect without losing the game session
- internationalization ; french translation
- fix trick with no trumps

## 0.2.6

- keep websocket alive : server sends ping every 55s (default nginx timeout = 60s)
- show dog at deal end
- show points margin to contract at deal end
- extended server status
- message 'your turn to play!'
- bidding stops after everybody talked
- show players names instead of position

## 0.2.5

- keep alive websocket

## 0.2.3

- maj static files

## 0.2.2

- new deal when all players pass
- scores styling

## 0.2.1

- nix front packaging

## 0.2.0

- sounds
- error messages
- fix server crashing when playing at bidding phase

## 0.1.1

- colors

## 0.1.0

- initial release : basic game is playable
