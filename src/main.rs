fn main() {
    const USAGE: &str = "
Usage: osm4routing <source.osm.pbf>";
    let args = docopt::Docopt::new(USAGE)
        .unwrap()
        .parse()
        .unwrap_or_else(|e| e.exit());
    let filename = args.get_str("<source.osm.pbf>");
    match osm4routing::reader::read(filename) {
        Ok((nodes, edges)) => osm4routing::writers::csv(nodes, edges),
        Err(error) => println!("Error: {}", error),
    }
}
