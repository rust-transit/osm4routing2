use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::categorize::EdgeProperties;
pub use osmpbfreader::objects::{NodeId, WayId};

// Coord are coordinates in decimal degress WGS84
type Coord = geo_types::Coord<f64>;

pub trait Distance {
    fn distance_to(&self, end: Coord) -> f64;
}

impl Distance for Coord {
    fn distance_to(&self, end: Coord) -> f64 {
        let r: f64 = 6_378_100.0;

        let d_lon: f64 = (end.x - self.x).to_radians();
        let d_lat: f64 = (end.y - self.y).to_radians();
        let lat1: f64 = (self.y).to_radians();
        let lat2: f64 = (end.y).to_radians();

        let a: f64 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
            + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
        let c: f64 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));

        r * c
    }
}

// Node is the OpenStreetMap node
#[derive(Copy, Clone, Debug)]
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
#[derive(Clone, Debug)]
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

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Edge {}

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
            .map(|coord| format!("{:.7} {:.7}", coord.x, coord.y))
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

    // Changes the direction of the geometry, source and target
    // Returns a new Edge
    pub fn reverse(mut self) -> Self {
        self.nodes.reverse();
        self.geometry.reverse();
        std::mem::swap(&mut self.target, &mut self.source);
        self
    }

    // Merges two edges together. It supposes that self.target == e2.source and will panic otherwise
    fn unsafe_merge(mut self, other: Self) -> Self {
        assert!(self.target == other.source);
        self.id = format!("{}-{}", self.id, other.id);
        self.target = other.target;
        self.nodes = [&self.nodes, &other.nodes[1..]].concat();
        self.geometry = [&self.geometry, &other.geometry[1..]].concat();
        self
    }

    // Creates a new edges by stiching together two edges at node `node`
    // Will panic if the node is not a common extremity for both
    pub fn merge(edge1: &Self, edge2: &Self, node: NodeId) -> Self {
        let edge1 = edge1.clone();
        let edge2 = edge2.clone();
        assert!(edge1.source == node || edge1.target == node);
        assert!(edge2.source == node || edge2.target == node);
        match (edge1.target == node, edge2.source == node) {
            (true, true) => edge1.unsafe_merge(edge2),
            (false, true) => edge1.reverse().unsafe_merge(edge2),
            (true, false) => edge1.unsafe_merge(edge2.reverse()),
            (false, false) => edge1.reverse().unsafe_merge(edge2.reverse()),
        }
    }
}

#[test]
fn test_as_wkt() {
    let edge = Edge {
        geometry: vec![
            Coord { x: 0., y: 0. },
            Coord { x: 1., y: 0. },
            Coord { x: 0., y: 1. },
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
    let a = Coord { x: 0., y: 0. };
    let b = Coord { x: 1., y: 0. };

    assert!(1. - (a.distance_to(b) / (1853. * 60.)).abs() < 0.01);
}

#[test]
fn test_length_until() {
    let e = Edge {
        target: NodeId(2),
        nodes: vec![NodeId(0), NodeId(1), NodeId(2)],
        geometry: vec![
            Coord { x: 0., y: 0. },
            Coord { x: 1., y: 0. },
            Coord { x: 1., y: 1. },
        ],
        ..Default::default()
    };

    assert!((1. - e.length_until(&NodeId(1)) / (1853. * 60.)).abs() < 0.01);
    assert_eq!(e.length_until(&NodeId(0)), 0.);
    assert!((1. - e.length_until(&NodeId(2)) / (2. * 1853. * 60.)).abs() < 0.01);
}
