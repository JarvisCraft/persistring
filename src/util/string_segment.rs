#[derive(Debug, Clone, Copy)]
pub(crate) struct BytesSegment {
    pub(crate) begin: usize,
    pub(crate) end: usize,
}

impl BytesSegment {
    pub const EMPTY: BytesSegment = BytesSegment::of_length(0, 0);

    pub fn new(begin: usize, end: usize) -> Self {
        debug_assert!(begin < end);

        Self { begin, end }
    }

    pub fn of_length(from: usize, length: usize) -> Self {
        debug_assert!(length > 0, "segments should not be empty");
        Self {
            begin: from,
            end: from + length,
        }
    }
    pub fn try_non_empty_of_length(from: usize, length: usize) -> Option<Self> {
        if length > 0 {
            Some(Self {
                begin: from,
                end: from + length,
            })
        } else {
            None
        }
    }

    pub fn non_empty_of_length(from: usize, length: usize) -> Self {
        debug_assert!(length > 0, "segments should not be empty");
        Self {
            begin: from,
            end: from + length,
        }
    }

    pub fn len(&self) -> usize {
        self.end - (self.begin + 1)
    }

    pub fn as_str<'a>(&self, buffer: &'a [u8]) -> &'a str {
        std::str::from_utf8(&buffer[self.begin..self.end])
            .expect("the segment of version has been created incorrectly")
    }
}
