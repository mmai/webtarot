#!/usr/bin/env sh

SERVER="ws://127.0.0.1:8001/ws/new_new"

GUID=`(echo '{"cmd": "show_server_status"}';sleep 0.1 )| websocat $SERVER | jq -r .games[0].game.game_id`

SET_SEED_10TRUMPS='{"SetSeed":[255, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]}'

echo '{"cmd": "debug_game", "game_id":"'$GUID'" , "operation": '$SET_SEED_10TRUMPS'}' | websocat $SERVER
