#!/usr/bin/env bash

PGPASSWORD=postgres psql -h localhost -p 5433 -U postgres -d listen -c 'update users set is_approved=true'
