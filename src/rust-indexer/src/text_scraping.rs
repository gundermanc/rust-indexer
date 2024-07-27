use std::str::FromStr;

use tokio::{task::JoinSet};

use crate::parallel::batch_items_by_cpu_count;

#[derive(Clone)]
pub struct Match {
    pub file_path: String,
    pub offset: usize,
    pub length: usize,
    pub text: String,
}

pub async fn parallel_scrape_files(files: &[String], query: &str) -> Vec<Match> {

    let mut set = JoinSet::new();

    let batches = batch_items_by_cpu_count(files);

    for batch in batches {
        let task_query = String::from_str(query).unwrap();

        set.spawn(
            tokio::spawn(
                async move {
                    scrape_files(&batch, &task_query)
                }));
    }

    let mut all_matches = Vec::new();

    while let Some(res) = set.join_next().await {
        for item in res.unwrap().iter().flat_map(|item| { item }) {
            all_matches.push((*item).clone());
        }
    }

    all_matches
}

pub fn scrape_files(files: &[String], query: &str) -> Vec<Match> {
    if query.len() == 0 {
        return vec![];
    }
    
    let mut matches = Vec::new();

    for file in files {
        // TODO: I'm sure that we can do this faster by doing case-insensitive comparisons
        // instead of a to_lowercase().
        let file_text = std::fs::read_to_string(file).unwrap();
        let file_text_without_bom = drop_bom(&file_text);
        let lowered_file_text = file_text_without_bom.to_lowercase();
        let lowered_query = query.to_lowercase();

        // TODO: hard coding 1 to skip BOM.
        for i in 0..file_text.len() {
            if lowered_file_text.is_char_boundary(i) &&
                lowered_file_text[i..].starts_with(&lowered_query) {
                matches.push(Match {
                    file_path: file.to_string(),
                    offset: i,
                    length: lowered_query.len(),
                    text: format_match(&file_text_without_bom, &lowered_file_text, i, lowered_query.len(), 5)
                });
            }
        }
    }

    matches
}

fn drop_bom(text: &str) -> &str {
    let bytes = text.as_bytes();

    if bytes.len() >= 3 &&
        bytes[0] == 0xef && bytes[1] == 0xbb && bytes[2] == 0xbf {
            return &text[3..];
        }

    return text;
}

fn format_match(file_text: &str, lowered_file_text: &str, offset: usize, length: usize, surrounding_lines: usize) -> String {
    let per_direction_line_budget = surrounding_lines / 2;
    let mut expanded_offset = offset;
    let mut expanded_length = length;

    // Start at the offset and iterate backwards.
    let mut lines: usize = 0;
    for c in lowered_file_text[0..offset].chars().rev().peekable() {
        if c == '\r' || c == '\n' {
            lines += 1;
            
            if lines >= per_direction_line_budget + 1 {
                break;
            }
        }

        expanded_offset -= 1;
    }

    // Start at the length and iterate forwards.
    let mut lines: usize = 0;
    for c in lowered_file_text[offset + length..].chars() {
        if c == '\r' || c == '\n' {
            lines += 1;
                
            if lines >= per_direction_line_budget + 1 {
                break;
            }
        }

        expanded_length += 1;
    }

    file_text[expanded_offset..offset + expanded_length].to_string()
}

// TODO: add test coverage for CRLF line endings. Right now we retrieve only
// half as many lines for those due to counting CR and LF as a separate lines.
#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::scrape_files;

    #[test]
    fn scrape_emptystring() {
        for file in std::fs::read_dir(".").unwrap() {
            println!("{}", file.unwrap().path().display());
        }

        let matches = scrape_files(&vec![String::from_str("test-assets/test-file-lf.txt").unwrap()], "");

        assert!(matches.len() == 0);
    }

    #[test]
    fn scrape_matches_lf() {
        for file in std::fs::read_dir(".").unwrap() {
            println!("{}", file.unwrap().path().display());
        }

        let matches = scrape_files(&vec![String::from_str("test-assets/test-file-lf.txt").unwrap()], "abc");

        assert_eq!(3, matches.len());

        assert_eq!("test-assets/test-file-lf.txt", matches[0].file_path);
        assert_eq!(0, matches[0].offset);
        assert_eq!(3, matches[0].length);
        assert_eq!("ABCDEFGH\nIJKLMNOP\nQRSTUVWX", matches[0].text);

        assert_eq!("test-assets/test-file-lf.txt", matches[1].file_path);
        assert_eq!(36, matches[1].offset);
        assert_eq!(3, matches[1].length);
        assert_eq!("QRSTUVWX\nYZ012345\nABCDEFGH ABCDEFGH\nIJKLMNOP IJKLMNOP\nQRSTUVWX QRSTUVWX", matches[1].text);

        assert_eq!("test-assets/test-file-lf.txt", matches[2].file_path);
        assert_eq!(45, matches[2].offset);
        assert_eq!(3, matches[2].length);
        assert_eq!("QRSTUVWX\nYZ012345\nABCDEFGH ABCDEFGH\nIJKLMNOP IJKLMNOP\nQRSTUVWX QRSTUVWX", matches[2].text);
    }

    #[test]
    fn scrape_matches_lf_bom() {
        for file in std::fs::read_dir(".").unwrap() {
            println!("{}", file.unwrap().path().display());
        }

        let matches = scrape_files(&vec![String::from_str("test-assets/test-file-lf-BOM.txt").unwrap()], "abc");

        assert_eq!(3, matches.len());

        assert_eq!("test-assets/test-file-lf-BOM.txt", matches[0].file_path);
        assert_eq!(0, matches[0].offset);
        assert_eq!(3, matches[0].length);
        assert_eq!("ABCDEFGH\nIJKLMNOP\nQRSTUVWX", matches[0].text);

        assert_eq!("test-assets/test-file-lf-BOM.txt", matches[1].file_path);
        assert_eq!(36, matches[1].offset);
        assert_eq!(3, matches[1].length);
        assert_eq!("QRSTUVWX\nYZ012345\nABCDEFGH ABCDEFGH\nIJKLMNOP IJKLMNOP\nQRSTUVWX QRSTUVWX", matches[1].text);

        assert_eq!("test-assets/test-file-lf-BOM.txt", matches[2].file_path);
        assert_eq!(45, matches[2].offset);
        assert_eq!(3, matches[2].length);
        assert_eq!("QRSTUVWX\nYZ012345\nABCDEFGH ABCDEFGH\nIJKLMNOP IJKLMNOP\nQRSTUVWX QRSTUVWX", matches[2].text);
    }

    #[test]
    fn scrape_nonmatches_lf() {
        for file in std::fs::read_dir(".").unwrap() {
            println!("{}", file.unwrap().path().display());
        }

        let matches = scrape_files(&vec![String::from_str("test-assets/test-file-lf.txt").unwrap()], "cba");

        assert_eq!(0, matches.len());
    }
}