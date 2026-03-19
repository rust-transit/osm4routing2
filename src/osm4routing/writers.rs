//! Output writers for routing graph data.
//!
//! This module provides functions to export the routing graph to various formats.

use super::error::Error;
use super::models::*;

/// Writes nodes and edges to CSV files.
///
/// Creates two CSV files: one for nodes and one for edges.
///
/// # Node CSV Format
/// Columns: `id`, `lon`, `lat`
/// - `id`: The OSM node ID
/// - `lon`: Longitude in decimal degrees (WGS84)
/// - `lat`: Latitude in decimal degrees (WGS84)
///
/// # Edge CSV Format
/// Columns: `id`, `osm_id`, `source`, `target`, `length`, `foot`, `car_forward`,
/// `car_backward`, `bike_forward`, `bike_backward`, `train`, `wkt`
/// - `id`: Unique edge identifier (format: "{way_id}-{index}")
/// - `osm_id`: The original OSM way ID
/// - `source`: ID of the starting node
/// - `target`: ID of the ending node
/// - `length`: Length in meters
/// - `foot`: Pedestrian accessibility (`Allowed`/`Forbidden`)
/// - `car_forward`: Car accessibility in forward direction
/// - `car_backward`: Car accessibility in backward direction
/// - `bike_forward`: Bike accessibility in forward direction
/// - `bike_backward`: Bike accessibility in backward direction
/// - `train`: Train accessibility
/// - `wkt`: Geometry as WKT LINESTRING
///
/// # Arguments
/// * `nodes` - List of nodes to write.
/// * `edges` - List of edges to write.
/// * `nodes_file` - Path for the nodes CSV file.
/// * `edges_file` - Path for the edges CSV file.
///
/// # Errors
/// Returns an error if file creation or CSV serialization fails.
///
/// # Example
///
/// ```no_run
/// use osm4routing;
///
/// let (nodes, edges) = osm4routing::read("map.osm.pbf").unwrap();
/// osm4routing::writers::csv(nodes, edges, "nodes.csv", "edges.csv").unwrap();
/// ```
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
