use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input OpenStreetMap in the .pbf format
    source_pbf: String,
    /// Output path of the csv file that will contain the nodes
    #[arg(short, long, default_value = "nodes.csv")]
    nodes_file: String,
    /// Output path of the csv file that will contain the edges
    #[arg(short, long, default_value = "edges.csv")]
    edges_file: String,
    /// Merge two edges from different OSM ways into a single edge when there is no intersection
    #[arg(short, long)]
    merge_edges: bool,
}
fn main() {
    let cli = Cli::parse();
    let mut reader = if cli.merge_edges {
        osm4routing::Reader::new().merge_ways()
    } else {
        osm4routing::Reader::new()
    };

    match reader.read(&cli.source_pbf) {
        Ok((nodes, edges)) => {
            osm4routing::writers::csv(nodes, edges, &cli.nodes_file, &cli.edges_file)
        }
        Err(error) => println!("Error: {}", error),
    }
}
