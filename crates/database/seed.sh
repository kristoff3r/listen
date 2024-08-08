#!/usr/bin/env bash

PGPASSWORD=postgres psql -h localhost -p 5433 -U postgres --dbname=listen --file=seeds.sql
