use super::error::Error;
use super::models::*;

pub fn csv(
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    nodes_file: &str,
    edges_file: &str,
) -> Result<(), Error> {
    let edges_path = std::path::Path::new(edges_file);
    let mut edges_csv = csv::Writer::from_path(edges_path)?;
    edges_csv.serialize(vec![
        "id",
        "osm_id",
        "source",
        "target",
        "length",
        "foot",
        "car_forward",
        "car_backward",
        "bike_forward",
        "bike_backward",
        "train",
        "wkt",
    ])?;
    for edge in edges {
        edges_csv.serialize((
            &edge.id,
            edge.osm_id.0,
            edge.source.0,
            edge.target.0,
            edge.length(),
            edge.properties.foot,
            edge.properties.car_forward,
            edge.properties.car_backward,
            edge.properties.bike_forward,
            edge.properties.bike_backward,
            edge.properties.train,
            edge.as_wkt(),
        ))?;
    }

    let nodes_path = std::path::Path::new(nodes_file);
    let mut nodes_csv = csv::Writer::from_path(nodes_path)?;
    nodes_csv.serialize(vec!["id", "lon", "lat"])?;
    for node in nodes {
        nodes_csv.serialize((node.id.0, node.coord.x, node.coord.y))?;
    }

    Ok(())
}

// pub fn pg(nodes: Vec<Node>, edges: Vec<Edge>) {}
