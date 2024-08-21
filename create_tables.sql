DROP TABLE IF EXISTS edges;
DROP TABLE IF EXISTS nodes;

-- Create the 'nodes' table
CREATE TABLE nodes (
    id BIGINT PRIMARY KEY,
    longitude DOUBLE PRECISION,
    latitude DOUBLE PRECISION
);

-- Create the 'edges' table
CREATE TABLE edges (
    id TEXT,
    osm_id BIGINT,
    source BIGINT REFERENCES nodes(id),
    target BIGINT REFERENCES nodes(id),
    length REAL,
    foot VARCHAR(50),
    car_forward VARCHAR(50),
    car_backward VARCHAR(50),
    bike_forward VARCHAR(50),
    bike_backward VARCHAR(50),
    train VARCHAR(50),
    wkt TEXT
);

