use std::fs::File;
use std::io::{BufWriter, Write};

use serde_json::{json, Value};

use super::models::*;

pub fn csv(nodes: Vec<Node>, edges: Vec<Edge>, output_path: &str) {
    let edges_path = std::path::Path::new(output_path).join("edges.csv");
    let mut edges_csv = csv::Writer::from_path(edges_path).unwrap();
    edges_csv
        .serialize(vec![
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
        ])
        .expect("CSV: unable to write edge header");
    for edge in edges {
        edges_csv
            .serialize((
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
            ))
            .expect("CSV: unable to write edge");
    }

    let nodes_path = std::path::Path::new(output_path).join("nodes.csv");
    let mut nodes_csv = csv::Writer::from_path(nodes_path).unwrap();
    nodes_csv
        .serialize(vec!["id", "lon", "lat"])
        .expect("CSV: unable to write node header");
    for node in nodes {
        nodes_csv
            .serialize((node.id.0, node.coord.lon, node.coord.lat))
            .expect("CSV: unable to write node");
    }
}

pub fn geojson(_: Vec<Node>, edges: Vec<Edge>, output_path: &str) {
    let features: Vec<Value> = edges
        .iter()
        .map(|edge| {
            let properties = json!({
                "id": edge.id,
                "osm_id": edge.osm_id.0,
                "source": edge.source.0,
                "target": edge.target.0,
                "length": edge.length(),
                "foot": edge.properties.foot,
                "car_forward": edge.properties.car_forward,
                "car_backward": edge.properties.car_backward,
                "bike_forward": edge.properties.bike_forward,
                "bike_backward": edge.properties.bike_backward,
                "train": edge.properties.train,
            });

            json!({
                "type": "Feature",
                "geometry": {
                    "type": "LineString",
                    "coordinates": edge.coordinates(),
                },
                "properties": properties,
            })
        })
        .collect();

    let feature_collection = json!({
        "type": "FeatureCollection",
        "features": features,
    });


    let geojson_path = std::path::Path::new(output_path).join("data.geojson");
    let file = File::create(&geojson_path)
        .expect("Unable to create file");
    (|| -> Result<(), std::io::Error> {
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &feature_collection)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    })().expect("Unable to write file");
}
