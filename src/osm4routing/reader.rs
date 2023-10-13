use csv::StringRecord;

use super::categorize::*;
use super::models::*;
use std::collections::{HashMap, HashSet};
use std::error::Error;

#[derive(Default)]
pub struct Reader {
  nodes: HashMap<NodeId, Node>,
  ways: Vec<Way>,
  nodes_to_keep: HashSet<NodeId>,
  forbidden: HashMap<String, HashSet<String>>,
}

impl Reader {
  pub fn new() -> Reader {
    Reader::default()
  }

  pub fn reject(mut self, key: &str, value: &str) -> Self {
    self.forbidden
      .entry(key.to_string())
      .or_default()
      .insert(value.to_string());
    self
  }

  /// Performs sanity checks on nodes and ways for debugging purposes.
  ///
  /// This function checks for missing nodes in ways and removes the ways with missing nodes.
  /// It also prints statistics about the removed ways and missing nodes to aid in debugging.
  /// This function is intended for debugging purposes and should not be used in production code.
  fn sanity_checks_nodes(&mut self) {
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

    // remove ways with missing nodes
    for way_index in ways_to_remove.iter().rev() {
        self.ways.remove(*way_index);
    }

    // print stats
    for (way_index, node_id) in missing_node_stats {
      println!("Way {} is missing node {}", way_index, node_id);
    }
    println!("{} ways removed ({:.2}%) because of missing nodes", removed_ways_count, removed_ways_percentage);
    println!("Missing nodes stats:");
  }

  /// Splits a way into multiple edges based on nodes with more than one usage.
  ///
  /// # Arguments
  ///
  /// * `way` - The way to split into edges.
  ///
  /// # Returns
  ///
  /// A vector of edges resulting from splitting the way. Each edge represents a segment of the original way
  /// between nodes with more than one usage. The vector is empty if the way does not need to be split.
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


  fn nodes(&self) -> Vec<Node> {
    self.nodes
      .values()
      .filter(|node| node.uses > 1)
      .copied()
      .collect()
  }

  fn ways(&self) -> Vec<Edge> {
    self.ways
      .iter()
      .flat_map(|way| self.split_way(way))
      .collect()
  }

  
}

pub struct CsvReader {
  reader: Reader,
}

impl CsvReader {
  pub fn new() -> CsvReader {
    CsvReader {
      reader: Reader::new(),
    }
  }

  pub fn reject(mut self, key: &str, value: &str) -> Self {
    self.reader = self.reader.reject(key, value);
    self
  }

  /// Reads the nodes and ways from the given CSV files and returns them.
  ///
  /// # Arguments
  ///
  /// * `nodes_file` - The path to the CSV file containing node data.
  /// * `ways_file` - The path to the CSV file containing way data.
  ///
  /// # Returns
  ///
  /// A tuple containing vectors of nodes and edges, wrapped in a `Result`. If successful,
  /// the vectors are returned. Otherwise, an error is returned as `Box<dyn Error>`.
  pub fn read(&mut self, nodes_file: &str, ways_file: &str) -> Result<(Vec<Node>, Vec<Edge>), Box<dyn Error>> {
    let nodes_file = std::fs::File::open(nodes_file)?;
    self.read_nodes(nodes_file)?;

    let ways_file = std::fs::File::open(ways_file)?;
    self.read_ways(ways_file)?;

    self.reader.sanity_checks_nodes();
    Ok((self.reader.nodes(), self.reader.ways()))
  }

  /// Reads the ways from the given CSV file.
  ///
  /// # Arguments
  ///
  /// * `file` - The CSV file containing the way data.
  ///
  /// # Returns
  ///
  /// An empty `Result` indicating success or an error as `Box<dyn Error>`.
  fn read_ways(&mut self, file: std::fs::File) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
      .has_headers(true)
      .from_reader(file);

    for result in reader.records() {
      let record = result?;
      self.consume_way(&record)?;
    }

    Ok(())
  }


  pub fn read_records(&mut self, nodes_record: Vec<StringRecord>, ways_record: Vec<StringRecord>) -> Result<(Vec<Node>, Vec<Edge>), Box<dyn Error>> {
    self.read_nodes_record(nodes_record)?;
    self.read_ways_record(ways_record)?;
    self.reader.sanity_checks_nodes();
    Ok((self.reader.nodes(), self.reader.ways()))
  }

  fn read_ways_record(&mut self, record: Vec<StringRecord>) -> Result<(), Box<dyn Error>> {
    for result in record {
      self.consume_way(&result)?;
    }
    Ok(())
  }

  fn read_nodes_record(&mut self, record: Vec<StringRecord>) -> Result<(), Box<dyn Error>> {
    for result in record {
      self.consume_node(&result)?;
    }
    Ok(())
  }

  /// Reads the nodes from the given CSV file.
  /// 
  /// # Arguments
  /// 
  /// * `file` - The CSV file containing the node data.
  /// 
  /// # Returns
  /// 
  /// An empty `Result` indicating success or an error as `Box<dyn Error>`.
  fn read_nodes(&mut self, file: std::fs::File) -> Result<(), Box<dyn Error>> {
    let mut reader = csv::ReaderBuilder::new()
      .has_headers(true)
      .from_reader(file);

    for result in reader.records() {
      let record = result?;
      self.consume_node(&record)?;
    }

    Ok(())
  }

  /// Consumes a way from the given CSV record.
  /// 
  /// # Arguments
  /// 
  /// * `record` - The CSV record containing the way data in string format.
  /// 
  /// # Returns
  /// 
  /// An empty `Result` indicating success or an error as `Box<dyn Error>`.
  fn consume_way(&mut self, record: &StringRecord) -> Result<(), Box<dyn Error>> {
    let way_id: WayId = WayId(record[0].parse::<i64>()?);
    let nodes_str = record[1].trim();
    let nodes: Vec<NodeId> = nodes_str
      .trim_start_matches('[')
      .trim_end_matches(']')
      .split(", ")
      .map(|id| NodeId(id.parse::<i64>().unwrap()))
      .collect();

    let way = Way {
      id: way_id,
      nodes: Nodes(nodes),
      properties: Default::default(),
    };

    self.reader.ways.push(way);

    Ok(())
  }

  /// Consumes a node from the given CSV record.
  /// 
  /// # Arguments
  /// 
  /// * `record` - The CSV record containing the node data in string format.
  /// 
  /// # Returns
  /// 
  /// An empty `Result` indicating success or an error as `Box<dyn Error>`.
  fn consume_node(&mut self, record: &StringRecord) -> Result<(), Box<dyn Error>> {
    let node_id: NodeId = NodeId(record[0].parse::<i64>()?);
    let lat: f64 = record[1].parse::<f64>()?;
    let lon: f64 = record[2].parse::<f64>()?;
    let node = Node {
      id: node_id,
      coord: Coord { lat, lon },
      uses: 0,
    };

    self.reader.nodes.insert(node_id, node);

    Ok(())
  }
}

pub struct OsmReader {
  reader: Reader,
}

impl OsmReader {
  pub fn new() -> OsmReader {
    OsmReader {
      reader: Reader::new(),
    }
  }

  pub fn reject(mut self, key: &str, value: &str) -> Self {
    self.reader = self.reader.reject(key, value);
    self
  }

  /// Reads the nodes and ways from the given OSM files and returns them.
  /// 
  /// # Arguments
  /// 
  /// * `filename` - The path to the OSM file containing node data.
  /// 
  /// # Returns
  /// 
  /// A tuple containing vectors of nodes and edges, wrapped in a `Result`. If successful,
  /// the vectors are returned. Otherwise, an error is returned as `Box<dyn Error>`.
  fn read(&mut self, filename: &str) -> Result<(Vec<Node>, Vec<Edge>), Box<dyn Error>> {
    let path = std::path::Path::new(filename);
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let _ = self.read_ways(file);
    let file_nodes = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let _ = self.read_nodes(file_nodes);
    
    self.reader.sanity_checks_nodes();
    Ok((self.reader.nodes(), self.reader.ways()))
  }

  /// Reads the ways from the given OSM files.
  /// 
  /// # Arguments
  /// 
  /// * `file` - The path to the OSM file containing way data.
  /// 
  /// # Returns
  /// 
  /// An empty `Result` indicating success or an error as `Box<dyn Error>`.
  fn read_ways(&mut self, file: std::fs::File) -> Result<(), Box<dyn Error>> {
    let mut reader = osmpbfreader::OsmPbfReader::new(file);

    for obj in reader.iter().flatten() {
      if let osmpbfreader::OsmObj::Way(way) = obj {
        self.consume_way(way)?;
      }
    }

    Ok(())
  }

  /// Reads the nodes from the given OSM file.
  /// 
  /// # Arguments
  /// 
  /// * `file` - The path to the OSM file containing node data.
  /// 
  /// # Returns
  /// 
  /// An empty `Result` indicating success or an error as `Box<dyn Error>`.
  fn read_nodes(&mut self, file: std::fs::File) -> Result<(), Box<dyn Error>> {
    let mut reader = osmpbfreader::OsmPbfReader::new(file);

    for obj in reader.iter().flatten() {
      if let osmpbfreader::OsmObj::Node(node) = obj {
        self.consume_node(node)?;
      }
    }

    Ok(())
  }

  /// Consumes a wat from the given OSM way.
  /// 
  /// # Arguments
  /// 
  /// * `record` - The OSM way to consume.
  /// 
  /// # Returns
  /// 
  /// An empty `Result` indicating success or an error as `Box<dyn Error>`.
  fn consume_way(&mut self, record: osmpbfreader::Way) -> Result<(), Box<dyn Error>> {
    let mut skip = false;
    let mut properties = EdgeProperties::default();

    for (key, val) in record.tags.iter() {
      properties.update(key.to_string(), val.to_string());

      skip |= self.reader.forbidden
        .get(key.as_str())
        .map(|vals| vals.contains(val.as_str()) || vals.contains("*")) == Some(true);
    }

    properties.normalize();
    if properties.accessible() && !skip {
      for node in &record.nodes {
        self.reader.nodes_to_keep.insert(node.into());
      }

      let way = Way {
        id: WayId(record.id.0),
        nodes: record.nodes.into(),
        properties,
      };

      self.reader.ways.push(way);
    }

    Ok(())
  }

  /// Consumes a node from the given OSM node.
  /// 
  /// # Arguments
  /// 
  /// * `record` - The OSM node to consume.
  /// 
  /// # Returns
  /// 
  /// An empty `Result` indicating success or an error as `Box<dyn Error>`.
  fn consume_node(&mut self, record: osmpbfreader::Node) -> Result<(), Box<dyn Error>> {
    let node_id: NodeId = record.id.into();
    if self.reader.nodes_to_keep.contains(&node_id) {
      self.reader.nodes_to_keep.remove(&node_id);

      let node = Node {
        id: node_id,
        coord: Coord { lon: record.lon(), lat: record.lat() },
        uses: 0,
      };

      self.reader.nodes.insert(node_id, node);
    }

    Ok(())
  }
}

pub fn csv_read(nodes_file: &str, ways_file: &str) -> Result<(Vec<Node>, Vec<Edge>), Box<dyn Error>> {
  CsvReader::new().read(nodes_file, ways_file)
}
pub fn record_read(nodes_record: Vec<StringRecord>, ways_record: Vec<StringRecord>) -> Result<(Vec<Node>, Vec<Edge>), Box<dyn Error>> {
  CsvReader::new().read_records(nodes_record, ways_record)
}




pub fn osm_read(filename: &str) -> Result<(Vec<Node>, Vec<Edge>), Box<dyn Error>> {
  OsmReader::new().read(filename)
}

#[test]
fn test_osm_read_all() {
  let (nodes, ways) = osm_read("src/osm4routing/test_data/minimal.osm.pbf").unwrap();
  assert_eq!(nodes.len(), 2);
  assert_eq!(ways.len(), 1);
}
