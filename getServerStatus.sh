#!/usr/bin/env sh

SERVER="ws://127.0.0.1:8001/ws"

(echo '{"cmd": "show_server_status"}';sleep 0.1) | websocat $SERVER | jq
