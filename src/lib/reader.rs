extern crate osmpbfreader;
use std::collections::HashMap;
use lib::models::*;
use lib::categorize::*;
use std;

// Way as represented in OpenStreetMap
struct Way {
    id: i64,
    nodes: Vec<i64>,
    properties: EdgeProperties,
}

struct Reader {
    nodes: HashMap<i64, Node>,
    ways: Vec<Way>,
}

impl Reader {
    fn new() -> Reader {
        Reader {
            nodes: HashMap::new(),
            ways: Vec::new(),
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
                    panic!("Missing node, id: {}", node_id)
                }
            }
        }
    }


    fn split_way(&self, way: &Way) -> Vec<Edge> {
        let mut result = Vec::new();

        let mut source = 0;
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
                        id: way.id,
                        source: source,
                        target: node_id,
                        geometry: geometry,
                        properties: way.properties,
                    });

                    source = node_id;
                    geometry = vec![node.coord];
                }
            }
        }
        result
    }

    fn read(&mut self, filename: &str) {
        let path = std::path::Path::new(filename);
        let r = std::fs::File::open(&path).unwrap();
        let mut pbf = osmpbfreader::OsmPbfReader::new(r);
        for obj in pbf.iter() {
            match obj {
                osmpbfreader::OsmObj::Node(node) => {
                    self.nodes.insert(node.id,
                                      Node {
                                          id: node.id,
                                          coord: Coord {
                                              lon: node.lon,
                                              lat: node.lat,
                                          },
                                          uses: 0,
                                      });
                }
                osmpbfreader::OsmObj::Way(way) => {
                    let mut properties = EdgeProperties::new();
                    for (key, val) in way.tags {
                        properties.update(key, val);
                    }
                    properties.normalize();
                    if properties.accessible() {
                        self.ways.push(Way {
                            id: way.id,
                            nodes: way.nodes,
                            properties: properties,
                        })
                    }
                }
                osmpbfreader::OsmObj::Relation(_) => {}
            }
        }
    }


    fn nodes(&self) -> Vec<Node> {
        self.nodes
            .iter()
            .map(|(_, node)| node)
            .filter(|node| node.uses > 1)
            .map(|n| n.clone())
            .collect()
    }

    fn edges(&self) -> Vec<Edge> {
        self.ways.iter().flat_map(|way| self.split_way(way)).collect()
    }
}

// Read all the nodes and ways of the osm.pbf file
pub fn read(filename: &str) -> (Vec<Node>, Vec<Edge>) {
    let mut r = Reader::new();
    r.read(filename);
    r.count_nodes_uses();
    return (r.nodes(), r.edges());
}

#[test]
fn test_real_all() {
    let (nodes, ways) = read("src/lib/test_data/minimal.osm.pbf");
    assert_eq!(2, nodes.len());
    assert_eq!(1, ways.len());
}

#[test]
fn test_count_nodes() {
    let ways = vec![Way {
                        id: 0,
                        nodes: vec![1, 2, 3],
                    }];
    let mut nodes = HashMap::new();
    nodes.insert(1, Node::new());
    nodes.insert(2, Node::new());
    nodes.insert(3, Node::new());
    let mut r = Reader {
        ways: ways,
        nodes: nodes,
    };
    r.count_nodes_uses();
    assert_eq!(2, r.nodes[&1].uses);
    assert_eq!(1, r.nodes[&2].uses);
    assert_eq!(2, r.nodes[&3].uses);

    assert_eq!(2, r.nodes().len());
}

#[test]
fn test_split() {
    let mut nodes = HashMap::new();
    nodes.insert(1, Node::new());
    nodes.insert(2, Node::new());
    nodes.insert(3, Node::new());
    nodes.insert(4, Node::new());
    nodes.insert(5, Node::new());
    let ways = vec![Way {
                        id: 0,
                        nodes: vec![1, 2, 3],
                    },
                    Way {
                        id: 0,
                        nodes: vec![4, 5, 2],
                    }];
    let mut r = Reader {
        nodes: nodes,
        ways: ways,
    };
    r.count_nodes_uses();
    let edges = r.edges();
    assert_eq!(3, edges.len());
}

#[test]
#[should_panic]
fn test_wrong_file() {
    read("i hope you have no file name like this one");
}
