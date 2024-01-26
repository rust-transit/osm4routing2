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
}
fn main() {
    let cli = Cli::parse();

    match osm4routing::read(&cli.source_pbf) {
        Ok((nodes, edges)) => {
            osm4routing::writers::csv(nodes, edges, &cli.nodes_file, &cli.edges_file)
        }
        Err(error) => println!("Error: {}", error),
    }
}
