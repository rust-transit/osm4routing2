extern crate csv;
use lib::models::*;
use std;


pub fn write(nodes: Vec<Node>, edges: Vec<Edge>) {
    let edges_path = std::path::Path::new("edges.csv");
    let mut edges_csv = csv::Writer::from_file(edges_path).unwrap();
    edges_csv.encode(vec!["id",
                     "source",
                     "target",
                     "length",
                     "foot",
                     "car_forward",
                     "car_backward",
                     "bike_forward",
                     "bike_backward",
                     "wkt"])
        .expect("CSV: unable to write edge header");
    for edge in edges {
        edges_csv.encode((edge.id,
                     edge.source,
                     edge.target,
                     edge.length(),
                     edge.properties.foot,
                     edge.properties.car_forward,
                     edge.properties.car_backward,
                     edge.properties.bike_forward,
                     edge.properties.bike_backward,
                     edge.as_wkt()))
            .expect("CSV: unable to write edge");
    }

    let nodes_path = std::path::Path::new("nodes.csv");
    let mut nodes_csv = csv::Writer::from_file(nodes_path).unwrap();
    nodes_csv.encode(vec!["id", "lon", "lat"]).expect("CSV: unable to write node header");
    for node in nodes {
        nodes_csv.encode((node.id, node.coord.lon, node.coord.lat))
            .expect("CSV: unable to write node");
    }
}
