extern crate serde;
extern crate serde_json;
extern crate csv;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use csv::StringRecord;


pub fn merge_csv_ways(
    filenames: Vec<&str>,
    merged_filename: &str,
    adjacent_geohashes: Vec<&str>,
    ways_to_load: HashSet<&str>,
) -> Result<(Vec<StringRecord>, Vec<i64>), Box<dyn Error>> {
    let mut merged_data = Vec::new();
    let merged_filename = Path::new(merged_filename);
    let mut print_header = true;
    let mut skip_header;
    let mut nodes_in_ways_to_load = Vec::new();

    let merged_nodes_file = File::create(merged_filename)?;
    let mut merged_nodes_writer = csv::Writer::from_writer(merged_nodes_file);
    for filename in filenames {
        skip_header = true;
        let skip_full_read = !adjacent_geohashes.iter().any(|geohash| filename.contains(geohash));
        if !Path::new(filename).exists() {
            eprintln!("Missing tile file: {}", filename);
            continue;
        }

        let file = File::open(filename)?;
        let mut reader = csv::ReaderBuilder::new()
        .has_headers(false) // Depending on your data, set this to true or false
        .delimiter(b',')    // Set the custom delimiter as a byte
        .from_reader(file);

        for result in reader.records(){
            let record = result?;
            let id = record.get(0).unwrap_or("");
            let node_list = record.get(1).unwrap_or("");
            
            if skip_header {
                skip_header = false;
                if print_header{
                    merged_nodes_writer.write_record(&record)?;
                }
                print_header=false;
                continue
            } else {
                if skip_full_read {
                    if filename.contains("_ways.csv") {
                        if !ways_to_load.contains(id) {
                            continue;
                        }
                        
                    }
                }
                nodes_in_ways_to_load.extend(node_list.split(",").map(|s| s.replace(&['[', ']','\"',' '][..], "").parse::<i64>().unwrap()));
            }
            merged_data.push(record);
        }
    }

    for result in merged_data.clone() {
        let record = result;
        merged_nodes_writer.write_record(&record)?;
    }

    Ok((merged_data, nodes_in_ways_to_load))
}


pub fn merge_csv_nodes(
    filenames: Vec<&str>,
    adjacent_geohashes: Vec<&str>,
    nodes_to_load: HashSet<i64>,
) -> Result<Vec<StringRecord>, Box<dyn Error>> {
    let mut merged_data = Vec::new();
    let mut skip_header;
    for filename in filenames {
        skip_header = true;
        let skip_full_read = !adjacent_geohashes.iter().any(|geohash| filename.contains(geohash));
        if !Path::new(filename).exists() {
            eprintln!("Missing tile file: {}", filename);
            continue;
        }

        let file = File::open(filename)?;
        let mut reader = csv::ReaderBuilder::new()
        .has_headers(false) // Depending on your data, set this to true or false
        .delimiter(b',')    // Set the custom delimiter as a byte
        .from_reader(file);
        for result in reader.records(){
            let record = result?;
            let node_id = record.get(0).unwrap_or("");
            if skip_header {
                skip_header = false;
                continue
            } else {
                if skip_full_read {
                    if filename.contains("_nodes.csv") {
                            if !nodes_to_load.contains(&node_id.parse::<i64>().unwrap()) {
                                continue;
                            }
                    } 
                }
            }
            merged_data.push(record);
        }
    }
    Ok(merged_data)
}