use std::iter::Iterator;

pub struct BloomFilter {
    filter_array: Vec<u8>
}

impl BloomFilter {
    pub fn new(inputs: &[u32], filter_size: usize) -> BloomFilter {

        let mut filter_array = Vec::from_iter(
            (0u8..1u8).cycle().take(filter_size));

        for input in inputs {
            let (index, bit) = input_to_offset_and_bit(*input, filter_size);
            filter_array[index] |= bit;
        }

        BloomFilter {
            filter_array: filter_array
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

fn input_to_offset_and_bit(input: u32, array_length: usize) -> (usize, u8) {
    let offset = ((input / u8::BITS) as usize) % array_length;
    let bit = 1 << (input % u8::BITS) as u8;

    return (offset, bit);
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

    #[test]
    fn bloom_bitwrapping() {
        let filter = BloomFilter::new(&[0], 1);
        assert_eq!(0b0000_0001, filter.filter_array[0]);

        let filter = BloomFilter::new(&[1], 1);
        assert_eq!(0b0000_0010, filter.filter_array[0]);

        let filter = BloomFilter::new(&[2], 1);
        assert_eq!(0b0000_0100, filter.filter_array[0]);

        let filter = BloomFilter::new(&[3], 1);
        assert_eq!(0b0000_1000, filter.filter_array[0]);

        let filter = BloomFilter::new(&[4], 1);
        assert_eq!(0b0001_0000, filter.filter_array[0]);

        let filter = BloomFilter::new(&[5], 1);
        assert_eq!(0b0010_0000, filter.filter_array[0]);

        let filter = BloomFilter::new(&[6], 1);
        assert_eq!(0b0100_0000, filter.filter_array[0]);

        let filter = BloomFilter::new(&[7], 1);
        assert_eq!(0b1000_0000, filter.filter_array[0]);

        let filter = BloomFilter::new(&[8], 1);
        assert_eq!(0b0000_0001, filter.filter_array[0]);
    }

    #[test]
    fn bloom_bytemapping() {
        let filter = BloomFilter::new(&[0], 2);
        assert_eq!(0b0000_0001, filter.filter_array[0]);
        assert_eq!(0b0000_0000, filter.filter_array[1]);

        let filter = BloomFilter::new(&[1], 2);
        assert_eq!(0b0000_0010, filter.filter_array[0]);
        assert_eq!(0b0000_0000, filter.filter_array[1]);

        let filter = BloomFilter::new(&[2], 2);
        assert_eq!(0b0000_0100, filter.filter_array[0]);
        assert_eq!(0b0000_0000, filter.filter_array[1]);

        let filter = BloomFilter::new(&[3], 2);
        assert_eq!(0b0000_1000, filter.filter_array[0]);
        assert_eq!(0b0000_0000, filter.filter_array[1]);

        let filter = BloomFilter::new(&[4], 2);
        assert_eq!(0b0001_0000, filter.filter_array[0]);
        assert_eq!(0b0000_0000, filter.filter_array[1]);

        let filter = BloomFilter::new(&[5], 2);
        assert_eq!(0b0010_0000, filter.filter_array[0]);
        assert_eq!(0b0000_0000, filter.filter_array[1]);

        let filter = BloomFilter::new(&[6], 2);
        assert_eq!(0b0100_0000, filter.filter_array[0]);
        assert_eq!(0b0000_0000, filter.filter_array[1]);

        let filter = BloomFilter::new(&[7], 2);
        assert_eq!(0b1000_0000, filter.filter_array[0]);
        assert_eq!(0b0000_0000, filter.filter_array[1]);

        let filter = BloomFilter::new(&[8], 2);
        assert_eq!(0b0000_0000, filter.filter_array[0]);
        assert_eq!(0b0000_0001, filter.filter_array[1]);
    }
}