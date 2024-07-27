# rust-indexer
(C) 2024 Christian Gunderman

## Vision
This is a trigram + bloom filter based text indexer that creates a persistent index of a folder containing source code that can then be 'searched'. The north star scenario is:
- User works in a large enterprise setting with dozens of repos and hundreds of thousands of code files.
- User works in a blended IDE and command line workflow.
- User wants to be able to quickly look across the set of all of their codebases for various types of code matches. Note that this is quite a bit different than your favorite IDE's text search in that it spans a much much larger set of data.
- User wants to be able to incrementally update the index, on demand.
- User may want advanced shell integrations between search and their command processor.

This app is the next iteration in a set of side projects of mine that started with https://github.com/gundermanc/codeindex.

## Current Progress
- Trigram scraping.
- Basic parallel indexing.
- Basic serialization of the index to disk and reloading.
- Basic command line app for building the index and searching using an existing index.

## Usage
- rust-indexer index [path] - creates a new index for the specified folder.
- rust-indexer search [path] [query] - searches a pre-existing index for the specified term, then scrapes matches from the file.

## Next Steps
- Code cleanup (delete the unwraps etc.)
- Incremental reindexing
- Better match formatting and customization of the output.
- Maybe a 'daemon' mode where the user can drop into a search session in their terminal, ask something, then drop back to their shell.
- Other types of matches -- fuzzy, string distance, structured search and syntax awareness.
- Syntax highlighting

## Building
- `cargo build --release` -- debug config is much much slower.
