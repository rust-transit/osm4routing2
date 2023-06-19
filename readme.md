# osm4routing2

This project is a rewrite in rust from https://github.com/Tristramg/osm4routing

It converts an OpenStreetMap file (in the `.pbf` format) into a CSV file.

## Build
Get a rust distribution with `cargo`: https://www.rust-lang.org/en-US/downloads.html

Run `cargo install osm4routing`

You can now use `osm4routing <some_osmfile.pbf>` to generate the `nodes.csv` and `edges.csv` that represent the road network.

If you prefer running the application from the sources, and not installing it, you run

`cargo run --release -- <path_to_your_osmfile.pbf>`

The identifiers for nodes and edges are from OpenStreetMap.

The `id` property of an edge is unique, while the `osm_id` can be duplicated.

## Importing in a database

If you prefer having the files in database, you can run the very basic `import_postgres.sh` script.

It supposes that a database `osm4routing` exists (otherwise modify it to your needs).

## Using as a library

In order to use osm4routing as a library, add `osm4routing = "*"` in your `Cargo.toml` file in the `[dependencies]` section.

Use it:

```
let (nodes, edges) = osm4routing::read("some_data.osm.pbf")?;

```

If you wand to reject certain edges based on their tag, use the `Reader` (it also accepts "*" to reject every value):

```
let (nodes, edges) = osm4routing::Reader::new().reject("area", "yes").read("some_data.osm.pbf")?;

```

If you need to read some tags, pass them to the `reader`:

```
let (nodes, edges) = osm4routing::Reader::new().read_tag("highway").read("some_data.osm.pbf")?;

```
