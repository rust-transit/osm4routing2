//! Convert OpenStreetMap data into routing-friendly graphs.
//!
//! This library extracts routing graphs from OpenStreetMap PBF files,
//! handling accessibility categorization, edge splitting, and optional
//! merging of continuous ways.
//!
//! # Quick Start
//!
//! The simplest way to use the library is with the [`read`] function:
//!
//! ```no_run
//! let (nodes, edges) = osm4routing::read("map.osm.pbf").unwrap();
//! println!("Loaded {} nodes and {} edges", nodes.len(), edges.len());
//! ```
//!
//! # Filtering Data
//!
//! Use [`Reader`] for more control over which data is extracted:
//!
//! ```no_run
//! use osm4routing::Reader;
//!
//! let (nodes, edges) = Reader::new()
//!     .reject("highway", "footway")      // Exclude footways
//!     .require("highway", "primary")     // Only include primary roads
//!     .read("map.osm.pbf")
//!     .unwrap();
//! ```
//!
//! # Preserving Tags
//!
//! By default, only computed accessibility properties are stored.
//! Use `read_tag` to preserve specific OSM tags:
//!
//! ```no_run
//! use osm4routing::Reader;
//!
//! let (_nodes, edges) = Reader::new()
//!     .read_tag("name")
//!     .read_tag("maxspeed")
//!     .read("map.osm.pbf")
//!     .unwrap();
//!
//! if let Some(name) = edges[0].tags.get("name") {
//!     println!("Road name: {}", name);
//! }
//! ```
//!
//! # Merging Ways
//!
//! Enable merging to combine consecutive edges that have been split
//! by OSM tag changes but are topologically continuous:
//!
//! ```no_run
//! use osm4routing::Reader;
//!
//! let (nodes, edges) = Reader::new()
//!     .merge_ways()
//!     .read("map.osm.pbf")
//!     .unwrap();
//! ```
//!
//! # Exporting Data
//!
//! Export the graph to CSV format:
//!
//! ```no_run
//! let (nodes, edges) = osm4routing::read("map.osm.pbf").unwrap();
//! osm4routing::writers::csv(nodes, edges, "nodes.csv", "edges.csv").unwrap();
//! ```
//!
//! # Module Overview
//!
//! - [`models`]: Core data structures ([`Node`], [`Edge`])
//! - [`categorize`]: Transportation mode accessibility enums
//! - [`reader`]: PBF file reading and graph construction
//! - [`writers`]: Output formats (CSV)
//! - [`error`]: Error types

mod osm4routing;

pub use crate::osm4routing::categorize::{
    BikeAccessibility, CarAccessibility, FootAccessibility, TrainAccessibility,
};
pub use crate::osm4routing::error::Error;
pub use crate::osm4routing::models::*;
pub use crate::osm4routing::reader::{Reader, read};
pub use crate::osm4routing::writers;

// Reexpose crates that are part of the API
// It helps consumer being sure they always use the correct version of types
pub use ahash;
pub use osmpbfreader;
