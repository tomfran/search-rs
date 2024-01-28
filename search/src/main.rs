use indicatif::HumanDuration;
use search::engine::{Engine, QueryResult};
use std::cmp::min;
use std::env;
use std::io::{self, Write};
use std::process::{exit, Command};
use std::time::{Duration, Instant};

const NUM_TOP_RESULTS: usize = 10;
const NUM_RESULTS: usize = 100;

fn print_results(result: &QueryResult) {
    println!("Search tokens: {:?}", result.tokens);

    if result.documents.is_empty() {
        println!("\nNo documents found\n");
        return;
    }

    println!(
        "\nTop {} results:\n",
        min(result.documents.len(), NUM_TOP_RESULTS)
    );

    for (i, doc) in result.documents.iter().take(NUM_TOP_RESULTS).enumerate() {
        println!("{:2}. score: {:>5.3}, path: {}", i + 1, doc.score, doc.path);
    }

    println!(
        "\nFetched {} documents in {} ms\n",
        result.documents.len(),
        result.time_ms,
    );
}

fn read_line(prompt: &str) -> String {
    print!("{prompt}");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn clear_terminal() {
    if cfg!(unix) {
        let _ = Command::new("clear").status();
    } else if cfg!(windows) {
        let _ = Command::new("cmd").arg("/c").arg("cls").status();
    }
}

fn main() {
    clear_terminal();

    println!("\x1B[1mSearch-rs\x1B[0m\n");
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 || args.len() > 5 {
        println!("Usage: cargo run -r <base_path> <load_or_build> <min_freq (integer)> <max_frequency_perc (float)>
        \nExample:
        \n\t- cargo run -r path/to/docs build 10 0.90
        \n\t- cargo run -r path/to/docs load");
        return;
    }

    let base_path = &args[1];
    let action = &args[2];
    let build_index = action == "build";

    let index_path = format!("{base_path}/index/idx");
    let docs_path = format!("{base_path}/docs");

    if build_index {
        let min_freq: Result<u32, _> = args[3].parse();
        let min_freq = match min_freq {
            Ok(value) => value,
            Err(_) => {
                println!("Error: min_freq must be an integer.");
                return;
            }
        };

        let max_frequency_perc: Result<f64, _> = args[4].parse();
        let max_frequency_perc = match max_frequency_perc {
            Ok(value) => value,
            Err(_) => {
                println!("Error: max_frequency_perc must be a float.");
                return;
            }
        };

        println!("Start build on directory [{docs_path}]\n");

        let start_time = Instant::now();
        Engine::build_engine(&docs_path, &index_path, max_frequency_perc, min_freq);
        let elapsed_time = start_time.elapsed();

        println!(
            "Index built in {}",
            HumanDuration(Duration::from_secs(elapsed_time.as_secs()))
        );

        exit(0);
    }

    let mut e = Engine::load_index(&index_path);

    println!(
        "Loaded search engine for directory: [{base_path}]\n\nWrite a query and press enter.\n"
    );

    loop {
        let query = read_line("> ");

        let result = e.query(&query, NUM_RESULTS);

        print_results(&result);
    }
}
