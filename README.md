# rust-indexer

## Why
This is a rapid-prototype/learning-hackathon project to help me gain familiarity with Rust. It is not maintained or intended for actual use.

## What
It is a trigram indexer that creates an in-memory index of a source code repository that can then be 'searched'. Search returns a pruned list of candidate files. It is conceptually similar to https://github.com/gundermanc/codeindex but much more primitive and without the ability to serialize and read the index from a file.

## TBD
- Well-formed main method and ability to take command line parameters.
- Serialization
- Sharding and merging
- Tests
- Ingestion of trigrams that cross a buffer seam

## Building
- `cargo build`
