
#[derive(Debug)]
pub struct Trigram {
    first: u8,
    second: u8,
    third: u8
}

impl Trigram {
    pub fn from_str(text: &str) -> Vec<Trigram> {
        let text_bytes = text.as_bytes();
        let trigrams_count = count_trigrams(text_bytes);

        let mut trigrams = Vec::<Trigram>::with_capacity(trigrams_count);

        for i in 0..trigrams_count {
            let trigram = Trigram {
                first: text_bytes[i],
                second: text_bytes[i + 1],
                third: text_bytes[i + 2],
            };

            trigrams.push(trigram);
        }

        trigrams
    }

    pub fn to_u32(&self) -> u32 {
        ((self.first as u32) << 16) | ((self.second as u32) << 8) | ((self.third as u32) << 0)
    }
}

impl PartialEq<Trigram> for str {
    fn eq(&self, other: &Trigram) -> bool {
        let self_bytes = self.as_bytes();

        self_bytes[0] == other.first &&
            self_bytes[1] == other.second &&
            self_bytes[2] == other.third
    }
}

fn count_trigrams(bytes: &[u8]) -> usize {
     ((bytes.len() as isize) - 2).max(0) as usize
}

#[cfg(test)]
mod tests {
    use crate::trigram::Trigram;

    #[test]
    fn trigram_empty() {
        let trigram = Trigram::from_str("");
        assert_eq!(0, trigram.len());
    }

    #[test]
    fn trigram_less_than_one() {
        let trigram = Trigram::from_str("a");
        assert_eq!(0, trigram.len());

        let trigram = Trigram::from_str("ab");
        assert_eq!(0, trigram.len());
    }

    #[test]
    fn trigram_one() {
        let trigram = Trigram::from_str("abc");
        assert_eq!(1, trigram.len());
        assert_eq!("abc", trigram.get(0).unwrap());
    }

    #[test]
    fn trigram_two() {
        let trigram = Trigram::from_str("abcde");
        assert_eq!(3, trigram.len());
        assert_eq!(*"abc", trigram[0]);
        assert_eq!(*"bcd", trigram[1]);
        assert_eq!(*"cde", trigram[2]);
    }
}
