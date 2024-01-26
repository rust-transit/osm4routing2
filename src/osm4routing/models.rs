use std::collections::HashMap;

use super::categorize::EdgeProperties;
pub use osmpbfreader::objects::{NodeId, WayId};

// Coord are coordinates in decimal degress WGS84
#[derive(Copy, Clone, Default)]
pub struct Coord {
    pub lon: f64,
    pub lat: f64,
}

impl Coord {
    pub fn distance_to(&self, end: Coord) -> f64 {
        let r: f64 = 6_378_100.0;

        let d_lon: f64 = (end.lon - self.lon).to_radians();
        let d_lat: f64 = (end.lat - self.lat).to_radians();
        let lat1: f64 = (self.lat).to_radians();
        let lat2: f64 = (end.lat).to_radians();

        let a: f64 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
            + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
        let c: f64 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));

        r * c
    }
}

// Node is the OpenStreetMap node
#[derive(Copy, Clone)]
pub struct Node {
    pub id: NodeId,
    pub coord: Coord,
    pub uses: i16,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            id: NodeId(0),
            coord: Default::default(),
            uses: Default::default(),
        }
    }
}

// Edge is a topological representation with only two extremities and no geometry
pub struct Edge {
    pub id: String,
    pub osm_id: WayId,
    pub source: NodeId,
    pub target: NodeId,
    pub geometry: Vec<Coord>,
    pub properties: EdgeProperties,
    pub nodes: Vec<NodeId>,
    pub tags: std::collections::HashMap<String, String>,
}

impl Default for Edge {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            osm_id: WayId(0),
            source: NodeId(1),
            target: NodeId(1),
            geometry: vec![],
            properties: EdgeProperties::default(),
            nodes: vec![],
            tags: HashMap::default(),
        }
    }
}

impl Edge {
    // Geometry in the well known format
    pub fn as_wkt(&self) -> String {
        let coords: Vec<String> = self
            .geometry
            .iter()
            .map(|coord| format!("{:.7} {:.7}", coord.lon, coord.lat))
            .collect();

        format!("LINESTRING({})", coords.as_slice().join(", "))
    }

    // Length in meters of the edge
    pub fn length(&self) -> f64 {
        self.geometry
            .windows(2)
            .map(|coords| coords[0].distance_to(coords[1]))
            .sum()
    }

    // Length in meter until the given node
    pub fn length_until(&self, node: &NodeId) -> f64 {
        let mut length = 0.;
        for i in 1..(self.nodes.len()) {
            length += self.geometry[i - 1].distance_to(self.geometry[i]);
            if &self.nodes[i] == node {
                return length;
            }
        }
        0.
    }
}

#[test]
fn test_as_wkt() {
    let edge = Edge {
        geometry: vec![
            Coord { lon: 0., lat: 0. },
            Coord { lon: 1., lat: 0. },
            Coord { lon: 0., lat: 1. },
        ],
        ..Default::default()
    };
    assert!(
        "LINESTRING(0.0000000 0.0000000, 1.0000000 0.0000000, 0.0000000 1.0000000)"
            == edge.as_wkt()
    );
}

#[test]
fn test_distance() {
    let a = Coord { lon: 0., lat: 0. };
    let b = Coord { lon: 1., lat: 0. };

    assert!(1. - (a.distance_to(b) / (1853. * 60.)).abs() < 0.01);
}

#[test]
fn test_length_until() {
    let e = Edge {
        target: NodeId(2),
        nodes: vec![NodeId(0), NodeId(1), NodeId(2)],
        geometry: vec![
            Coord { lon: 0., lat: 0. },
            Coord { lon: 1., lat: 0. },
            Coord { lon: 1., lat: 1. },
        ],
        ..Default::default()
    };

    assert!((1. - e.length_until(&NodeId(1)) / (1853. * 60.)).abs() < 0.01);
    assert_eq!(e.length_until(&NodeId(0)), 0.);
    assert!((1. - e.length_until(&NodeId(2)) / (2. * 1853. * 60.)).abs() < 0.01);
}
