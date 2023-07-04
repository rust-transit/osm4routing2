
use osm4routing::{osm_read, csv_read, writers, Node, Edge};

const USAGE: &str = "
Usage: osm4routing [--format=<output_format>] [--output=<path>] (<source.osm.pbf> | <nodes.csv> <ways.csv>)
Options:
    --format=<output_format>  Output format (csv or geojson) [default: csv]
    --output=<path>           Output directory [default: .]";

fn main() {
    let args = docopt::Docopt::new(USAGE)
        .unwrap()
        .parse()
        .unwrap_or_else(|e| e.exit());

    let fmt = args.get_str("--format");
    let output = match args.get_str("--output") {
        "" => ".",
        o => {
            generate_output_folder(o);
            o
        },
    };  


    if args.get_bool("<source.osm.pbf>") {
        match osm_read(args.get_str("<source.osm.pbf>")) {
            Ok((nodes, edges)) => handle_output_format(fmt, nodes, edges, output),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        
        match csv_read(args.get_str("<nodes.csv>"), args.get_str("<ways.csv>"))  {
            Ok((nodes, edges)) => handle_output_format(fmt, nodes, edges, output),
            Err(e) => println!("Error: {}", e),
        }
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