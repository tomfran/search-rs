use indicatif::HumanDuration;
use search::index::Index;
use search::query::{DocumentResult, QueryProcessor};
use std::cmp::min;
use std::env;
use std::io::{self, Write};
use std::process::{exit, Command};
use std::time::{Duration, Instant};

const NUM_TOP_RESULTS: usize = 10;
const NUM_RESULTS: usize = 1_000_000;

fn print_results(results: &[DocumentResult], elapsed_time: Duration) {
    if results.is_empty() {
        println!("\nNo documents found\n");
        return;
    }

    println!("\nTop {} results:\n", min(results.len(), NUM_TOP_RESULTS));

    for (i, doc) in results.iter().take(NUM_TOP_RESULTS).enumerate() {
        println!("{:2}. score: {:>5.3}, path: {}", i + 1, doc.score, doc.path);
    }

    println!(
        "\nFetched {} documents in {} ms\n",
        results.len(),
        elapsed_time.as_millis()
    );
}

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
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
        println!("Usage: cargo run --bin search <base_path> <load_or_build> [build_num_threads]");
        return;
    }

    let base_path = &args[1];
    let action = &args[2];
    let build_index = action == "build";

    let index_path = format!("{}/index/index", base_path);
    let tokenizer_path = format!("{}/tokenizer/roberta-large", base_path);
    let docs_path = format!("{}/docs", base_path);

    if build_index {
        println!("Start build on directory [{}]\n", docs_path);

        let num_threads = args.get(3).map_or(0, |s| s.parse().unwrap_or(0));

        if num_threads != 0 {
            println!("Setting thread number to {}", num_threads);

            rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build_global()
                .unwrap();
        }

        let start_time = Instant::now();

        Index::build_index(&docs_path, &index_path, &tokenizer_path);
        let elapsed_time = start_time.elapsed();
        println!(
            "Index built in {}.\n\nLoad options:\n- CLI: cargo run --release --bin search {} load",
            HumanDuration(Duration::from_secs(elapsed_time.as_secs())),
            base_path
        );

        exit(0);
    }

    let mut q = QueryProcessor::build_query_processor(&index_path, &tokenizer_path);

    println!(
        "Loaded search engine for directory: [{}]\n\nWrite a query and press enter.\n",
        base_path
    );

    loop {
        let query = read_line("> ");

        let start_time = Instant::now();
        let results = q.query(&query, NUM_RESULTS);
        let elapsed_time = start_time.elapsed();

        print_results(&results, elapsed_time);
    }
}
