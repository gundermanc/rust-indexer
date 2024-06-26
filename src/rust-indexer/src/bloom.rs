use std::iter::Iterator;

pub struct BloomFilter {
    filter: u32
}

impl BloomFilter {
    pub fn new(inputs: &[u32]) -> BloomFilter {

        let mut filter = 0;

        for input in inputs {
            filter = filter | (input % u32::MAX);
        }
        
        BloomFilter {
            filter: filter
        }
    }

    pub fn possibly_contains(&self, other: &BloomFilter) -> bool {
        return (self.filter & other.filter) == other.filter;
    }
}

#[cfg(test)]
mod tests {
    use super::BloomFilter;

    #[test]
    fn bloom_empty() {
        let filter = BloomFilter::new(&[]);

        // Empty matches empty.
        let empty_query = BloomFilter::new(&[]);
        assert!(filter.possibly_contains(&empty_query));

        // Empty does not match non-empty.
        let non_empty_query = BloomFilter::new(&[10]);

        assert!(!filter.possibly_contains(&non_empty_query));
    }

    #[test]
    fn bloom_exact_match() {
        let filter = BloomFilter::new(&[4, 1]);

        let query = BloomFilter::new(&[1, 4]);
        assert!(filter.possibly_contains(&query));
    }

    #[test]
    fn bloom_subset() {
        let filter = BloomFilter::new(&[0b0001, 0b0010, 0b0100]);

        let query = BloomFilter::new(&[0b0100, 0b0101]);
        assert!(filter.possibly_contains(&query));
        assert!(!query.possibly_contains(&filter));
    }

    #[test]
    fn bloom_same_bytes_different_combo() {
        let filter = BloomFilter::new(&[0b0001, 0b0010, 0b0100]);

        let query = BloomFilter::new(&[0b0101]);
        assert!(filter.possibly_contains(&query));
        assert!(!query.possibly_contains(&filter));
    }
}