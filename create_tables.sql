DROP TABLE IF EXISTS edges;
DROP TABLE IF EXISTS nodes;

CREATE TABLE nodes (
    id BIGINT PRIMARY KEY,
    longitude DOUBLE PRECISION,
    latitude DOUBLE PRECISION
);

CREATE TABLE edges (
    id BIGINT,
    source BIGINT REFERENCES nodes(id),
    target BIGINT REFERENCES nodes(id),
    length REAL,
    foot INT,
    car_forward INT,
    car_backward INT,
    bike_forward INT,
    bike_backward INT,
    wkt TEXT
);

