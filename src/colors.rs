use csv::Reader;
use palette::{Srgb, Lab, IntoColor};
use palette::white_point::D65;
use palette::color_difference::Ciede2000;
use serde::{Deserialize, Deserializer};
use std::fs::{File, create_dir_all};
use std::error::Error;
use std::io::Write;
use std::path::Path;
use sprs::{TriMat, CsMat};
use sprs::io::{write_matrix_market_sym, SymmetryMode};

fn deserialize_hex_color<'de, D>(deserializer: D) -> Result<Srgb<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let hex = String::deserialize(deserializer)?;
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(serde::de::Error::custom)?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(serde::de::Error::custom)?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(serde::de::Error::custom)?;
    Ok(Srgb::new(r, g, b))
}

#[derive(Debug, Deserialize)]
struct ColorItem {
    id: i32,
    #[serde(deserialize_with = "deserialize_hex_color")]
    rgb: Srgb<u8>,
}

fn read_colors_from_csv(file_path: &str) -> Result<Vec<ColorItem>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = Reader::from_reader(file);
    let items = reader.deserialize().collect::<Result<Vec<_>, _>>()?;
    Ok(items)
}

fn compute_distance_matrix(colors: &[ColorItem]) -> Vec<Vec<f32>> {
    let lab_colors: Vec<Lab<D65, f32>> = colors.iter()
        .map(|item| item.rgb.into_format::<f32>().into_color())
        .collect();

    let mut matrix = vec![vec![0.0; colors.len()]; colors.len()];
    
    for (i, color1) in lab_colors.iter().enumerate() {
        for (j, color2) in lab_colors.iter().enumerate() {
            matrix[i][j] = color1.difference(*color2);
        }
    }

    matrix
}

fn write_distance_matrix_to_mm(matrix: &[Vec<f32>], output_path: &str) -> Result<(), Box<dyn Error>> {
    let n = matrix.len();
    let mut triplets = TriMat::new((n, n));
    
    // Add all non-zero entries to the triplet matrix
    for (i, row) in matrix.iter().enumerate() {
        for (j, &value) in row.iter().enumerate() {
            if value != 0.0 {
                triplets.add_triplet(i, j, value);
            }
        }
    }
    
    let sparse_matrix: CsMat<f32> = triplets.to_csr();
    
    write_matrix_market_sym(output_path, &sparse_matrix, SymmetryMode::Symmetric)?;
    
    Ok(())
}

fn write_color_id_mapping(colors: &[ColorItem], output_path: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(output_path)?;
    writeln!(file, "matrix_index,color_id")?;
    
    for (index, color) in colors.iter().enumerate() {
        writeln!(file, "{},{}", index, color.id)?;
    }
    
    Ok(())
}


pub fn main_with_output_dir(output_dir: &str) -> Result<(), Box<dyn Error>> {
    // Create output directory if it doesn't exist
    create_dir_all(output_dir)?;
    println!("Output directory: {}", output_dir);
    
    let colors = read_colors_from_csv("data/colors.csv")?;
    let distance_matrix = compute_distance_matrix(&colors);

    let distance_matrix_path = Path::new(output_dir).join("color_distance_matrix.mtx");
    write_distance_matrix_to_mm(&distance_matrix, distance_matrix_path.to_str().unwrap())?;
    println!("Distance matrix written to {}", distance_matrix_path.display());

    let color_mapping_path = Path::new(output_dir).join("color_id_mapping.csv");
    write_color_id_mapping(&colors, color_mapping_path.to_str().unwrap())?;
    println!("Color ID mapping written to {}", color_mapping_path.display());

    Ok(())
}
