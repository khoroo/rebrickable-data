use std::process;
use clap::{Parser, Subcommand};

mod colors;
mod inventory_parts;
mod knapsack;

#[derive(Parser)]
#[command(name = "lego")]
#[command(about = "LEGO Data Processor", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Process colors.csv and compute distance matrix
    Colors {
        /// Output directory (default: 'data')
        #[arg(short, long, default_value = "data")]
        output_dir: String,
    },
    /// Process inventory_parts.csv and create sparse matrix
    Inventory {
        /// Output directory (default: 'data')
        #[arg(short, long, default_value = "data")]
        output_dir: String,
    },
    /// Run greedy knapsack approximation
    Knapsack,
    /// Run dynamic programming solution for all N parts
    Dp,
}

fn run_colors(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running colors processor...");
    colors::main_with_output_dir(output_dir)
}

fn run_inventory(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running inventory processor...");
    inventory_parts::main_with_output_dir(output_dir)
}

fn run_knapsack() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running greedy knapsack approximation...");
    knapsack::greedy()
}

fn run_dp() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running dynamic programming solution for all N parts...");
    knapsack::dynamic_programming()
}

fn main() {
    let cli = Cli::parse();
    
    let result = match cli.command {
        Commands::Colors { output_dir } => run_colors(&output_dir),
        Commands::Inventory { output_dir } => run_inventory(&output_dir),
        Commands::Knapsack => run_knapsack(),
        Commands::Dp => run_dp(),
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}