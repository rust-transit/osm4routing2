DROP TABLE IF EXISTS edges;
DROP TABLE IF EXISTS nodes;
DROP TYPE IF EXISTS accessibility;

-- Could be seperated into multiple enum types, this was a quick enough fix
CREATE TYPE accessibility AS ENUM ('Unknown', 'Forbidden', 'Allowed', 'Residential', 'Tertiary', 'Secondary', 'Primary', 'Trunk', 'Motorway', 'Lane', 'Busway', 'Track');

CREATE TABLE nodes (
    id BIGINT PRIMARY KEY,
    longitude DOUBLE PRECISION,
    latitude DOUBLE PRECISION
);

--id,osm_id,source,target,length,foot,car_forward,car_backward,bike_forward,bike_backward,train,wkt

CREATE TABLE edges (
    id TEXT,
	osm_id BIGINT,
    source BIGINT REFERENCES nodes(id),
    target BIGINT REFERENCES nodes(id),
    length REAL,
    foot accessibility,
    car_forward accessibility,
    car_backward accessibility,
    bike_forward accessibility,
    bike_backward accessibility,
	train accessibility,
    wkt TEXT
);

