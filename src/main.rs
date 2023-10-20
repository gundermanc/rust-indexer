mod indexer;

use async_std::io;
use glob::glob;

use crate::indexer::indexer::IndexBatch;

fn main() {

    print!("Enter index directory> \r\n");
    let mut index_dir = String::new();
    async_std::task::block_on(io::stdin().read_line(&mut index_dir));

    let batch_count = 20;

    let mut batches: Vec<Vec<String>> = Vec::new();

    for batch_num in 0..batch_count {
        batches.push(Vec::new());
    }

    let mut i = 0;

    // Break up into batches.
    for path in glob(index_dir).expect("Failed to read glob pattern") {
        match path {
            Ok(path) => {
                batches[i % batch_count].push(path.display().to_string());
            }
            Err(e) => println!("{:?}", e),
        }

        i += 1;
    }

    let mut tasks: Vec<async_std::task::JoinHandle<IndexBatch>> = Vec::new();

    // Start each batch in parallel.
    for batch in batches {
        let task = async_std::task::spawn(async {
            let mut batch_index = indexer::indexer::IndexBatch::new();

            for file in batch {
                batch_index.index_file(&file);
            }

            return batch_index;
        });

        tasks.push(task);
    }

    let results = async_std::task::block_on(futures::future::join_all(tasks));

    let merged_index = IndexBatch::merge(results);

    loop {
        print!("Enter query> \r\n");

        let mut buf = String::new();
        async_std::task::block_on(io::stdin().read_line(&mut buf));
        let candidates = merged_index.get_candidates(buf.as_str());

        print!("\r\n");

        for candidate in candidates {
            print!("{}\r\n", candidate);
        }
    }
}
