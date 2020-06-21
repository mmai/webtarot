#!/usr/bin/env sh

SERVER="ws://127.0.0.1:8001/ws"

UUID=`(echo '{"cmd": "show_uuid"}';sleep 0.1 )| websocat $SERVER | jq -r .player_id`
# UUID="53f41803-eedf-4f08-a3d0-4bc01dee5aa2"

# snapshots
init='{"type":"game_state_snapshot","players":[{"player":{"id":"'$UUID'","nickname":"aa"},"pos":"P0","role":"spectator","ready":false}],"turn":"Pregame","deal":{"hand":[18295981441286144,524529],"current":"P0","contract":null,"scores":[0.0,0.0,0.0,0.0,0.0],"last_trick":{"cards":[null,null,null,null,null],"first":"P0","winner":"P0"},"initial_dog":[0,0]},"scores":[]}'
bidding='{"type":"game_state_snapshot","players":[{"player":{"id":"'$UUID'","nickname":"bb"},"pos":"P0","role":"pre_deal","ready":true},{"player":{"id":"6abeaa52-24f5-4810-8d20-eedf2eff6f98","nickname":"bb"},"pos":"P1","role":"pre_deal","ready":true},{"player":{"id":"a298f6dc-acf3-47ab-a618-4b6d7e044097","nickname":"VUT-QHF"},"pos":"P2","role":"pre_deal","ready":true},{"player":{"id":"6c32ed3e-e809-437f-a973-16b8f7842842","nickname":"ccc"},"pos":"P3","role":"pre_deal","ready":true},{"player":{"id":"dbfcbfab-223a-4b4e-ba48-c15fcaf492cd","nickname":"ddd"},"pos":"P4","role":"pre_deal","ready":true}],"turn":{"Bidding":["Bidding","P0"]},"deal":{"hand":[5085258058106,17334336],"current":"P0","contract":null,"scores":[0.0,0.0,0.0,0.0,0.0],"last_trick":{"cards":[null,null,null,null,null],"first":"P0","winner":"P0"},"initial_dog":[0,0]},"scores":[]}'
callking='{"type":"game_state_snapshot","players":[{"player":{"id":"'$UUID'","nickname":"bb"},"pos":"P0","role":"taker","ready":true},{"player":{"id":"6abeaa52-24f5-4810-8d20-eedf2eff6f98","nickname":"bb"},"pos":"P1","role":"pre_deal","ready":true},{"player":{"id":"a298f6dc-acf3-47ab-a618-4b6d7e044097","nickname":"VUT-QHF"},"pos":"P2","role":"pre_deal","ready":true},{"player":{"id":"6c32ed3e-e809-437f-a973-16b8f7842842","nickname":"ccc"},"pos":"P3","role":"pre_deal","ready":true},{"player":{"id":"dbfcbfab-223a-4b4e-ba48-c15fcaf492cd","nickname":"ddd"},"pos":"P4","role":"pre_deal","ready":true}],"turn":"CallingKing","deal":{"hand":[5085258058106,17334336],"current":"P0","contract":{"author":"P0","target":"Garde"},"scores":[0.0,0.0,0.0,0.0,0.0],"last_trick":{"cards":[null,null,null,null,null],"first":"P0","winner":"P0"},"initial_dog":[0,0]},"scores":[]}'
makedog='{"type":"game_state_snapshot","players":[{"player":{"id":"'$UUID'","nickname":"bb"},"pos":"P0","role":"taker","ready":true},{"player":{"id":"6abeaa52-24f5-4810-8d20-eedf2eff6f98","nickname":"bb"},"pos":"P1","role":"pre_deal","ready":true},{"player":{"id":"a298f6dc-acf3-47ab-a618-4b6d7e044097","nickname":"VUT-QHF"},"pos":"P2","role":"pre_deal","ready":true},{"player":{"id":"6c32ed3e-e809-437f-a973-16b8f7842842","nickname":"ccc"},"pos":"P3","role":"pre_deal","ready":true},{"player":{"id":"dbfcbfab-223a-4b4e-ba48-c15fcaf492cd","nickname":"ddd"},"pos":"P4","role":"pre_deal","ready":true}],"turn":"MakingDog","deal":{"hand":[5085258058106,17334336],"current":"P0","contract":{"author":"P0","target":"Garde"},"scores":[0.0,0.0,0.0,0.0,0.0],"last_trick":{"cards":[null,null,null,null,null],"first":"P0","winner":"P0"},"initial_dog":[4194816,128]},"scores":[]}'
playing='{"type":"game_state_snapshot","players":[{"player":{"id":"'$UUID'","nickname":"bb"},"pos":"P0","role":"opponent","ready":true},{"player":{"id":"6abeaa52-24f5-4810-8d20-eedf2eff6f98","nickname":"bb"},"pos":"P1","role":"opponent","ready":true},{"player":{"id":"a298f6dc-acf3-47ab-a618-4b6d7e044097","nickname":"VUT-QHF"},"pos":"P2","role":"taker","ready":true},{"player":{"id":"6c32ed3e-e809-437f-a973-16b8f7842842","nickname":"ccc"},"pos":"P3","role":"partner","ready":true},{"player":{"id":"dbfcbfab-223a-4b4e-ba48-c15fcaf492cd","nickname":"ddd"},"pos":"P4","role":"opponent","ready":true}],"turn":{"Playing":"P4"},"deal":{"hand":[5085258058090,557248],"current":"P4","contract":{"author":"P0","target":"Garde"},"scores":[0.0,0.0,0.0,0.0,0.0],"last_trick":{"cards":[[0,16777216],[0,4194304],[0,8388608],[0,262144],null],"first":"P0","winner":"P0"},"initial_dog":[0,0]},"scores":[]}'
meplaying='{"type":"game_state_snapshot","players":[{"player":{"id":"'$UUID'","nickname":"aa"},"pos":"P0","role":"pre_deal","ready":true},{"player":{"id":"84aa2320-62e3-410e-bc28-d6c0ea2c5e9f","nickname":"bb"},"pos":"P1","role":"pre_deal","ready":true},{"player":{"id":"65d67ced-6e28-4795-9ccb-7171c2dafc59","nickname":"cc"},"pos":"P2","role":"pre_deal","ready":true},{"player":{"id":"be0c28cf-78a3-441f-bb2a-92877852f085","nickname":"dd"},"pos":"P3","role":"taker","ready":true},{"player":{"id":"96d11fce-152c-455e-b6d1-6eb70885b790","nickname":"ee"},"pos":"P4","role":"pre_deal","ready":true}],"turn":{"Playing":"P0"},"deal":{"hand":[11264548191995944,16777216],"current":"P0","contract":{"author":"P3","target":"Prise"},"scores":[0.0,0.0,0.0,0.0,0.0],"last_trick":{"cards":[null,null,null,[4294967296,0],[137438953472,0]],"first":"P3","winner":"P4"},"initial_dog":[0,0]},"scores":[]}'
endtrick='{"type":"game_state_snapshot","players":[{"player":{"id":"'$UUID'","nickname":"bb"},"pos":"P0","role":"taker","ready":false},{"player":{"id":"6abeaa52-24f5-4810-8d20-eedf2eff6f98","nickname":"bb"},"pos":"P1","role":"pre_deal","ready":false},{"player":{"id":"a298f6dc-acf3-47ab-a618-4b6d7e044097","nickname":"VUT-QHF"},"pos":"P2","role":"pre_deal","ready":false},{"player":{"id":"6c32ed3e-e809-437f-a973-16b8f7842842","nickname":"ccc"},"pos":"P3","role":"pre_deal","ready":false},{"player":{"id":"dbfcbfab-223a-4b4e-ba48-c15fcaf492cd","nickname":"ddd"},"pos":"P4","role":"pre_deal","ready":false}],"turn":"Intertrick","deal":{"hand":[5085258058090,557248],"current":"P0","contract":{"author":"P0","target":"Garde"},"scores":[0.0,0.0,0.0,0.0,0.0],"last_trick":{"cards":[[0,16777216],[0,4194304],[0,8388608],[0,262144],[0,131072]],"first":"P0","winner":"P0"},"initial_dog":[0,0]},"scores":[]}'
scores='{"type":"game_state_snapshot","players":[{"player":{"id":"'$UUID'","nickname":"aa"},"pos":"P0","role":"unknown","ready":false},{"player":{"id":"5d6da62b-33ca-4891-8060-164e9745d3b6","nickname":"bb"},"pos":"P1","role":"unknown","ready":false},{"player":{"id":"ece04b30-1763-4aa1-89fa-0a4c58ac32b7","nickname":"cc"},"pos":"P2","role":"unknown","ready":false},{"player":{"id":"ed821b0a-df0f-4f7c-b8ef-ff1a49e31802","nickname":"dd"},"pos":"P3","role":"unknown","ready":false},{"player":{"id":"c4c28940-81e5-432d-9b65-4df64b8c7bf6","nickname":"ee"},"pos":"P4","role":"unknown","ready":false}],"turn":"Interdeal","deal":{"hand":[0,0],"current":"P0","contract":{"author":"P2","target":"Garde"},"scores":[102.0,-102.0,204.0,-102.0,-102.0],"last_trick":{"cards":[[0,1048576],[16777216,0],[1099511627776,0],[33554432,0],[67108864,0]],"first":"P3","winner":"P0"},"initial_dog":[0,0]},"scores":[[102.0,-102.0,204.0,-102.0,-102.0]]}'

SNAPSHOT=$playing

echo '{"cmd": "debug_ui", "player_id":"'$UUID'" , "snapshot": '$SNAPSHOT'}' | websocat $SERVER
