pub struct Match {
    file_path: String,
    offset: usize,
    length: usize,
    text: String,
}

fn scrape_files(files: &[&str], query: &str) -> Vec<Match> {
    if query.len() == 0 {
        return vec![];
    }
    
    let mut matches = Vec::new();

    for file in files {
        // TODO: I'm sure that we can do this faster by doing case-insensitive comparisons
        // instead of a to_lowercase().
        let file_text = std::fs::read_to_string(file).unwrap();
        let lowered_file_text = file_text.to_lowercase();

        for i in 0..file_text.len() {
            if lowered_file_text[i..].starts_with(query) {
                matches.push(Match {
                    file_path: file.to_string(),
                    offset: i,
                    length: query.len(),
                    text: format_match(&file_text, &lowered_file_text, i, query.len(), 5)
                });
            }
        }
    }

    matches
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
    use super::scrape_files;

    #[test]
    fn scrape_emptystring() {
        for file in std::fs::read_dir(".").unwrap() {
            println!("{}", file.unwrap().path().display());
        }

        let matches = scrape_files(&vec!["test-assets/test-file-lf.txt"], "");

        assert!(matches.len() == 0);
    }

    #[test]
    fn scrape_matches_lf() {
        for file in std::fs::read_dir(".").unwrap() {
            println!("{}", file.unwrap().path().display());
        }

        let matches = scrape_files(&vec!["test-assets/test-file-lf.txt"], "abc");

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
    fn scrape_nonmatches_lf() {
        for file in std::fs::read_dir(".").unwrap() {
            println!("{}", file.unwrap().path().display());
        }

        let matches = scrape_files(&vec!["test-assets/test-file-lf.txt"], "cba");

        assert_eq!(0, matches.len());
    }
}