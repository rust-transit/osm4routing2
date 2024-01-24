use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    source_pbf: String,
}
fn main() {
    let cli = Cli::parse();

    match osm4routing::read(&cli.source_pbf) {
        Ok((nodes, edges)) => osm4routing::writers::csv(nodes, edges),
        Err(error) => println!("Error: {}", error),
    }
}
