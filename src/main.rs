use colored::{ColoredString, Colorize};
use rust_indexer::{index::Index, text_scraping::{self}};
use std::env::args;

#[tokio_macros::main]
async fn main() {
    let cmd_args: Vec<String> = args().collect();

    if cmd_args.len() < 2 {
        print_help();
        return;
    }

    let command = cmd_args.get(1).unwrap();
    let path = cmd_args.get(2).unwrap();

    let index_path = format!("{}.index.dat", path);

    if command == "index" {
        print_with_color("Indexing...".cyan());
        let index = rust_indexer::index::parallel_index_directory(path).await;

        print_with_color("Saving index...".cyan());
        index.save(&index_path);

        print_with_color("Done!".green());
    } else if command == "search" {
        if cmd_args.len() != 4 {
            print_help();
            return;
        }

        let query = cmd_args.get(3).unwrap();
        let index = Index::from_file(&index_path);

        let matching_files = get_matching_files(&index, &query).await;

        scrape_and_format_matches(&matching_files, &query).await;

        let files_matched_percentage = (matching_files.len() as f32 / index.files_count() as f32) * 100f32;
        println!(
            "Matched {} files ({}%)",
            matching_files.len(),
            files_matched_percentage);
    } else if command == "repl" {
        let index = Index::from_file(&index_path);

        loop {
            let query = prompt_for_input("Search >");    
            let matching_files = get_matching_files(&index, &query).await;
    
            scrape_and_format_matches(&matching_files, &query).await;
    
            let files_matched_percentage = (matching_files.len() as f32 / index.files_count() as f32) * 100f32;
            println!(
                "Matched {} files ({}%)",
                matching_files.len(),
                files_matched_percentage);
        }
    } else {
        print_help();
    }
}

fn print_help() {
    print_with_color("Rust Code Indexer".cyan());
    print_with_color("(C) 2024 Christian Gunderman".cyan());
    println!();
    print_with_color("Usage:".white());
    print_with_color("  rust-indexer [index] [path] -- reindex folder.".white());
    print_with_color("  rust-indexer [search] [path] [query] -- find matches.".white());
    print_with_color("  rust-indexer [repl] [path] -- keep alive. Potentially faster.".white());
}

fn prompt_for_input(prompt: &str) -> String {
    println!("{} >", prompt.cyan());
    let mut buffer = String::new();
    let stdin = std::io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut buffer).unwrap();
    buffer
}

async fn get_matching_files(index: &Index, query: &str) -> Vec<String> {
    let matches = index.search_files(&query.trim()).await;
    let mut ordered_matches: Vec<String> = Vec::from_iter(matches);
    ordered_matches.sort();

    ordered_matches
}

async fn scrape_and_format_matches(files: &Vec<String>, query: &str) {
    let scrapings = text_scraping::parallel_scrape_files(&files, &query.trim()).await;

    for scraped_match in scrapings {
        println!("In '{}'...", scraped_match.file_path.black().on_cyan());
        println!("{}", scraped_match.text.italic().yellow());
        println!();
    }
}

fn print_with_color(colored_str: ColoredString) {
    println!("{}", colored_str);
}
