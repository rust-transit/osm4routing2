use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The osm file to be parsed in the .pbf format
    source_pbf: String,
    #[arg(short, long)]
    /// Merge two edges from different OSM ways into a single edge when there is no intersection
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
        Ok((nodes, edges)) => osm4routing::writers::csv(nodes, edges),
        Err(error) => println!("Error: {}", error),
    }
}
