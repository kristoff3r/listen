#!/usr/bin/env bash

set -eu -o pipefail

cd "$(dirname "$0")"

rm -rf ./dev-certificates || true
