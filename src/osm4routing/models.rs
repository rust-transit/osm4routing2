//! Data models for nodes and edges in a routing graph.
//!
//! This module defines the core data structures used to represent
//! the routing network extracted from OpenStreetMap data.

use ahash::HashMap;
use std::hash::{Hash, Hasher};

use super::categorize::EdgeProperties;
pub use osmpbfreader::objects::{NodeId, WayId};

/// Coordinate type alias for WGS84 coordinates in decimal degrees.
///
/// Uses x for longitude and y for latitude.
type Coord = geo_types::Coord<f64>;

/// Trait for calculating distances between coordinates.
pub trait Distance {
    /// Calculate the great-circle distance to another coordinate in meters.
    ///
    /// Uses the haversine formula with Earth's mean radius of 6,378,100 meters.
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

/// A node in the routing graph, representing an OpenStreetMap node.
///
/// Nodes are the vertices of the routing graph where edges connect.
/// The `uses` field tracks how many ways reference this node, which
/// is used to identify intersection points when splitting ways into edges.
#[derive(Copy, Clone, Debug)]
pub struct Node {
    /// The OpenStreetMap node ID.
    pub id: NodeId,
    /// The geographical coordinates (longitude, latitude) in WGS84.
    pub coord: Coord,
    /// Count of how many times this node is referenced by ways.
    ///
    /// Endpoints of ways are counted twice to ensure dead-ends are preserved.
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

/// An edge in the routing graph, representing a traversable segment between two nodes.
///
/// Edges are created by splitting OpenStreetMap ways at intersection points.
/// Each edge has a unique ID (different from the OSM way ID, which can be shared
/// by multiple edges if a way is split).
#[derive(Clone, Debug)]
pub struct Edge {
    /// Unique identifier for this edge, typically "{way_id}-{index}".
    pub id: String,
    /// The original OpenStreetMap way ID.
    pub osm_id: WayId,
    /// The starting node of this edge.
    pub source: NodeId,
    /// The ending node of this edge.
    pub target: NodeId,
    /// The geometry of this edge as a sequence of coordinates.
    pub geometry: Vec<Coord>,
    /// Accessibility properties for different transportation modes.
    pub properties: EdgeProperties,
    /// The sequence of node IDs along this edge (including source and target).
    pub nodes: Vec<NodeId>,
    /// Additional OSM tags requested by the user.
    pub tags: HashMap<String, String>,
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
    /// Returns the geometry as a Well-Known Text (WKT) LINESTRING.
    ///
    /// Coordinates are formatted with 7 decimal places of precision.
    ///
    /// # Example
    /// ```
    /// use osm4routing::Edge;
    /// use geo_types::Coord;
    ///
    /// let edge = Edge {
    ///     geometry: vec![
    ///         Coord { x: 0.0, y: 0.0 },
    ///         Coord { x: 1.0, y: 1.0 },
    ///     ],
    ///     ..Default::default()
    /// };
    /// assert!(edge.as_wkt().contains("LINESTRING"));
    /// ```
    pub fn as_wkt(&self) -> String {
        let coords: Vec<String> = self
            .geometry
            .iter()
            .map(|coord| format!("{:.7} {:.7}", coord.x, coord.y))
            .collect();

        format!("LINESTRING({})", coords.as_slice().join(", "))
    }

    /// Calculate the total length of the edge in meters.
    ///
    /// Sums the great-circle distances between consecutive coordinates.
    pub fn length(&self) -> f64 {
        self.geometry
            .windows(2)
            .map(|coords| coords[0].distance_to(coords[1]))
            .sum()
    }

    /// Calculate the length from the start of the edge to a specific node.
    ///
    /// Returns 0.0 if the node is not found on this edge or is the first node.
    ///
    /// # Arguments
    /// * `node` - The node ID to measure to.
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

    /// Returns a new edge with reversed direction.
    ///
    /// The source and target are swapped, and the geometry and node sequence
    /// are reversed.
    pub fn reverse(mut self) -> Self {
        self.nodes.reverse();
        self.geometry.reverse();
        std::mem::swap(&mut self.target, &mut self.source);
        self
    }

    /// Merges two edges together, assuming this edge's target equals other's source.
    ///
    /// # Panics
    /// Panics if `self.target != other.source`.
    fn unsafe_merge(mut self, other: Self) -> Self {
        assert!(self.target == other.source);
        self.id = format!("{}-{}", self.id, other.id);
        self.target = other.target;
        self.nodes = [&self.nodes, &other.nodes[1..]].concat();
        self.geometry = [&self.geometry, &other.geometry[1..]].concat();
        self
    }

    /// Creates a new edge by stitching together two edges at a common node.
    ///
    /// Automatically handles reversing edges as needed to ensure proper connectivity.
    ///
    /// # Arguments
    /// * `edge1` - The first edge to merge.
    /// * `edge2` - The second edge to merge.
    /// * `node` - The common node where the edges meet.
    ///
    /// # Panics
    /// Panics if `node` is not an endpoint of both edges.
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
