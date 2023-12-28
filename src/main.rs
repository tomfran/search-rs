use std::io::{self, Write};
use std::process::Command;

// use search::index::Index;
use search::query::QueryProcessor;

const NUM_RESULTS: usize = 10;

fn print_results(results: &[u32]) {
    println!("\nSearch Results:");
    for (i, doc_id) in results.iter().enumerate() {
        println!("\t- {:3}. Doc ID: {}", i + 1, doc_id);
    }
    println!();
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
    let base_path = "data/wiki-data";
    let index_path = base_path.to_string() + "/index/index";
    let tokenizer_path = base_path.to_string() + "/tokenizer/bert-base-uncased";

    // let docs_path = base_path.to_string() + "/docs";
    // Index::build_index(&docs_path, &index_path, &tokenizer_path);

    clear_terminal();

    println!(
        "Search engine for base path: [{}]\nWrite a query and press enter.\n",
        base_path
    );

    let mut q = QueryProcessor::build_query_processor(&index_path, &tokenizer_path);

    loop {
        let query = read_line("> ");

        // Perform search
        let results = q.query(&query, NUM_RESULTS);

        // Display results
        print_results(&results);
    }
}
