use osm4routing::{record_read, writers, loader, Node, Edge};
use std::collections::HashSet;
use csv::StringRecord;
const USAGE: &str = "
Usage: osm4routing [--wayfilenames=<wayfilenames>] [--nodefilenames=<nodefilenames>] [--adjacentgeohashes=<adjacent_geohashes>] [--waystoload=<ways_to_load>] [--format=<output_format>] [--output=<output>]
Options:
    --wayfilenames=<wayfilenames>  list of way files
    --adjacentgeohashes=<adjacent_geohashes> list of the 9 geohashes around our current position
    --waystoload=<adjacent_geohashes> list of ways that pass through the 9 geohashes, without starting in one of them
    --nodefilenames=<nodefilenames> list of node files
    --output=<output> output_path for the merged geojson or csv
    --format=<output_format>  Output format (csv or geojson) [default: csv]
    ";

fn main() {
    let args = docopt::Docopt::new(USAGE)
        .unwrap()
        .parse()
        .unwrap_or_else(|e| e.exit());

    let fmt = args.get_str("--format");
    let way_files_iterator = args.get_str("--wayfilenames").split(",");
    let adjacentgeohashes_iterator = args.get_str("--adjacentgeohashes").split(",");
    let ways_to_load_iterator = args.get_str("--waystoload").split(","); 
    let node_files_iterator = args.get_str("--nodefilenames").split(",");

    let way_files: Vec<&str> = way_files_iterator.collect();
    let node_files: Vec<&str> = node_files_iterator.collect();
    let adjacentgeohashes: Vec<&str> = adjacentgeohashes_iterator.collect();
    let ways_to_load_vec: Vec<&str> = ways_to_load_iterator.collect();

    let mut nodes_to_load: Vec<i64>=Vec::new();
    let mut merged_way_records:  Vec<StringRecord>=Vec::new();
    let mut merged_node_records:  Vec<StringRecord>=Vec::new();
    
    let output = match args.get_str("--output") {
        "" => ".",
        o => {
            generate_output_folder(o);
            o
        },
    };  

    // Convert the Vec<&str> into a HashSet<&str> for quicker searches
    let ways_to_load_set: HashSet<&str> = ways_to_load_vec.into_iter().collect();
    match loader::merge_csv_ways(way_files,&[output,"/way_properties.csv"].join(""), adjacentgeohashes.clone(), ways_to_load_set) {
        Ok((f_records, nodes)) => {
        merged_way_records=f_records;
        nodes_to_load=nodes;},
        Err(e) => println!("Error: {}", e),
    };
    let nodes_to_load_hash: HashSet<i64> = HashSet::from_iter(nodes_to_load.iter().cloned());
    match loader::merge_csv_nodes(node_files, adjacentgeohashes, nodes_to_load_hash) {
        Ok(f_records) => {
        merged_node_records=f_records;},
        Err(e) => println!("Error: {}", e),
    };
    match record_read(merged_node_records, merged_way_records)  {
        Ok((nodes, edges)) => {handle_output_format(fmt, nodes, edges, output)},
        Err(e) =>  println!("Error: {}", e),
    }
}


fn handle_output_format(fmt: &str, nodes: Vec<Node>, edges: Vec<Edge>, output_path: &str) {
    match fmt {
        "csv" | "" => writers::csv(nodes, edges ,output_path),
        "geojson" => writers::geojson(nodes, edges, output_path),
        _ => println!("Invalid output format. Please choose csv or geojson."),
    }
}

fn generate_output_folder(output_path: &str) {
    let path = std::path::Path::new(output_path);
    if !path.exists() {
        std::fs::create_dir(path).expect("Unable to create output folder");
    }
}