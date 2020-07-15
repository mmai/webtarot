#!/usr/bin/env sh

# SERVER="ws://tarot.rhumbs.fr/ws"
SERVER="ws://127.0.0.1:8001/ws"

UUID="25d0ca15-225e-40e5-9485-789d0b1529e0"
GUID="none"
# UUID="toto"

(echo '{"cmd": "show_server_status"}';sleep 0.1) | websocat $SERVER/$GUID"_"$UUID | jq
