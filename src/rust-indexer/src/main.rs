use rust_indexer::{index::Index, text_scraping};
use std::str::FromStr;

const MAX_STATS_ROWS: usize = 100;

#[tokio_macros::main]
async fn main() {
    let index = rust_indexer::index::parallel_index_directory("D:\\Repos\\Roslyn\\src").await;

    print_stats(&index);

    loop {
        println!();
        println!();
        println!();
        println!();

        println!("--- ");
        let mut buffer = String::new();
        let stdin = std::io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).unwrap();

        let matches = index.search_files(&buffer.trim());
        let mut ordered_matches: Vec<String> = Vec::from_iter(matches);
        ordered_matches.sort();

        for result_file in &ordered_matches {
            println!("{}", result_file);
        }

        let files_matched_percentage = (ordered_matches.len() as f32 / index.files_count() as f32) * 100f32;
        println!(
            "Matched {} files ({}%)",
            ordered_matches.len(),
            files_matched_percentage);

        let scrapings = text_scraping::parallel_scrape_files(&ordered_matches, &buffer.trim()).await;
        for scraped_match in scrapings {
            println!("In '{}'...", scraped_match.file_path);
            println!("{}", scraped_match.text);
            println!();
        }
    }
}

fn print_stats(index: &Index) {
    if let Some(stats) = index.trigram_stats() {
        println!("Trigrams statistics:");
        println!("- Total trigrams: {}", stats.len());
        println!("- Min occurrences: {}", stats.iter().min_by_key(|trigram|trigram.1).unwrap().1);
        println!("- Max occurrences: {}", stats.iter().max_by_key(|trigram|trigram.1).unwrap().1);
        println!("- Average occurrences: {}", stats.iter().map(|trigram|trigram.1).sum::<usize>() as f32 / stats.len() as f32);
        println!();
        println!("- Top trigrams:");

        for trigram in stats.iter().take(MAX_STATS_ROWS) {
            let trigram_text: String = if let Ok(text) = trigram.0.to_string() {
                text
            } else {
                String::from_str("Error decoding trigram").unwrap()
            };

            println!("  - {} - {}", trigram_text, trigram.1);
        }
    }
}