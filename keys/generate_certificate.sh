#!/usr/bin/env bash

set -eu -o pipefail

openssl req -x509 -out ./dev.listen.pwnies.dk.crt -keyout ./dev.listen.pwnies.dk.key -newkey rsa:2048 -nodes -sha256   -subj '/CN=dev.listen.pwnies.dk' -extensions EXT -config <( \
   printf "[dn]\nCN=dev.listen.pwnies.dk\n[req]\ndistinguished_name = dn\n[EXT]\nsubjectAltName=DNS:dev.listen.pwnies.dk\nkeyUsage=digitalSignature\nextendedKeyUsage=serverAuth")
