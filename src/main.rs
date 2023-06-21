fn main() {
    const USAGE: &str = "
Usage: osm4routing <source.osm.pbf> [--format=<output_format>]
Options:
    --format=<output_format>  Output format (csv or geojson) [default: csv]";

    let args = docopt::Docopt::new(USAGE)
        .unwrap()
        .parse()
        .unwrap_or_else(|e| e.exit());
    let filename = args.get_str("<source.osm.pbf>");

    match osm4routing::read(filename) {
        Ok((nodes, edges)) => {
            let fmt = args.get_str("--format");
            match fmt {
                "csv" | "" => osm4routing::writers::csv(nodes, edges),
                "geojson" => osm4routing::writers::geojson(nodes, edges),
                _ => println!("Invalid output format. Please choose csv or geojson."),
            }
        },
        Err(error) => println!("Error: {}", error),
    }
}
