fn main() {
    const USAGE: &str = "
Usage: osm4routing <source.osm.pbf> [--edge-file <EDGE_FILE> --node-file <NODE_FILE>]
Options:
    --edge-file EDGE_FILE File to output edges to. Default is 'edges.csv'.
    --node-file NODE_FILE File to output nodes to. Default is 'nodes.csv'.
    ";
    let args = docopt::Docopt::new(USAGE)
        .unwrap()
        .parse()
        .unwrap_or_else(|e| e.exit());
    let filename = args.get_str("<source.osm.pbf>");
    let edge_output = match args.get_str("<EDGE_FILE>") {
        "" => "edges.csv",
        s => s,
    };

    let node_output = match args.get_str("<NODE_FILE>") {
        "" => "nodes.csv",
        s => s,
    };
    match osm4routing::read(filename) {
        Ok((nodes, edges)) => osm4routing::writers::csv(nodes, edges, edge_output, node_output),
        Err(error) => println!("Error: {}", error),
    }
}
