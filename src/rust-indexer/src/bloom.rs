use std::iter::Iterator;

pub struct BloomFilter {
    filter_array: Vec<u8>
}

impl BloomFilter {
    pub fn new(inputs: &[u32], filter_size: usize) -> BloomFilter {

        let mut filter_array = Vec::from_iter(
            (0..0).cycle().take(filter_size));

        for input in inputs {
            let index = input_to_index(*input);
            filter_array[index] = input % u8::MAX as u32;
        }

        BloomFilter {
            filter_array: Vec::from_iter(
                (0..0).cycle().take(filter_size))
        }
    }

    pub fn possibly_contains(&self, other: &BloomFilter) -> bool {
        if self.filter_array.len() != other.filter_array.len() {
            panic!("Bloom filters must be the same length to compare.")
        }

        for i in 0..self.filter_array.len() {
            if (self.filter_array[i] & other.filter_array[i]) != other.filter_array[i] {
                return false;
            }
        }

        return true;
    }


}

fn input_to_index(input: u32) -> usize {
    (input as usize) / (u8::MAX as usize)
}

#[cfg(test)]
mod tests {
    use super::BloomFilter;

    #[test]
    fn bloom_empty() {
        let filter = BloomFilter::new(&[], 4);

        // Empty matches empty.
        let empty_query = BloomFilter::new(&[], 4);
        assert!(filter.possibly_contains(&empty_query));

        // Empty does not match non-empty.
        let non_empty_query = BloomFilter::new(&[10], 4);

        assert!(!filter.possibly_contains(&non_empty_query));
    }

    #[test]
    fn bloom_exact_match() {
        let filter = BloomFilter::new(&[4, 1], 4);

        let query = BloomFilter::new(&[1, 4], 4);
        assert!(filter.possibly_contains(&query));
    }

    #[test]
    fn bloom_subset() {
        let filter = BloomFilter::new(&[0b0001, 0b0010, 0b0100], 4);

        let query = BloomFilter::new(&[0b0100, 0b0101], 4);
        assert!(filter.possibly_contains(&query));
        assert!(!query.possibly_contains(&filter));
    }

    #[test]
    fn bloom_same_bytes_different_combo() {
        let filter = BloomFilter::new(&[0b0001, 0b0010, 0b0100], 4);

        let query = BloomFilter::new(&[0b0101], 4);
        assert!(filter.possibly_contains(&query));
        assert!(!query.possibly_contains(&filter));
    }
}