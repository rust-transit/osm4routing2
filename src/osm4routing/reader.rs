//! PBF file reading and graph construction.
//!
//! This module provides functionality to read OpenStreetMap PBF files
//! and convert them into a routing graph structure (nodes and edges).

use super::categorize::*;
use super::error::Error;
use super::models::*;
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use osmpbfreader::objects::{NodeId, WayId};
use std::path::Path;

/// Internal representation of an OpenStreetMap way during processing.
///
/// Stores the node references and computed properties before conversion to edges.
struct Way {
    /// The OSM way ID.
    id: WayId,
    /// Ordered list of node IDs that make up this way.
    nodes: Vec<NodeId>,
    /// Computed accessibility properties from OSM tags.
    properties: EdgeProperties,
    /// Tags requested to be preserved (via `read_tag`).
    tags: HashMap<String, String>,
}

impl Default for Way {
    fn default() -> Self {
        Self {
            id: WayId(0),
            nodes: vec![],
            properties: EdgeProperties::default(),
            tags: HashMap::default(),
        }
    }
}

/// Configurable reader for extracting routing graphs from PBF files.
///
/// Uses the builder pattern to allow filtering and customization of the
/// extraction process.
///
/// # Example
///
/// ```no_run
/// use osm4routing::Reader;
///
/// let (nodes, edges) = Reader::new()
///     .reject("highway", "footway")
///     .read("data.osm.pbf")
///     .unwrap();
/// ```
#[derive(Default)]
pub struct Reader {
    /// Map of node IDs to their full data, populated during `read_nodes`.
    nodes: HashMap<NodeId, Node>,
    /// List of ways to be converted to edges.
    ways: Vec<Way>,
    /// Set of node IDs referenced by ways (used to filter which nodes to load).
    nodes_to_keep: HashSet<NodeId>,
    /// Tags that should cause ways to be excluded. Use "*" to match any value.
    forbidden_tags: HashMap<String, HashSet<String>>,
    /// Tags that must be present for ways to be included. Use "*" to match any value.
    required_tags: HashMap<String, HashSet<String>>,
    /// Additional OSM tags to preserve in edge output.
    tags_to_read: HashSet<String>,
    /// Whether to merge consecutive edges from different ways at non-intersections.
    should_merge_ways: bool,
}

impl Reader {
    /// Creates a new reader with default configuration.
    pub fn new() -> Reader {
        Reader::default()
    }

    /// Adds a tag filter to reject ways with a specific key-value pair.
    ///
    /// Use "*" as the value to reject any way with the given key, regardless of value.
    /// Can be chained to add multiple reject filters.
    ///
    /// # Example
    ///
    /// ```
    /// use osm4routing::Reader;
    ///
    /// let reader = Reader::new()
    ///     .reject("area", "yes")
    ///     .reject("access", "private");
    /// ```
    pub fn reject(mut self, key: &str, value: &str) -> Self {
        self.forbidden_tags
            .entry(key.to_string())
            .or_default()
            .insert(value.to_string());
        self
    }

    /// Requires ways to have a specific tag key-value pair.
    ///
    /// Use "*" as the value to accept any value for the given key.
    /// Multiple requirements can be added, and ways matching ANY requirement are included.
    ///
    /// # Example
    ///
    /// ```
    /// use osm4routing::Reader;
    ///
    /// // Only include railways
    /// let reader = Reader::new()
    ///     .require("railway", "rail");
    ///
    /// // Include primary or secondary roads
    /// let reader = Reader::new()
    ///     .require("highway", "primary")
    ///     .require("highway", "secondary");
    /// ```
    pub fn require(mut self, key: &str, value: &str) -> Self {
        self.required_tags
            .entry(key.to_string())
            .or_default()
            .insert(value.to_string());
        self
    }

    /// Requests that a specific OSM tag be preserved in the edge output.
    ///
    /// By default, only computed accessibility properties are stored.
    /// Use this to keep additional tag values for later analysis.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use osm4routing::Reader;
    ///
    /// let (nodes, edges) = Reader::new()
    ///     .read_tag("name")
    ///     .read_tag("maxspeed")
    ///     .read("data.osm.pbf")
    ///     .unwrap();
    ///
    /// // Access the preserved tags
    /// if let Some(name) = edges[0].tags.get("name") {
    ///     println!("Road name: {}", name);
    /// }
    /// ```
    pub fn read_tag(mut self, key: &str) -> Self {
        self.tags_to_read.insert(key.to_string());
        self
    }

    /// Enables merging of consecutive edges from different OSM ways.
    ///
    /// When enabled, edges that meet at a node with no other connections
    /// (degree 2) and have the same properties and tags will be merged
    /// into a single edge. This is useful when tags change mid-way (e.g.,
    /// a tunnel) but the road is topologically continuous.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use osm4routing::Reader;
    ///
    /// let (nodes, edges) = Reader::new()
    ///     .merge_ways()
    ///     .read("data.osm.pbf")
    ///     .unwrap();
    /// ```
    pub fn merge_ways(mut self) -> Self {
        self.should_merge_ways = true;
        self
    }

    /// Counts how many times each node is referenced by ways.
    ///
    /// Endpoint nodes are counted twice to ensure dead-end roads are
    /// preserved. Nodes with uses > 1 become intersection points where
    /// ways are split into edges.
    ///
    /// Returns an error if a way references a node not present in `nodes`.
    fn count_nodes_uses(&mut self) -> Result<(), Error> {
        for way in &self.ways {
            for (i, node_id) in way.nodes.iter().enumerate() {
                let node = self
                    .nodes
                    .get_mut(node_id)
                    .ok_or(Error::MissingNode(*node_id))?;
                // Count double extremities nodes to be sure to include dead-ends
                if i == 0 || i == way.nodes.len() - 1 {
                    node.uses += 2;
                } else {
                    node.uses += 1;
                }
            }
        }
        Ok(())
    }

    /// Splits an OSM way into multiple edges at intersection points.
    ///
    /// A way is split at every node where `uses > 1` (intersection).
    /// Creates edges with unique IDs in the format "{way_id}-{index}".
    ///
    /// # Arguments
    /// * `way` - The way to split.
    fn split_way(&self, way: &Way) -> Vec<Edge> {
        let mut result = Vec::new();

        let mut source = NodeId(0);
        let mut geometry = Vec::new();
        let mut nodes = Vec::new();
        for (i, &node_id) in way.nodes.iter().enumerate() {
            let node = self.nodes[&node_id];
            geometry.push(node.coord);
            nodes.push(node.id);
            if i == 0 {
                source = node_id;
            } else if node.uses > 1 {
                result.push(Edge {
                    id: format!("{}-{}", way.id.0, result.len()),
                    osm_id: way.id,
                    source,
                    target: node_id,
                    geometry,
                    properties: way.properties,
                    nodes,
                    tags: way.tags.clone(),
                });

                source = node_id;
                geometry = vec![node.coord];
                nodes = vec![node.id]
            }
        }
        result
    }

    /// Recursively merges consecutive edges at degree-2 nodes.
    ///
    /// OSM ways can be split by tag changes (e.g., bridge=yes) even when
    /// there is no topological intersection. This function merges such
    /// edges to simplify the routing graph.
    ///
    /// Two edges are merged when:
    /// - They meet at a node with exactly 2 edge connections (degree 2)
    /// - They have identical accessibility properties
    /// - They have identical tags (if tags_to_read is used)
    /// - They haven't been merged in a previous iteration
    ///
    /// # Arguments
    /// * `edges` - The edges to potentially merge.
    fn do_merge_edges(&mut self, edges: Vec<Edge>) -> Vec<Edge> {
        let initial_edges_count = edges.len();

        // We build an adjacency map for every node that might have exactly two edges
        let mut neighbors: HashMap<NodeId, Vec<_>> = HashMap::new();
        for edge in edges.iter() {
            // Extremities of a way in `count_nodes_uses` are counted twice to avoid pruning deadends.
            // We want to look at nodes with at two extremities, hence 4 uses
            if !self.nodes.contains_key(&edge.source) {
                println!("Problem with node {}, edge {}", &edge.source.0, edge.id);
            }
            if self.nodes.get(&edge.source).unwrap().uses == 4 {
                neighbors.entry(edge.source).or_default().push(edge);
            }
            if self.nodes.get(&edge.target).unwrap().uses == 4 {
                neighbors.entry(edge.target).or_default().push(edge);
            }
        }

        let mut result = Vec::new();
        let mut already_merged = HashSet::new();
        for (node, edges) in neighbors.drain() {
            // We merge two edges at the node if there are only two edges
            // The edges must have the same accessibility properties
            // If the consummer asked to store the tags, the tags must be the same for both edges
            // The edges must be from different ways (no surface)
            // The edges must not have been merged this iteration (they might be re-merged through a recurive call)
            if edges.len() == 2
                && edges[0].properties == edges[1].properties
                && edges[0].tags == edges[1].tags
                && edges[0].id != edges[1].id
                && !already_merged.contains(&edges[0].id)
                && !already_merged.contains(&edges[1].id)
            {
                let edge1 = edges[0];
                let edge2 = edges[1];
                result.push(Edge::merge(edge1, edge2, node));
                already_merged.insert(edge1.id.clone());
                already_merged.insert(edge2.id.clone());
                self.nodes.remove(&node);
            }
        }

        for edge in edges {
            if !already_merged.contains(&edge.id) {
                result.push(edge);
            }
        }

        // If we reduced the number of edges, that means that we merged edges
        // They might need to be merged again, recursively
        if initial_edges_count > result.len() {
            self.do_merge_edges(result)
        } else {
            result
        }
    }

    /// Checks if a way should be rejected based on user-specified filters.
    ///
    /// Returns true if the way should be excluded because:
    /// - It has a forbidden tag (via `reject`)
    /// - It doesn't have any required tag (via `require`)
    fn is_user_rejected(&self, way: &osmpbfreader::Way) -> bool {
        let meet_required_tags = self.required_tags.is_empty()
            || way.tags.iter().any(|(key, val)| {
                self.required_tags
                    .get(key.as_str())
                    .map(|values| values.contains(val.as_str()) || values.contains("*"))
                    == Some(true)
            });

        let has_forbidden_tags = way.tags.iter().any(|(key, val)| {
            self.forbidden_tags
                .get(key.as_str())
                .map(|vals| vals.contains(val.as_str()) || vals.contains("*"))
                == Some(true)
        });

        !meet_required_tags || has_forbidden_tags
    }

    /// Reads all ways from the PBF file and populates `ways` and `nodes_to_keep`.
    ///
    /// Processes each way in the file:
    /// 1. Computes accessibility properties from OSM tags
    /// 2. Filters by accessibility and user-specified rules
    /// 3. Stores way data and marks referenced nodes for loading
    ///
    /// # Arguments
    /// * `file` - Open file handle to the PBF file.
    fn read_ways(&mut self, file: std::fs::File) {
        let mut pbf = osmpbfreader::OsmPbfReader::new(file);
        for obj in pbf.par_iter().flatten() {
            if let osmpbfreader::OsmObj::Way(way) = obj {
                let mut properties = EdgeProperties::default();
                let mut tags = HashMap::new();
                for (key, val) in way.tags.iter() {
                    properties.update(key.to_string(), val.to_string());
                    if self.tags_to_read.contains(key.as_str()) {
                        tags.insert(key.to_string(), val.to_string());
                    }
                }
                properties.normalize();
                if properties.accessible() && !self.is_user_rejected(&way) {
                    for node in &way.nodes {
                        self.nodes_to_keep.insert(*node);
                    }
                    self.ways.push(Way {
                        id: way.id,
                        nodes: way.nodes,
                        properties,
                        tags,
                    });
                }
            }
        }
    }

    /// Reads all nodes from the PBF file that are referenced by ways.
    ///
    /// Only loads nodes that are in `nodes_to_keep` (populated by `read_ways`).
    /// Removes loaded nodes from `nodes_to_keep` as they are found.
    ///
    /// # Arguments
    /// * `file` - Open file handle to the PBF file.
    fn read_nodes(&mut self, file: std::fs::File) {
        let mut pbf = osmpbfreader::OsmPbfReader::new(file);
        self.nodes.reserve(self.nodes_to_keep.len());
        for obj in pbf.par_iter().flatten() {
            if let osmpbfreader::OsmObj::Node(node) = obj
                && self.nodes_to_keep.contains(&node.id)
            {
                self.nodes_to_keep.remove(&node.id);
                self.nodes.insert(
                    node.id,
                    Node {
                        id: node.id,
                        coord: geo_types::Coord {
                            x: node.lon(),
                            y: node.lat(),
                        },
                        uses: 0,
                    },
                );
            }
        }
    }

    /// Returns all nodes that are part of the routing graph.
    ///
    /// Filters out nodes that are not used by any edge (uses <= 1).
    fn nodes(&self) -> Vec<Node> {
        self.nodes
            .values()
            .filter(|node| node.uses > 1)
            .copied()
            .collect()
    }

    /// Converts all ways to edges by splitting at intersections.
    fn edges(&self) -> Vec<Edge> {
        self.ways
            .iter()
            .flat_map(|way| self.split_way(way))
            .collect()
    }

    /// Reads the PBF file and constructs the routing graph.
    ///
    /// This is the main entry point for extracting routing data.
    /// The file is read twice: once for ways, then for nodes.
    ///
    /// # Arguments
    /// * `filename` - Path to the OSM PBF file.
    ///
    /// # Returns
    /// A tuple of (nodes, edges) representing the routing graph.
    ///
    /// # Errors
    /// Returns an error if:
    /// - The file cannot be opened
    /// - A way references a node not present in the file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use osm4routing::Reader;
    ///
    /// let (nodes, edges) = Reader::new()
    ///     .read("map.osm.pbf")
    ///     .expect("Failed to read PBF file");
    ///
    /// println!("Loaded {} nodes and {} edges", nodes.len(), edges.len());
    /// ```
    pub fn read<P: AsRef<Path>>(&mut self, filename: P) -> Result<(Vec<Node>, Vec<Edge>), Error> {
        let file = std::fs::File::open(filename.as_ref())?;
        self.read_ways(file);
        let file_nodes = std::fs::File::open(filename.as_ref())?;
        self.read_nodes(file_nodes);
        self.count_nodes_uses()?;

        let edges = if self.should_merge_ways {
            self.do_merge_edges(self.edges())
        } else {
            self.edges()
        };
        Ok((self.nodes(), edges))
    }
}

/// Convenience function to read a PBF file with default settings.
///
/// This is equivalent to:
/// ```ignore
/// osm4routing::Reader::new().read(filename)
/// ```
///
/// For more control over the extraction process, use [`Reader`] directly.
///
/// # Arguments
/// * `filename` - Path to the OSM PBF file.
///
/// # Returns
/// A tuple of (nodes, edges) representing the routing graph.
pub fn read<P: AsRef<Path>>(filename: P) -> Result<(Vec<Node>, Vec<Edge>), Error> {
    Reader::new().read(filename)
}

#[test]
fn test_real_all() {
    let (nodes, ways) = read("src/osm4routing/test_data/minimal.osm.pbf").unwrap();
    assert_eq!(2, nodes.len());
    assert_eq!(1, ways.len());
}

#[test]
fn test_count_nodes() {
    let ways = vec![Way {
        nodes: vec![NodeId(1), NodeId(2), NodeId(3)],
        ..Default::default()
    }];
    let mut nodes = HashMap::new();
    nodes.insert(NodeId(1), Node::default());
    nodes.insert(NodeId(2), Node::default());
    nodes.insert(NodeId(3), Node::default());
    let mut r = Reader {
        ways,
        nodes,
        ..Default::default()
    };
    r.count_nodes_uses().unwrap();
    assert_eq!(2, r.nodes[&NodeId(1)].uses);
    assert_eq!(1, r.nodes[&NodeId(2)].uses);
    assert_eq!(2, r.nodes[&NodeId(3)].uses);

    assert_eq!(2, r.nodes().len());
}

#[test]
fn test_split() {
    let mut nodes = HashMap::new();
    nodes.insert(NodeId(1), Node::default());
    nodes.insert(NodeId(2), Node::default());
    nodes.insert(NodeId(3), Node::default());
    nodes.insert(NodeId(4), Node::default());
    nodes.insert(NodeId(5), Node::default());
    let ways = vec![
        Way {
            nodes: vec![NodeId(1), NodeId(2), NodeId(3)],
            ..Default::default()
        },
        Way {
            nodes: vec![NodeId(4), NodeId(5), NodeId(2)],
            ..Default::default()
        },
    ];
    let mut r = Reader {
        nodes,
        ways,
        ..Default::default()
    };
    r.count_nodes_uses().unwrap();
    let edges = r.edges();
    assert_eq!(3, edges.len());
}

#[test]
fn test_wrong_file() {
    let r = read("i hope you have no file name like this one");
    assert!(r.is_err());
}

#[test]
fn forbidden_values() {
    let (_, ways) = Reader::new()
        .reject("highway", "secondary")
        .read(&"src/osm4routing/test_data/minimal.osm.pbf")
        .unwrap();
    assert_eq!(0, ways.len());
}

#[test]
fn forbidden_wildcard() {
    let (_, ways) = Reader::new()
        .reject("highway", "*")
        .read(&"src/osm4routing/test_data/minimal.osm.pbf")
        .unwrap();
    assert_eq!(0, ways.len());
}

#[test]
fn way_of_node() {
    let mut r = Reader::new();
    let (_nodes, edges) = r
        .read(&"src/osm4routing/test_data/minimal.osm.pbf")
        .unwrap();

    assert_eq!(2, edges[0].nodes.len());
}

#[test]
fn read_tags() {
    let (_nodes, edges) = Reader::new()
        .read_tag("highway")
        .read(&"src/osm4routing/test_data/minimal.osm.pbf")
        .unwrap();

    assert_eq!("secondary", edges[0].tags.get("highway").unwrap());
}

#[test]
fn require_value_ok() {
    let (_, ways) = Reader::new()
        .require("highway", "secondary")
        .read(&"src/osm4routing/test_data/minimal.osm.pbf")
        .unwrap();
    assert_eq!(1, ways.len());
}

#[test]
fn require_value_missing() {
    let (_, ways) = Reader::new()
        .require("highway", "primary")
        .read(&"src/osm4routing/test_data/minimal.osm.pbf")
        .unwrap();
    assert_eq!(0, ways.len());
}

#[test]
fn require_wildcart() {
    let (_, ways) = Reader::new()
        .require("highway", "*")
        .read(&"src/osm4routing/test_data/minimal.osm.pbf")
        .unwrap();
    assert_eq!(1, ways.len());
}

#[test]
fn require_multiple_tags() {
    let (_, ways) = Reader::new()
        .require("highway", "primary")
        .require("highway", "secondary")
        .read(&"src/osm4routing/test_data/minimal.osm.pbf")
        .unwrap();
    assert_eq!(1, ways.len());
}

#[test]
fn merging_edges() {
    let (_nodes, edges) = Reader::new()
        .read(&"src/osm4routing/test_data/ways_to_merge.osm.pbf")
        .unwrap();
    assert_eq!(2, edges.len());

    let (_nodes, edges) = Reader::new()
        .merge_ways()
        .read(&"src/osm4routing/test_data/ways_to_merge.osm.pbf")
        .unwrap();
    assert_eq!(1, edges.len());
}
