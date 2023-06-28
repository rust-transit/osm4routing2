use super::categorize::*;
use super::models::*;
use std::collections::{HashMap, HashSet};
use std::error::Error;



#[derive(Default)]
pub struct OsmReader {
    nodes: HashMap<NodeId, Node>,
    ways: Vec<Way>,
    nodes_to_keep: HashSet<NodeId>,
    forbidden: HashMap<String, HashSet<String>>,
}

impl OsmReader {
    pub fn new() -> OsmReader {
        OsmReader::default()
    }

    pub fn reject(mut self, key: &str, value: &str) -> Self {
        self.forbidden
            .entry(key.to_string())
            .or_default()
            .insert(value.to_string());
        self
    }

    fn count_nodes_uses(&mut self) {
        for way in &self.ways {
            for (i, node_id) in way.nodes.iter().enumerate() {
                let node = self
                    .nodes
                    .get_mut(node_id)
                    .expect("Missing node, id: {node_id}");
                // Count double extremities nodes
                if i == 0 || i == way.nodes.len() - 1 {
                    node.uses += 2;
                } else {
                    node.uses += 1;
                }
            }
        }
    }

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
                });

                source = node_id;
                geometry = vec![node.coord];
                nodes = vec![node.id]
            }
        }
        result
    }

    fn read_ways(&mut self, file: std::fs::File) {
        let mut pbf = osmpbfreader::OsmPbfReader::new(file);
        for obj in pbf.iter().flatten() {
            if let osmpbfreader::OsmObj::Way(way) = obj {
                let mut skip = false;
                let mut properties = EdgeProperties::default();
                for (key, val) in way.tags.iter() {
                    properties.update(key.to_string(), val.to_string());
                    if self
                        .forbidden
                        .get(key.as_str())
                        .map(|vals| vals.contains(val.as_str()) || vals.contains("*"))
                        == Some(true)
                    {
                        skip = true;
                    }
                }


                properties.normalize();
                if properties.accessible() && !skip {
                    for node in &way.nodes {
                        self.nodes_to_keep.insert(node.into());
                    }
                    self.ways.push(Way {
                        id: way.id.into(),
                        nodes: way.nodes.into(),
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
                let node_id: NodeId = node.id.into();
                if self.nodes_to_keep.contains(&node_id) {
                    self.nodes_to_keep.remove(&node_id);
                    self.nodes.insert(
                        node_id,
                        Node {
                            id: node_id,
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

    fn nodes(&self) -> Vec<Node> {
        self.nodes
            .values()
            .filter(|node| node.uses > 1)
            .copied()
            .collect()
    }

    fn edges(&self) -> Vec<Edge> {
        self.ways
            .iter()
            .flat_map(|way| self.split_way(way))
            .collect()
    }

    pub fn read(&mut self, filename: &str) -> Result<(Vec<Node>, Vec<Edge>), String> {
        let path = std::path::Path::new(filename);
        let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        self.read_ways(file);
        let file_nodes = std::fs::File::open(path).map_err(|e| e.to_string())?;
        self.read_nodes(file_nodes);
        self.count_nodes_uses();
        Ok((self.nodes(), self.edges()))
    }
}

// Read all the nodes and ways of the osm.pbf file
pub fn osm_read(filename: &str) -> Result<(Vec<Node>, Vec<Edge>), String> {
    OsmReader::new().read(filename)
}


#[derive(Default)]
pub struct CsvReader {
    nodes: HashMap<NodeId, Node>,
    ways: Vec<Way>,
    nodes_to_keep: HashSet<NodeId>,
    forbidden: HashMap<String, HashSet<String>>,
}

impl CsvReader {
    pub fn new() -> CsvReader {
        CsvReader::default()
    }

    pub fn reject(mut self, key: &str, value: &str) -> Self {
        self.forbidden
            .entry(key.to_string())
            .or_default()
            .insert(value.to_string());
        self
    }

    // fn count_nodes_uses(&mut self) {
    //     for way in &self.ways {
    //         for (i, node_id) in way.nodes.iter().enumerate() {
    //             let node = self
    //                 .nodes
    //                 .get_mut(node_id)
    //                 .expect("Missing node, id: {node_id}");
    //             // Count double extremities nodes
    //             if i == 0 || i == way.nodes.len() - 1 {
    //                 node.uses += 2;
    //             } else {
    //                 node.uses += 1;
    //             }
    //         }
    //     }
    // }

    fn count_nodes_uses(&mut self) {
        let mut ways_to_remove = Vec::new();
        let mut removed_ways_count = 0;
        let mut missing_node_stats = Vec::new();
    
        for (way_index, way) in self.ways.iter().enumerate() {
            let mut missing_node = false;
    
            for (i, node_id) in way.nodes.iter().enumerate() {
                if let Some(node) = self.nodes.get_mut(node_id) {
                    // Count double extremities nodes
                    if i == 0 || i == way.nodes.len() - 1 {
                        node.uses += 2;
                    } else {
                        node.uses += 1;
                    }
                } else {
                    missing_node = true;
                    missing_node_stats.push((way_index, *node_id));
                    break;
                }
            }
    
            if missing_node {
                ways_to_remove.push(way_index);
                removed_ways_count += 1;
            }
        }
    
        let removed_ways_percentage = (removed_ways_count as f32 / self.ways.len() as f32) * 100.0;

        // Remove ways with missing nodes
        for index in ways_to_remove.into_iter().rev() {
            self.ways.remove(index);
        }
    
        
    
        println!("Statistiques :");
        println!("Pourcentage de ways retirées : {:.2}%", removed_ways_percentage);
        println!("Ways retirées :");
        for (way_index, node_id) in missing_node_stats {
            println!("Way {} - NodeId {}", way_index, node_id);
        }
    }
    
    

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
                });

                source = node_id;
                geometry = vec![node.coord];
                nodes = vec![node.id]
            }
        }
        result
    }

    fn read_nodes(&mut self, file: std::fs::File) -> Result<(), Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in reader.records() {
            let record = result?;
            let node_id_v: i64 = record[0].parse()?;
            let node_id = NodeId(node_id_v);
            let lat: f64 = record[1].parse()?;
            let lon: f64 = record[2].parse()?;
            // let tags_str = record[3].trim();
            // let tags: HashMap<String, String> = serde_json::from_str(tags_str)?;
            // print tags
            // for (key, value) in &tags {
                // if let Some(values) = self.forbidden.get(key) {
                    // if values.contains(value) {
                        // continue;
                    // }
                // }
                // println!("{}: {}", key, value);
            // }

            self.nodes.insert(
                node_id,
                Node {
                    id: node_id,
                    coord: Coord { lon, lat },
                    uses: 0,
                },
            );
            self.nodes_to_keep.insert(node_id);
        }

        Ok(())
    }

    fn read_ways(&mut self, file: std::fs::File) -> Result<(), Box<dyn Error>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        for result in reader.records() {
            let record = result?;
            let way_id: WayId = WayId(record[0].parse::<i64>()?);
            let nodes_str = record[1].trim();
            let nodes: Vec<NodeId> = nodes_str
                .trim_start_matches('[')
                .trim_end_matches(']')
                .split(", ")
                .map(|id| NodeId(id.parse::<i64>().unwrap()))
                .collect();

            self.ways.push(Way {
                id: way_id,
                nodes: Nodes(nodes),
                properties: Default::default(), // todo
            });
        }

        Ok(())
    }

    fn nodes(&self) -> Vec<Node> {
        self.nodes
            .values()
            .filter(|node| node.uses > 1)
            .copied()
            .collect()
    }

    fn edges(&self) -> Vec<Edge> {
        self.ways
            .iter()
            .flat_map(|way| self.split_way(way))
            .collect()
    }


    pub fn read(&mut self, nodes_filename: &str, ways_filename: &str) -> Result<(Vec<Node>, Vec<Edge>), String> {
        let node_path = std::path::Path::new(nodes_filename);
        let way_path = std::path::Path::new(ways_filename);

        

        let node_file = std::fs::File::open(node_path).map_err(|e| e.to_string())?;
        let way_file = std::fs::File::open(way_path).map_err(|e| e.to_string())?;
       

        self.read_nodes(node_file).map_err(|e| e.to_string())?;
        self.read_ways(way_file).map_err(|e| e.to_string())?;

        self.count_nodes_uses();


        // let nodes_file = std::fs::File::open(nodes_filename);
        // self.read_nodes(nodes_file);

        // let ways_file = File::open(ways_filename)?;
        // self.read_ways(ways_file)?;

        // self.count_nodes_uses();

        Ok((self.nodes(), self.edges()))
    }
}

pub fn csv_read(nodes_filename: &str, ways_filename: &str) -> Result<(Vec<Node>, Vec<Edge>), String> {
    CsvReader::new().read(nodes_filename, ways_filename)
}

// #[test]
// fn test_real_all() {
//     let (nodes, ways) = read("src/osm4routing/test_data/minimal.osm.pbf").unwrap();
//     assert_eq!(2, nodes.len());
//     assert_eq!(1, ways.len());
// }

// #[test]
// fn test_count_nodes() {
//     let ways = vec![Way {
//         id: WayId(0),
//         nodes: Nodes(vec![NodeId(1), NodeId(2), NodeId(3)]),
//         properties: EdgeProperties::default(),
//     }];
//     let mut nodes = HashMap::new();
//     nodes.insert(NodeId(1), Node::default());
//     nodes.insert(NodeId(2), Node::default());
//     nodes.insert(NodeId(3), Node::default());
//     let mut r = Reader {
//         ways,
//         nodes,
//         ..Default::default()
//     };
//     r.count_nodes_uses();
//     assert_eq!(2, r.nodes[&NodeId(1)].uses);
//     assert_eq!(1, r.nodes[&NodeId(2)].uses);
//     assert_eq!(2, r.nodes[&NodeId(3)].uses);

//     assert_eq!(2, r.nodes().len());
// }

// #[test]
// fn test_split() {
//     let mut nodes = HashMap::new();
//     nodes.insert(NodeId(1), Node::default());
//     nodes.insert(NodeId(2), Node::default());
//     nodes.insert(NodeId(3), Node::default());
//     nodes.insert(NodeId(4), Node::default());
//     nodes.insert(NodeId(5), Node::default());
//     let ways = vec![
//         Way {
//             id: WayId(0),
//             nodes: Nodes(vec![NodeId(1), NodeId(2), NodeId(3)]),
//             properties: EdgeProperties::default(),
//         },
//         Way {
//             id: WayId(0),
//             nodes: Nodes(vec![NodeId(4), NodeId(5), NodeId(2)]),
//             properties: EdgeProperties::default(),
//         },
//     ];
//     let mut r = Reader {
//         nodes,
//         ways,
//         ..Default::default()
//     };
//     r.count_nodes_uses();
//     let edges = r.edges();
//     assert_eq!(3, edges.len());
// }

// #[test]
// fn test_wrong_file() {
//     let r = read("i hope you have no file name like this one");
//     assert!(r.is_err());
// }

// #[test]
// fn forbidden_values() {
//     let (_, ways) = Reader::new()
//         .reject("highway", "secondary")
//         .read("src/osm4routing/test_data/minimal.osm.pbf")
//         .unwrap();
//     assert_eq!(0, ways.len());
// }

// #[test]
// fn forbidden_wildcard() {
//     let (_, ways) = Reader::new()
//         .reject("highway", "*")
//         .read("src/osm4routing/test_data/minimal.osm.pbf")
//         .unwrap();
//     assert_eq!(0, ways.len());
// }

// #[test]
// fn way_of_node() {
//     let mut r = Reader::new();
//     let (_nodes, edges) = r.read("src/osm4routing/test_data/minimal.osm.pbf").unwrap();

//     assert_eq!(2, edges[0].nodes.len());
// }
