#!/usr/bin/env bash

PGPASSWORD=postgres psql -h localhost -U postgres --dbname=listen --file=seeds.sql