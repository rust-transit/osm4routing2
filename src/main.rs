extern crate osm4routing;

fn main() {
    let (nodes, edges) = osm4routing::reader::read("idf.osm.pbf");
    osm4routing::writers::csv(nodes, edges);
}
