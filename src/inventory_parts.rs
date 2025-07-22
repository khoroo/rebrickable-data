use csv::Reader;
use serde::Deserialize;
use std::error::Error;
use std::collections::HashMap;
use std::fs::{File, create_dir_all};
use std::path::Path;
use sprs::{TriMat, CsMat};

/// Represents a single record from the inventory_parts.csv file.
#[derive(Debug, Deserialize)]
struct InventoryItem {
    inventory_id: i32,
    part_num: String,
    quantity: i32,
    #[serde(deserialize_with = "deserialize_bool_from_string")]
    is_spare: bool,
    // color_id and img_url are deserialized but not used in the current logic.
    // color_id: String,
    // img_url: String,
}

fn deserialize_bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => Err(serde::de::Error::custom("Expected 'True' or 'False'")),
    }
}

fn read_inventory_from_csv(file_path: &str) -> Result<Vec<InventoryItem>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);

    // Use collect() to gather all deserialized records into a Vec.
    // The `?` operator will propagate any errors that occur during deserialization.
    let items = reader.deserialize().collect::<Result<Vec<_>, _>>()?;

    Ok(items)
}

pub fn main_with_output_dir(output_dir: &str) -> Result<(), Box<dyn Error>> {
    // Create output directory if it doesn't exist
    create_dir_all(output_dir)?;
    println!("Output directory: {}", output_dir);
    
    let raw_items = read_inventory_from_csv("data/inventory_parts.csv")?;
    
    // Filter out spare parts
    let filtered_items: Vec<_> = raw_items.into_iter()
        .filter(|item| !item.is_spare)
        .collect();
    
    println!("Filtered {} non-spare records from CSV.", filtered_items.len());
    
    // Group by (inventory_id, part_num) and sum quantities
    let mut quantity_map: HashMap<(i32, String), i32> = HashMap::new();
    
    for item in filtered_items {
        let key = (item.inventory_id, item.part_num);
        *quantity_map.entry(key).or_insert(0) += item.quantity;
    }
    
    println!("Grouped into {} unique (inventory_id, part_num) pairs.", quantity_map.len());
    
    // Create index mappings
    let mut inventory_ids = Vec::new();
    let mut part_nums = Vec::new();
    let mut inventory_id_to_index: HashMap<i32, usize> = HashMap::new();
    let mut part_num_to_index: HashMap<String, usize> = HashMap::new();
    
    // Collect unique inventory_ids and part_nums in order of appearance
    for ((inventory_id, part_num), _) in &quantity_map {
        // Add inventory_id if not seen before
        if !inventory_id_to_index.contains_key(inventory_id) {
            inventory_id_to_index.insert(*inventory_id, inventory_ids.len());
            inventory_ids.push(*inventory_id);
        }
        
        // Add part_num if not seen before
        if !part_num_to_index.contains_key(part_num) {
            part_num_to_index.insert(part_num.clone(), part_nums.len());
            part_nums.push(part_num.clone());
        }
    }
    
    println!("Matrix dimensions: {} rows (inventory_ids) × {} cols (part_nums)", 
             inventory_ids.len(), part_nums.len());
    
    // Create CSV files for row and column mappings
    let row_mapping_path = Path::new(output_dir).join("row_mapping.csv");
    let col_mapping_path = Path::new(output_dir).join("col_mapping.csv");
    
    // Write row mapping (inventory_ids) to CSV
    let mut row_writer = csv::Writer::from_path(&row_mapping_path)?;
    row_writer.write_record(&["inventory_id"])?;
    for inventory_id in &inventory_ids {
        row_writer.write_record(&[&inventory_id.to_string()])?;
    }
    row_writer.flush()?;
    println!("Saved row mapping to {}", row_mapping_path.display());
    
    // Write column mapping (part_nums) to CSV
    let mut col_writer = csv::Writer::from_path(&col_mapping_path)?;
    col_writer.write_record(&["part_num"])?;
    for part_num in &part_nums {
        col_writer.write_record(&[part_num])?;
    }
    col_writer.flush()?;
    println!("Saved column mapping to {}", col_mapping_path.display());
    
    // Create sparse matrix using sprs TriMat (triplet format)
    let mut trimat: TriMat<i32> = TriMat::new((inventory_ids.len(), part_nums.len()));
    
    // Add triplets to the matrix
    for ((inventory_id, part_num), quantity) in quantity_map {
        let row = inventory_id_to_index[&inventory_id];
        let col = part_num_to_index[&part_num];
        trimat.add_triplet(row, col, quantity);
    }
    
    // Convert to compressed sparse row format
    let csr_mat: CsMat<i32> = trimat.to_csr();
    
    // Write matrix using sprs write_matrix_market function
    let mtx_path = Path::new(output_dir).join("matrix.mtx");
    sprs::io::write_matrix_market(&mtx_path, &csr_mat)?;
    
    println!("Saved sparse matrix to {} in Matrix Market format", mtx_path.display());
    
    Ok(())
}
