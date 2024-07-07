use crate::{bloom::BloomFilter, compression_utils::lowercase_alphanumeric_only};
use crate::trigram::Trigram;
use std::collections::HashMap;
use std::{collections::HashSet, path::Path};

const BLOOM_FILTER_SIZE: usize = 714;

pub fn index_directory(path: &str, track_repo_stats: bool) -> Index {
    let mut index = Index::new(track_repo_stats);

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
    trigram_counts: Option<HashMap<Trigram, usize>>,
    files: Vec<File>
}

impl Index {
    fn new(track_repo_stats: bool) -> Index {
        Index {
            trigram_counts: if track_repo_stats { Some(HashMap::new()) } else { None },
            files: Vec::new()
        }
    }

    pub fn add_file(&mut self, file_path: &str) {
        if let Ok(file_text) = std::fs::read_to_string(Path::new(file_path)) {
            let trigrams = Trigram::from_str(&&lowercase_alphanumeric_only(&file_text));

            // If stats tracking is enabled, record how often we see each trigram.
            Self::record_trigram_counts_if_needed(&mut self.trigram_counts, &trigrams);

            let u32s: Vec<u32> = trigrams
                .iter()
                .map(|t| t.to_u32())
                .collect();
    
            let bloom_filter = BloomFilter::new(&u32s, BLOOM_FILTER_SIZE);
    
            self.files.push(File {
                file_path: file_path.to_string(),
                bloom_filter,
            })
        } else {
            // TODO
        }
    }

    pub fn files_count(&self) -> usize {
        self.files.len()
    }

    pub fn trigram_stats(&self) -> Option<Vec<(Trigram, usize)>> {
        if let Some(trigram_counts_present) = &self.trigram_counts {
            let mut trigrams: Vec<(Trigram, usize)> = Vec::from_iter(
                trigram_counts_present.iter().map(|pair| (pair.0.clone(), *pair.1)));

            // Sort in descending order by count.
            trigrams.sort_by(|a, b| a.1.cmp(&b.1));

            Some(trigrams)
        } else {
            None
        }
    }

    pub fn search_files(&self, query: &str) -> HashSet<String> {
        let query_trigrams = Trigram::from_str(&lowercase_alphanumeric_only(query));

        let u32s: Vec<u32> = query_trigrams
            .iter()
            .map(|t| t.to_u32())
            .collect();
        
        let query_bloom_filter = BloomFilter::new(&u32s, BLOOM_FILTER_SIZE);

        let mut results = HashSet::new();

        for file in &self.files {
            if file.bloom_filter.possibly_contains(&query_bloom_filter) {
                results.insert(file.file_path.clone());
            }
        }

        results
    }

    fn record_trigram_counts_if_needed(trigram_counts: &mut Option<HashMap<Trigram, usize>>, trigrams: &[Trigram]) {
        if let Some(trigram_counts_present) = trigram_counts {
            for trigram in trigrams {

                let updated_count = if let Some(this_trigrams_counts) = trigram_counts_present.get(trigram) {
                    *this_trigrams_counts + 1
                } else {
                    1usize
                };
    
                trigram_counts_present.insert(trigram.clone(), updated_count);
            }
        }
    }
}

struct File {
    file_path: String,
    bloom_filter: BloomFilter,
}
