#!/usr/bin/env bash

set -eu -o pipefail

cd "$(dirname "$0")"

TMPDIR=$(mktemp -d /tmp/ca-cert-generation-XXXXXX)
trap "rm -rf $TMPDIR" EXIT QUIT TERM
chmod og-rwx $TMPDIR

HOST=dev.listen.pwnies.dk

mkdir dev-certificates
cd dev-certificates

# Generate root CA
cat <<EOF >${TMPDIR}/ca.conf
[ req ]
distinguished_name = dn
x509_extensions = v3_ca
prompt = no

[ dn ]
CN = Listen development Root CA

[ v3_ca ]
basicConstraints = critical, CA:TRUE, pathlen:0
keyUsage = critical, keyCertSign, cRLSign
subjectKeyIdentifier = hash
nameConstraints = critical, permitted;DNS:${HOST}, excluded;IP:0.0.0.0/0.0.0.0, excluded;IP:0:0:0:0:0:0:0:0/0:0:0:0:0:0:0:0, excluded;email:.*, excluded;URI:.*
EOF

openssl req -x509 -days 365 -out ${HOST}-ca.crt -keyout ${TMPDIR}/${HOST}-ca.key -newkey ec:<(openssl ecparam -name secp384r1) -nodes -sha256 -config ${TMPDIR}/ca.conf -extensions v3_ca

# Generate host keys and certificate
cat <<EOF >${TMPDIR}/host.conf
[req]
distinguished_name = req_dn
req_extensions = v3_req
prompt = no

[ req_dn ]
CN = ${HOST}

[ v3_req ]
basicConstraints = critical, CA:FALSE
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = DNS:${HOST}
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer
EOF

openssl req -x509 -days 365 -out ${HOST}.crt -keyout ${HOST}.key -newkey ec:<(openssl ecparam -name secp384r1) -nodes -sha256 -CAkey ${TMPDIR}/${HOST}-ca.key -CA ${HOST}-ca.crt -config ${TMPDIR}/host.conf -extensions v3_req

# Verify
openssl verify -CAfile ${HOST}-ca.crt ${HOST}.crt || true
openssl x509 -in ${HOST}-ca.crt -text -noout | grep "Version"
openssl x509 -in ${HOST}.crt -text -noout | grep "Version"
openssl x509 -in ${HOST}-ca.crt -text -noout | grep -A 7 "X509v3 Name Constraints"
