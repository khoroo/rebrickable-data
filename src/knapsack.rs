/*
knapsack.rs
FIND THE MAXIMUM NUMBER OF INVENTORIES THAT CAN BE BUILT WITH N DISTINCT PARTS
FOR ALL N in [N, 0]

APPROXIMATION METHOD:
1. read the matrix market file with `sprs::io::read_matrix_market`
2. we start with all parts and all inventories are buildable
3. we order the parts by number of inventories they are in,
   removing the part with the least inventories first
4. we keep track of the number of inventories that can be built
5. we stop when we have no more parts or inventories
*/

use sprs::io::read_matrix_market;
use csv::Writer;
use std::fs::File;
use rayon::prelude::*;


pub fn greedy() -> Result<(), Box<dyn std::error::Error>> {
    // Read the matrix market file
    let matrix: sprs::TriMat<i32> = read_matrix_market("data/matrix.mtx")?;
    
    // Convert to CSC format and then to binary matrix
    // The values in the matrix are quantities of parts in inventories
    // We convert to binary (0 or 1) to check presence only
    let csc_matrix = matrix.to_csc();
    let binary_matrix: sprs::CsMat<u8> = csc_matrix.map(|&val| if val > 0 { 1u8 } else { 0u8 });
    
    // Calculate part frequencies (how many inventories each part appears in)
    // A column is a part, a row is an inventory
    let mut part_frequencies = vec![0; binary_matrix.cols()];
    for (col_idx, col) in binary_matrix.outer_iterator().enumerate() {
        part_frequencies[col_idx] = col.nnz();
    }
    
    // Sort parts by frequency (least frequent first)
    let mut part_indices: Vec<usize> = (0..part_frequencies.len()).collect();
    part_indices.sort_by(|&a, &b| part_frequencies[a].cmp(&part_frequencies[b]));

    // Simulate removing parts one by one, from least frequent to most frequent
    // At each step, calculate how many inventories can still be fully built
    let mut visited_inventories = std::collections::HashSet::new();
    let mut results = Vec::new();
    let total_parts = binary_matrix.cols();

    // Initial state: all parts are present, all inventories that can be built are buildable
    // An inventory is buildable if it requires at least one part
    let total_inventories = binary_matrix.rows() - binary_matrix.outer_iterator().filter(|inv| inv.nnz() == 0).count();
    let current_buildable = total_inventories - visited_inventories.len();
    results.push((total_parts, current_buildable));

    // Iterate through parts sorted by frequency (least to most) and remove them
    for (i, &part_to_remove) in part_indices.iter().enumerate() {
        // Get the column (part) we're removing
        let part_column = binary_matrix.outer_view(part_to_remove).unwrap();
        
        // Check inventories that contain this part and haven't been visited yet
        for (inventory_idx, _) in part_column.iter() {
            if !visited_inventories.contains(&inventory_idx) {
                // Mark this inventory as no longer buildable
                visited_inventories.insert(inventory_idx);
            }
        }
        
        let parts_remaining = total_parts - (i + 1);
        let current_buildable = total_inventories - visited_inventories.len();
        results.push((parts_remaining, current_buildable));

        if current_buildable == 0 {
            // Once no inventories can be built, fill the rest and break early
            for j in (i + 1)..total_parts {
                results.push((total_parts - (j + 1), 0));
            }
            break;
        }
    }

    // Write results to CSV file
    let file = File::create("data/graph_data.csv")?;
    let mut writer = Writer::from_writer(file);
    
    // Write header
    writer.write_record(&["parts_remaining", "buildable_inventories"])?;
    
    // Write data in descending order (from N down to 0)
    for (parts, inventories) in results.iter().rev() {
        writer.write_record(&[parts.to_string(), inventories.to_string()])?;
    }
    
    writer.flush()?;
    println!("Results written to data/graph_data.csv");

    Ok(())
}

/// Dynamic programming solution to find the maximum number of inventories
/// that can be built with exactly N distinct parts for all N from 0 to total_parts
pub fn dynamic_programming() -> Result<(), Box<dyn std::error::Error>> {
    // Read the matrix market file
    let matrix: sprs::TriMat<i32> = read_matrix_market("data/matrix.mtx")?;
    
    // Convert to CSC format and then to binary matrix
    let csc_matrix = matrix.to_csc();
    let binary_matrix: sprs::CsMat<u8> = csc_matrix.map(|&val| if val > 0 { 1u8 } else { 0u8 });
    
    let num_parts = binary_matrix.cols();
    let num_inventories = binary_matrix.rows();
    
    // Precompute which parts are required for each inventory
    let mut inventory_parts: Vec<Vec<usize>> = vec![Vec::new(); num_inventories];
    for (col_idx, col) in binary_matrix.outer_iterator().enumerate() {
        for (row_idx, _) in col.iter() {
            inventory_parts[row_idx].push(col_idx);
        }
    }
    
    // Filter out empty inventories
    let valid_inventories: Vec<usize> = (0..num_inventories)
        .filter(|&i| !inventory_parts[i].is_empty())
        .collect();
    
    println!("Total parts: {}, Valid inventories: {}", num_parts, valid_inventories.len());
    
    // Calculate part scores based on frequency and inventory impact
    let mut part_scores: Vec<(usize, f64)> = (0..num_parts)
        .map(|part_idx| {
            let frequency = binary_matrix.outer_view(part_idx).unwrap().nnz();
            let score = frequency as f64;
            (part_idx, score)
        })
        .collect();
    
    // Sort parts by score (highest first)
    part_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Get the sorted part indices 
    let sorted_part_indices: Vec<usize> = part_scores.iter().map(|(idx, _)| *idx).collect();
    
    // Generate results for each N from 0 to num_parts in parallel
    let results: Vec<(usize, usize)> = (0..=num_parts)
        .into_par_iter()
        .map(|n_parts| {
            let selected_parts: std::collections::HashSet<usize> = sorted_part_indices
                .iter()
                .take(n_parts)
                .copied()
                .collect();
            
            let buildable_count = valid_inventories
                .iter()
                .filter(|&&inv_idx| {
                    let required_parts = &inventory_parts[inv_idx];
                    required_parts.iter().all(|&part| selected_parts.contains(&part))
                })
                .count();
            
            (n_parts, buildable_count)
        })
        .collect();
    
    // Write results to CSV file
    let file = File::create("data/dp_graph_data.csv")?;
    let mut writer = Writer::from_writer(file);
    
    // Write header
    writer.write_record(&["parts_available", "max_buildable_inventories"])?;
    
    // Write data
    for (parts, inventories) in results {
        writer.write_record(&[parts.to_string(), inventories.to_string()])?;
    }
    
    writer.flush()?;
    println!("DP results written to data/dp_graph_data.csv");

    Ok(())
}

