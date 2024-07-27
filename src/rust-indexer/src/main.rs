use rust_indexer::text_scraping;

#[tokio_macros::main]
async fn main() {
    println!("Indexing...");
    let index = rust_indexer::index::parallel_index_directory("D:\\Repos\\Roslyn\\src").await;
    println!("Done!");

    loop {
        println!();
        print!("Search >");

        let mut buffer = String::new();
        let stdin = std::io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).unwrap();

        let matches = index.search_files(&buffer.trim());
        let mut ordered_matches: Vec<String> = Vec::from_iter(matches);
        ordered_matches.sort();

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
