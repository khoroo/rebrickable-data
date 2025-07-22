use std::env;
use std::process;

mod colors;
mod inventory_parts;
mod knapsack;

fn print_usage() {
    println!("LEGO Data Processor");
    println!();
    println!("USAGE:");
    println!("    lego <COMMAND> [OUTPUT_DIR]");
    println!();
    println!("COMMANDS:");
    println!("    colors       Process colors.csv and compute distance matrix");
    println!("    inventory    Process inventory_parts.csv and create sparse matrix");
    println!("    knapsack     Run greedy knapsack approximation");
    println!("    dp           Run dynamic programming solution for all N parts");
    println!("    help         Show this help message");
    println!();
    println!("ARGUMENTS:");
    println!("    OUTPUT_DIR   Output directory (default: 'data')");
    println!();
    println!("EXAMPLES:");
    println!("    lego colors");
    println!("    lego colors output_folder");
    println!("    lego inventory");
    println!("    lego inventory output_folder");
    println!("    lego knapsack");
    println!("    lego dp");
}

fn run_colors(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running colors processor...");
    
    // Get output directory from args if provided
    let output_dir = if args.len() > 2 {
        &args[2]
    } else {
        "data"
    };
    
    colors::main_with_output_dir(output_dir)
}

fn run_inventory(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running inventory processor...");
    
    // Get output directory from args if provided
    let output_dir = if args.len() > 2 {
        &args[2]
    } else {
        "data"
    };
    
    inventory_parts::main_with_output_dir(output_dir)
}

fn run_knapsack() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running greedy knapsack approximation...");
    knapsack::greedy()
}

fn run_dp(_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running dynamic programming solution for all N parts...");
    knapsack::dynamic_programming()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    let command = &args[1];
    
    let result = match command.as_str() {
        "colors" => run_colors(&args),
        "inventory" => run_inventory(&args),
        "knapsack" => run_knapsack(),
        "dp" => run_dp(&args),
        "help" | "--help" | "-h" => {
            print_usage();
            return;
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            eprintln!();
            print_usage();
            process::exit(1);
        }
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}