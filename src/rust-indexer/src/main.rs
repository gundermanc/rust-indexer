fn main() {
    let index = rust_indexer::index::index_directory("D:\\Repos\\Roslyn\\src");

    loop {
        println!();
        println!();
        println!();
        println!();

        println!("--- ");
        let mut buffer = String::new();
        let stdin = std::io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).unwrap();

        let matches = index.search_files(&buffer);

        for result_file in &matches {
            println!("{}", result_file);
        }

        println!("Matched {} files", matches.len());
    }
}
