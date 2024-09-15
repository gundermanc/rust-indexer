use colored::{ColoredString, Colorize};
use rust_indexer::{index::IndexTree, text_scraping::{self}};
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

    let index_directory = format!("{}/.index", path);
    let index_root_path = format!("{}/root.dat", index_directory);

    std::fs::create_dir_all(&index_directory).unwrap();

    if command == "index" {
        print_with_color("Indexing...".cyan());
        let index = rust_indexer::index::parallel_index_directory(path).await;
        let index_tree = IndexTree::from_index(&index, &index_directory);

        print_with_color("Saving index...".cyan());
        index_tree.save(&index_root_path);

        print_with_color("Done!".green());
    } else if command == "search" {
        if cmd_args.len() != 4 {
            print_help();
            return;
        }

        let query = cmd_args.get(3).unwrap();
        let index_tree = IndexTree::from_file(&index_root_path);

        let (matching_files, comparisons) = get_matching_files(&index_tree, &query).await;

        scrape_and_format_matches(&matching_files, &query).await;

        print_perf_stats(&matching_files, &index_tree, comparisons);

    } else if command == "repl" {
        let index_tree = IndexTree::from_file(&index_root_path);

        loop {
            let query = prompt_for_input("Search >");
            let (matching_files, comparisons) = get_matching_files(&index_tree, &query).await;
    
            scrape_and_format_matches(&matching_files, &query).await;
    
            print_perf_stats(&matching_files, &index_tree, comparisons);
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

async fn get_matching_files(index: &IndexTree, query: &str) -> (Vec<String>, usize) {
    let matches = index.search_files(&query.trim());
    let mut ordered_matches: Vec<String> = Vec::from_iter(matches.0);
    ordered_matches.sort();

    (ordered_matches, matches.1)
}

async fn scrape_and_format_matches(files: &Vec<String>, query: &str) {
    let scrapings = text_scraping::parallel_scrape_files(&files, &query.trim()).await;

    for scraped_match in scrapings {
        println!("In '{}'...", scraped_match.file_path.black().on_cyan());
        println!("{}", scraped_match.text.italic().yellow());
        println!();
    }
}

fn print_perf_stats(matching_files: &Vec<String>, index: &IndexTree, comparisons: usize) {
    let files_matched_percentage = (matching_files.len() as f32 / index.files_count() as f32) * 100f32;

    // Percentage of file bloom filters checked. Not technically accurate because this a count
    // of total bloom comparisons, including non-leaf tree nodes from IndexTree, but it helps
    // us see the relative cost savings of using IndexTree during lookup to reduce the number
    // of required bloom comparisons.
    let bloom_comparisons_percentage = (comparisons as f32 / index.files_count() as f32) * 100f32;

    println!(
        "Narrowed search to {} of {} files ({}%) using {} bloom comparisons ({}%)",
        matching_files.len(),
        index.files_count(),
        files_matched_percentage,
        comparisons,
        bloom_comparisons_percentage);
}

fn print_with_color(colored_str: ColoredString) {
    println!("{}", colored_str);
}
