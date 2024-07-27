use colored::{ColoredString, Colorize};
use rust_indexer::{index::Index, text_scraping::{self}};

#[tokio_macros::main]
async fn main() {
    print_with_color("Indexing...".cyan());
    let index = rust_indexer::index::parallel_index_directory("D:\\Repos\\Roslyn\\src").await;
    print_with_color("Done!".green());

    loop {
        println!();
        let query = prompt_for_input("Search");

        let matching_files = get_matching_files(&index, &query);

        scrape_and_format_matches(&matching_files, &query).await;
    }
}

fn prompt_for_input(prompt: &str) -> String {
    println!("{} >", prompt.cyan());

    let mut buffer = String::new();
    let stdin = std::io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut buffer).unwrap();

    buffer
}

fn get_matching_files(index: &Index, query: &str) -> Vec<String> {
    let matches = index.search_files(&query.trim());
    let mut ordered_matches: Vec<String> = Vec::from_iter(matches);
    ordered_matches.sort();

    let files_matched_percentage = (ordered_matches.len() as f32 / index.files_count() as f32) * 100f32;
    println!(
        "Matched {} files ({}%)",
        ordered_matches.len(),
        files_matched_percentage);

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
