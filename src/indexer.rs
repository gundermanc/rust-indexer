pub mod indexer {
    use std::{fs::File, io::Read, collections::{HashSet, HashMap}, hash::Hash, rc::Rc};

    pub struct Index {
        files: HashSet<Rc<String>>,
        trigrams_to_files: HashMap<Trigram, HashSet<Rc<String>>>
    }

    impl Index {
        pub fn get_candidates(&self, query: &str) -> HashSet<Rc<String>> {
            let mut trigrams: Vec<Trigram> = Vec::new();

            let query_bytes = query.as_bytes();

            for i in 0..(query.len() - 3) {
                trigrams.push(Trigram::create(&query_bytes[i..]))
            }

            let mut candidate_files: HashSet<Rc<String>> = HashSet::new();

            // Add all files to candidates.
            for file in &self.files {
                candidate_files.insert(file.clone());
            }

            // Remove any files that don't match all of the trigrams.
            for trigram in trigrams {
                match self.trigrams_to_files.get(&trigram) {
                    Some (matching_files) => {
                        for file in &candidate_files.clone() {
                            if !matching_files.contains(file) {
                                candidate_files.remove(file);
                            }
                        }
                    }
                    _ => { }
                }
            }

            return candidate_files;
        }
    }

    pub struct IndexBatch {
        contained_trigrams: HashMap<String, HashSet<Trigram>>
    }

    impl IndexBatch {
        pub fn new() -> IndexBatch {
            IndexBatch { contained_trigrams: HashMap::new() }
        }

        pub fn merge(batches: Vec<IndexBatch>) -> Index {
            let mut files: HashSet<Rc<String>> = HashSet::new();
            let mut trigrams_to_files: HashMap<Trigram, HashSet<Rc<String>>> = HashMap::new();

            for batch in batches {
                for document in &batch.contained_trigrams {
                    let ref_counted_doc = Rc::new(document.0.clone());

                    files.insert(ref_counted_doc.clone());

                    for trigram in document.1 {
                        match trigrams_to_files.get_mut(&trigram) {
                            Some (value) => {
                                value.insert(ref_counted_doc.clone());
                            }

                            None => {
                                let mut set: HashSet<Rc<String>> = HashSet::new();

                                set.insert(ref_counted_doc.clone());

                                trigrams_to_files.insert(trigram.clone(), set);
                            }
                        }
                    }
                }
            }

            return Index { files: files, trigrams_to_files: trigrams_to_files };
        }

        pub fn index_file(&mut self, file_name: &str) {
            match File::open(file_name) {
                Ok (reader) => {
                    let mut trigrams: HashSet<Trigram> = HashSet::new();

                    IndexBatch::index_file_reader(&mut trigrams, reader);

                    self.contained_trigrams.insert(
                        String::from(file_name),
                        trigrams);
                }

                // TODO: log failing files.
                _ => { }
            }
        }

        fn index_file_reader(trigrams: &mut HashSet<Trigram>, mut reader: File) {
            let mut buf: [u8; 16000] = [0; 16000];

            match reader.read(&mut buf) {
                Ok (bytes_read) => IndexBatch::index_buffer(trigrams, &buf[..bytes_read]),

                // TODO: log failing files.
                _ => ()
            }
        }

        fn index_buffer(trigrams: &mut HashSet<Trigram>, mut buf: &[u8]) {
            let buffer_len = buf.len();

            if buffer_len < 3 {
                return;
            }
            
             // TODO: make this work on buffer seams.
             for i in 0..(buffer_len - 3) {
                let trigram = Trigram::create(&buf[i..i+3]);

                trigrams.insert(trigram);
            }
        }
    }

    #[derive(Clone, Eq, Hash, PartialEq)]
    pub struct Trigram {
        first: u8,
        second: u8,
        third: u8,
    }

    impl Trigram {
        fn create(buf: &[u8]) -> Trigram {
            return Trigram {
                first: char::to_lowercase(buf[0] as char).nth(0).unwrap() as u8,
                second: char::to_lowercase(buf[1] as char).nth(0).unwrap() as u8,
                third: char::to_lowercase(buf[2] as char).nth(0).unwrap() as u8,
            };
        }
    }
}
