use crate::batching::{batch_items, batch_items_by_cpu_count};
use crate::{bloom::BloomFilter, compression_utils::lowercase_alphanumeric_only};
use crate::trigram::Trigram;
use rmp_serde::Serializer;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::io::{Read, Write};
use std::usize;
use std::{collections::HashSet, path::Path};
use std::fs::File;
use tokio::task::JoinSet;

const BLOOM_FILTER_SIZE: usize = 714;
const CHILDREN_PER_NODE: usize = 2;

#[derive(Clone, Serialize, Deserialize)]
pub struct IndexTree {
    child_indexes: Vec<LazyIndex>,
    child_nodes: Vec<IndexTree>,
    bloom_filter: BloomFilter,
    files_count: usize,
}

impl IndexTree {
    pub fn from_index(index: &Index, output_path: &str) -> IndexTree {
        // Create a new mini index from each batch.
        let batches: Vec<Index> = batch_items(&index.files, index.files_count() / CHILDREN_PER_NODE)
            .into_iter()
            .map(|batch| Index { files: batch })
            .collect();

        let mut nodes: Vec<IndexTree> = batch_items(&batches, batches.len() / CHILDREN_PER_NODE)
            .into_iter()
            .map(|batch| Self::from_nodes(&batch, &[], output_path))
            .collect();

        while nodes.len() > CHILDREN_PER_NODE {
            nodes = batch_items(&nodes, nodes.len() / CHILDREN_PER_NODE)
                .into_iter()
                .map(|batch| Self::from_nodes(&[], &batch, output_path))
                .collect();
        }

        IndexTree::from_nodes(&[], &nodes, output_path)
    }

    pub fn from_nodes(child_indexes: &[Index], nodes: &[IndexTree], output_path: &str) -> IndexTree {

        // Get the bloom filters from the child index.
        let index_filters = child_indexes
            .iter()
            .flat_map(|index| index.files.clone())
            .map(|file|file.bloom_filter);

        // Get the bloom filters from the child nodes.
        let child_filters = nodes.iter().map(|node| node.bloom_filter.clone());

        let combined: Vec<BloomFilter> = child_filters
            .chain(index_filters)
            .collect();

        // Serialize the in-memory indexes to a lazy form that can be loaded
        // piecemeal on demand.
        let lazy_indexes: Vec<LazyIndex> = child_indexes
            .iter()
            .map(|index| LazyIndex::from_index(index, output_path))
            .collect();

        let files_count = nodes
            .iter().map(|node| node.files_count).sum::<usize>() +
            child_indexes.iter().map(|index|index.files_count()).sum::<usize>();

        IndexTree {
            child_indexes: lazy_indexes,
            child_nodes: Vec::from(nodes),
            bloom_filter: BloomFilter::from_filters(&combined),
            files_count
        }
    }

    pub fn from_file(path: &str) -> IndexTree {
        let mut buf = Vec::new();
        let mut index_read = File::open(path).unwrap();
        index_read.read_to_end(&mut buf).unwrap();

        rmp_serde::from_slice(&buf).unwrap()
    }

    pub fn save(&self, path: &str) {
        let mut buf = Vec::new();
        self.serialize(&mut Serializer::new(&mut buf)).unwrap();
    
        let mut file = File::create(path).unwrap();
        file.write_all(&buf).unwrap();
    }

    pub fn search_files(&self, query: &str) -> (HashSet<String>, usize) {
        let mut files = HashSet::new();

        let trigrams = Trigram::from_str(&&lowercase_alphanumeric_only(&query));

        let u32s: Vec<u32> = trigrams
            .iter()
            .map(|t| t.to_u32())
            .collect();
    
        let query_bloom_filter = BloomFilter::new(&u32s, BLOOM_FILTER_SIZE);

        let bloom_filters_checked = Self::search_node_for_files(&query_bloom_filter, &mut files, self);

        (files, bloom_filters_checked)
    }

    pub fn files_count(&self) -> usize {
        self.files_count
    }

    fn search_node_for_files(query: &BloomFilter, files: &mut HashSet<String>, node: &IndexTree) -> usize {
        let mut bloom_filters_checked = 0;

        // Check if the merged bloom filter is a match. If so, there may be relevant children.
        if !node.bloom_filter.possibly_contains(query) {
            bloom_filters_checked += 1;
            return bloom_filters_checked;
        }

        // Search relevant child nodes.
        for child_node in &node.child_nodes {
            bloom_filters_checked += Self::search_node_for_files(query, files, child_node);
        }

        // Search any direct children.
        for index in &node.child_indexes {
            for file in &index.get().files {
                bloom_filters_checked += 1;

                if file.bloom_filter.possibly_contains(query) {
                    files.insert(file.file_path.clone());
                }
            }
        }

        bloom_filters_checked
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LazyIndex {
    file_name: String,
}

impl LazyIndex {
    pub fn from_file(path: &str) -> LazyIndex {
        LazyIndex {
            file_name: path.to_string(),
        }
    }

    pub fn from_index(index: &Index, output_path: &str) -> LazyIndex {
        let file_name = format!("{}/{}", output_path, Uuid::new_v4());
        index.save(&file_name);

        LazyIndex {
            file_name,
        }
    }

    pub fn get(&self) -> Index {
        Index::from_file(&self.file_name)
    }
}

pub async fn parallel_index_directory(path: &str) -> Index {
    let files = enumerate_directory(path);

    let mut set = JoinSet::new();

    for batch in batch_items_by_cpu_count(&files) {
        set.spawn(
            async move {
                Vec::from_iter(batch.iter().map(|file| bloom_index_file(&file)))
            });
    }

    let mut all_matches = Vec::new();

    while let Some(res) = set.join_next().await {
        for item in res.unwrap() {
            if let Ok(ok_item) = item {
                all_matches.push(ok_item);
            }
        }
    }

    let mut index = Index::new();

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

fn bloom_index_file(file_path: &str) -> Result<FileEntry, std::io::Error> {

    let file_text = std::fs::read_to_string(Path::new(file_path))?;

    let trigrams = Trigram::from_str(&&lowercase_alphanumeric_only(&file_text));

    let u32s: Vec<u32> = trigrams
        .iter()
        .map(|t| t.to_u32())
        .collect();

    let bloom_filter = BloomFilter::new(&u32s, BLOOM_FILTER_SIZE);

    Ok(
        FileEntry {
            file_path: file_path.to_string(),
            bloom_filter,
        }
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Index {
    files: Vec<FileEntry>
}

impl Index {
    pub fn new() -> Index {
        Index {
            files: Vec::new()
        }
    }

    pub fn from_file(path: &str) -> Index {
        let mut buf = Vec::new();
        let mut index_read = File::open(path).unwrap();
        index_read.read_to_end(&mut buf).unwrap();

        rmp_serde::from_slice(&buf).unwrap()
    }

    pub fn add_file(&mut self, file: FileEntry) {
        self.files.push(file);
    }

    pub fn files_count(&self) -> usize {
        self.files.len()
    }

    pub fn save(&self, path: &str) {
        let mut buf = Vec::new();
        self.serialize(&mut Serializer::new(&mut buf)).unwrap();
    
        let mut file = File::create(path).unwrap();
        file.write_all(&buf).unwrap();
    }

    pub async fn search_files(&self, query: &str) -> HashSet<String> {
        let query_trigrams = Trigram::from_str(&lowercase_alphanumeric_only(query));

        let u32s: Vec<u32> = query_trigrams
            .iter()
            .map(|t| t.to_u32())
            .collect();
        
        let query_bloom_filter = BloomFilter::new(&u32s, BLOOM_FILTER_SIZE);

        let mut set = JoinSet::new();

        for batch in batch_items_by_cpu_count(&self.files) {
            let task_bloom_filter = query_bloom_filter.clone();
            set.spawn(
                async move {
                    Vec::from_iter(batch
                        .iter()
                        .map(|file| file.clone())
                        .filter(|file| file.bloom_filter.possibly_contains(&task_bloom_filter)))
                });
        }
        
        let mut all_matches: HashSet<String> = HashSet::new();
    
        while let Some(res) = set.join_next().await {
            for item in res.unwrap() {
                all_matches.insert(item.file_path.clone());
            }
        }
    
        all_matches
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct FileEntry {
    file_path: String,
    bloom_filter: BloomFilter,
}
