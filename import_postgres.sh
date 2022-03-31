#!/bin/bash

# You can create the database using `createdb osm4routing`
database=osm4routing
psql $database -f create_tables.sql
psql $database -c "COPY nodes FROM STDIN CSV HEADER;" < nodes.csv
psql $database -c "COPY edges FROM STDIN CSV HEADER;" < edges.csv
