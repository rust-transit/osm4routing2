mod lib;
fn main() {
    let (nodes, edges) = lib::reader::read("idf.osm.pbf");
    lib::writers::write(nodes, edges);
}
