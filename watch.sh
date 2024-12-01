#!/usr/bin/env bash

set -eu -o pipefail

socat openssl-listen:3002,fork,reuseaddr,cert=./crates/backend/dev-certificates/dev.listen.pwnies.dk.crt,key=./crates/backend/dev-certificates/dev.listen.pwnies.dk.key,verify=0 tcp-connect:localhost:3001 &

trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

LEPTOS_RELOAD_WS_PROTOCOL="wss" LEPTOS_RELOAD_EXTERNAL_PORT="3002" cargo leptos watch "$@"
