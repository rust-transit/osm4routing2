use crate::categorize::*;
use crate::models::*;
use osmpbfreader::objects::{NodeId, WayId};
use std::collections::{HashMap, HashSet};

// Way as represented in OpenStreetMap
struct Way {
    id: WayId,
    nodes: Vec<NodeId>,
    properties: EdgeProperties,
}

struct Reader {
    nodes: HashMap<NodeId, Node>,
    ways: Vec<Way>,
    nodes_to_keep: HashSet<NodeId>,
}

impl Reader {
    fn new() -> Reader {
        Reader {
            nodes: HashMap::new(),
            ways: Vec::new(),
            nodes_to_keep: HashSet::new(),
        }
    }

    fn count_nodes_uses(&mut self) {
        for way in &self.ways {
            for (i, node_id) in way.nodes.iter().enumerate() {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    // Count double extremities nodes
                    if i == 0 || i == way.nodes.len() - 1 {
                        node.uses += 2;
                    } else {
                        node.uses += 1;
                    }
                } else {
                    panic!("Missing node, id: {:?}", node_id)
                }
            }
        }
    }

    fn split_way(&self, way: &Way) -> Vec<Edge> {
        let mut result = Vec::new();

        let mut source = NodeId(0);
        let mut geometry = Vec::new();
        for (i, &node_id) in way.nodes.iter().enumerate() {
            let node = self.nodes[&node_id];
            if i == 0 {
                source = node_id;
                geometry.push(node.coord);
            } else {
                geometry.push(node.coord);

                if node.uses > 1 {
                    result.push(Edge {
                        id: format!("{}-{}", way.id.0, result.len()),
                        osm_id: way.id,
                        source,
                        target: node_id,
                        geometry,
                        properties: way.properties,
                    });

                    source = node_id;
                    geometry = vec![node.coord];
                }
            }
        }
        result
    }

    fn read_ways(&mut self, file: std::fs::File) {
        let mut pbf = osmpbfreader::OsmPbfReader::new(file);
        for obj in pbf.iter().flatten() {
            if let osmpbfreader::OsmObj::Way(way) = obj {
                let mut properties = EdgeProperties::default();
                for (key, val) in way.tags.iter() {
                    properties.update(key.to_string(), val.to_string());
                }
                properties.normalize();
                if properties.accessible() {
                    for node in &way.nodes {
                        self.nodes_to_keep.insert(*node);
                    }
                    self.ways.push(Way {
                        id: way.id,
                        nodes: way.nodes,
                        properties,
                    });
                }
            }
        }
    }

    fn read_nodes(&mut self, file: std::fs::File) {
        let mut pbf = osmpbfreader::OsmPbfReader::new(file);
        self.nodes.reserve(self.nodes_to_keep.len());
        for obj in pbf.iter().flatten() {
            if let osmpbfreader::OsmObj::Node(node) = obj {
                if self.nodes_to_keep.contains(&node.id) {
                    self.nodes_to_keep.remove(&node.id);
                    self.nodes.insert(
                        node.id,
                        Node {
                            id: node.id,
                            coord: Coord {
                                lon: node.lon(),
                                lat: node.lat(),
                            },
                            uses: 0,
                        },
                    );
                }
            }
        }
    }

    fn nodes(self) -> Vec<Node> {
        self.nodes
            .into_values()
            .filter(|node| node.uses > 1)
            .collect()
    }

    fn edges(&self) -> Vec<Edge> {
        self.ways
            .iter()
            .flat_map(|way| self.split_way(way))
            .collect()
    }
}

// Read all the nodes and ways of the osm.pbf file
pub fn read(filename: &str) -> Result<(Vec<Node>, Vec<Edge>), String> {
    let mut r = Reader::new();
    let path = std::path::Path::new(filename);
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    r.read_ways(file);
    let file_nodes = std::fs::File::open(path).map_err(|e| e.to_string())?;
    r.read_nodes(file_nodes);
    r.count_nodes_uses();
    let edges = r.edges();
    Ok((r.nodes(), edges))
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
        id: WayId(0),
        nodes: vec![NodeId(1), NodeId(2), NodeId(3)],
        properties: EdgeProperties::default(),
    }];
    let mut nodes = HashMap::new();
    nodes.insert(NodeId(1), Node::default());
    nodes.insert(NodeId(2), Node::default());
    nodes.insert(NodeId(3), Node::default());
    let mut r = Reader {
        ways,
        nodes,
        nodes_to_keep: HashSet::new(),
    };
    r.count_nodes_uses();
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
            id: WayId(0),
            nodes: vec![NodeId(1), NodeId(2), NodeId(3)],
            properties: EdgeProperties::default(),
        },
        Way {
            id: WayId(0),
            nodes: vec![NodeId(4), NodeId(5), NodeId(2)],
            properties: EdgeProperties::default(),
        },
    ];
    let mut r = Reader {
        nodes,
        ways,
        nodes_to_keep: HashSet::new(),
    };
    r.count_nodes_uses();
    let edges = r.edges();
    assert_eq!(3, edges.len());
}

#[test]
fn test_wrong_file() {
    let r = read("i hope you have no file name like this one");
    assert!(r.is_err());
}
