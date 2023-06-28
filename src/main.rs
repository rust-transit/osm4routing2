
use osm4routing::{osm_read, csv_read, writers, Node, Edge};

const USAGE: &str = "
Usage: osm4routing [--format=<output_format>] (<source.osm.pbf> | <nodes.csv> <ways.csv>)
Options:
    --format=<output_format>  Output format (csv or geojson) [default: csv]";

fn main() {
    let args = docopt::Docopt::new(USAGE)
        .unwrap()
        .parse()
        .unwrap_or_else(|e| e.exit());

    let fmt = args.get_str("--format");

    if args.get_bool("<source.osm.pbf>") {
        match osm_read(args.get_str("<source.osm.pbf>")) {
            Ok((nodes, edges)) => handle_output_format(fmt, nodes, edges),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        
        match csv_read(args.get_str("<nodes.csv>"), args.get_str("<ways.csv>"))  {
            Ok((nodes, edges)) => handle_output_format(fmt, nodes, edges),
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn handle_output_format(fmt: &str, nodes: Vec<Node>, edges: Vec<Edge>) {
    match fmt {
        "csv" | "" => writers::csv(nodes, edges),
        "geojson" => writers::geojson(nodes, edges),
        _ => println!("Invalid output format. Please choose csv or geojson."),
    }
}
