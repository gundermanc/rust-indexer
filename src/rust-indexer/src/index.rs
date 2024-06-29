use crate::bloom::BloomFilter;
use crate::trigram::Trigram;
use std::{collections::{HashMap, HashSet}, path::Path};

pub fn index_directory(path: &str) -> Index {
    let mut index = Index::new();

    index_directory_recursive(&mut index, path);

    index
}

fn index_directory_recursive(index: &mut Index, path: &str) {
    let files = std::fs::read_dir(path).unwrap();

    for file in files {
        let unwrapped_file = file.unwrap();
        
        if unwrapped_file.file_type().unwrap().is_file() {
            println!("Indexing {}...", unwrapped_file.path().display());
    
            index.add_file(unwrapped_file.path().to_str().unwrap());
        } else if unwrapped_file.file_type().unwrap().is_dir() {
            index_directory_recursive(index, unwrapped_file.path().to_str().unwrap());
        }
    }
}

pub struct Index {
    files: Vec<File>
}

impl Index {
    pub fn new() -> Index {
        Index {
            files: Vec::new()
        }
    }

    pub fn add_file(&mut self, file_path: &str) {
        if let Ok(file_text) = std::fs::read_to_string(Path::new(file_path)) {
            let trigrams = Trigram::from_str(&file_text);

            let u32s: Vec<u32> = trigrams
                .iter()
                .map(|t| t.to_u32())
                .collect();
    
            let bloom_filter = BloomFilter::new(&u32s, 2_000);
    
            self.files.push(File {
                file_path: file_path.to_string(),
                bloom_filter,
            })
        } else {
            // TODO
        }
    }

    pub fn search_files(&self, query: &str) -> HashSet<String> {
        let query_trigrams = Trigram::from_str(query);

        let u32s: Vec<u32> = query_trigrams
            .iter()
            .map(|t| t.to_u32())
            .collect();
        
        let query_bloom_filter = BloomFilter::new(&u32s, 2_000);

        let mut results = HashSet::new();

        for file in &self.files {
            if file.bloom_filter.possibly_contains(&query_bloom_filter) {
                results.insert(file.file_path.clone());
            }
        }

        results
    }
}

struct File {
    file_path: String,
    bloom_filter: BloomFilter,
}
