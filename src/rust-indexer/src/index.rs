use tokio::task::JoinSet;

use crate::parallel::batch_items_by_cpu_count;
use crate::{bloom::BloomFilter, compression_utils::lowercase_alphanumeric_only};
use crate::trigram::Trigram;
use std::collections::HashMap;
use std::{collections::HashSet, path::Path};

const BLOOM_FILTER_SIZE: usize = 714;

pub async fn parallel_index_directory(path: &str) -> Index {
    let files = enumerate_directory(path);

    let mut set = JoinSet::new();

    for batch in batch_items_by_cpu_count(&files) {
        set.spawn(
            tokio::spawn(
                async move {
                    Vec::from_iter(batch.iter().map(|file| bloom_index_file(&file)))
                }));
    }

    let mut all_matches = Vec::new();

    while let Some(res) = set.join_next().await {
        for item in res.unwrap().iter().flat_map(|item| { item }) {
            if let Ok(ok_item) = item {
                all_matches.push(ok_item.clone());
            }
        }
    }

    let mut index = Index::new(false);

    for item in all_matches {
        index.add_file(item)
    }

    index
}

fn enumerate_directory(path: &str) -> Vec<String> {
    let mut file_paths = Vec::new();
    let files = std::fs::read_dir(path).unwrap();

    for file in files {
        let unwrapped_file = file.unwrap();

        if unwrapped_file.file_type().unwrap().is_file() {
            file_paths.push(unwrapped_file.path().to_str().unwrap().to_string());
        } else if unwrapped_file.file_type().unwrap().is_dir() {
            let file_path_buffer = unwrapped_file.path();
            let file_path = file_path_buffer.to_str().unwrap();

            // Exclude the dot git folder in repos.
            if !file_path.ends_with(".git") {
                for path in enumerate_directory(unwrapped_file.path().to_str().unwrap()) {
                    file_paths.push(path.to_string());
                }
            }
        }
    }

    file_paths
}

fn bloom_index_file(file_path: &str) -> Result<File, std::io::Error> {

    let file_text = std::fs::read_to_string(Path::new(file_path))?;

    let trigrams = Trigram::from_str(&&lowercase_alphanumeric_only(&file_text));

    let u32s: Vec<u32> = trigrams
        .iter()
        .map(|t| t.to_u32())
        .collect();

    let bloom_filter = BloomFilter::new(&u32s, BLOOM_FILTER_SIZE);

    Ok(
        File {
            file_path: file_path.to_string(),
            bloom_filter,
        }
    )
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

    pub fn add_file(&mut self, file: File) {
        self.files.push(file);
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
}

#[derive(Clone)]
struct File {
    file_path: String,
    bloom_filter: BloomFilter,
}
