#!/usr/bin/env sh

SERVER="ws://127.0.0.1:8001/ws/new_new"

GUID=`(echo '{"cmd": "show_server_status"}';sleep 0.1 )| websocat $SERVER | jq -r .games[0].game.game_id`

OPERATION='{"SetSeed":[3, 32, 3, 32, 54, 1, 84, 3, 32, 54, 1, 84, 3, 32, 65, 1, 84, 3, 32, 64, 1, 44, 3, 32, 54, 1, 84, 3, 32, 65, 1, 44]}'

echo '{"cmd": "debug_game", "game_id":"'$GUID'" , "operation": '$OPERATION'}' | websocat $SERVER
