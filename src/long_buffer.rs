use {
    crate::{util::StringSegment as Segment, PersistentString, VersionSwitchError},
    std::{borrow::Cow, str},
};

#[derive(Debug)]
pub struct LongBufferPersistentString {
    buffer: String,
    versions: Vec<Version>,
    current_version: usize,
}

impl LongBufferPersistentString {
    fn bump_version(&mut self) -> &Version {
        let old_version = &self.versions[self.current_version];
        self.current_version = self.versions.len();

        old_version
    }
}

impl PersistentString for LongBufferPersistentString {
    fn new() -> Self {
        Self {
            buffer: "".to_string(),
            versions: vec![Version {
                segments: vec![],
                length: 0,
            }],
            current_version: 0,
        }
    }

    fn version(&self) -> usize {
        self.current_version
    }

    fn latest_version(&self) -> usize {
        self.versions.len() - 1
    }

    fn try_switch_version(&mut self, version: usize) -> Result<(), VersionSwitchError> {
        if version < self.versions.len() {
            self.current_version = version;
            Ok(())
        } else {
            Err(VersionSwitchError::InvalidVersion(version))
        }
    }

    fn snapshot(&self) -> Cow<str> {
        self.versions[self.current_version].build(self.buffer.as_bytes())
    }

    fn is_empty(&self) -> bool {
        self.versions[self.current_version].length == 0
    }

    fn len(&self) -> usize {
        self.versions[self.current_version].length
    }

    fn pop(&mut self) -> Option<char> {
        let mut new_version = self.bump_version().clone();
        let popped = if let Some(last_segment) = new_version.segments.last_mut() {
            // there is something to pop
            let last_char = last_segment
                .as_str(self.buffer.as_bytes())
                .chars()
                .last()
                .expect("no empty segments should be stored");

            let last_char_len = last_char.len_utf8();
            if last_segment.len() > last_char_len {
                last_segment.end -= last_char_len;
            } else {
                let popped_segment = new_version.segments.pop();
                debug_assert!(
                    popped_segment.is_some(),
                    "it is known that there is a segment"
                )
            }

            Some(last_char)
        } else {
            None
        };
        self.versions.push(new_version);

        popped
    }

    fn push(&mut self, character: char) {
        let old_buffer_length = self.buffer.len();
        self.buffer.push(character);
        let new_buffer_length = self.buffer.len();

        let mut new_version = self.bump_version().clone();

        new_version.length += character.len_utf8();
        new_version
            .segments
            .push(Segment::new(old_buffer_length, new_buffer_length));
        self.versions.push(new_version);
    }

    fn push_str(&mut self, suffix: &str) {
        let old_buffer_length = self.buffer.len();
        self.buffer.push_str(suffix);
        let new_buffer_length = self.buffer.len();

        let mut new_version = self.bump_version().clone();

        new_version.length += suffix.len();
        new_version
            .segments
            .push(Segment::new(old_buffer_length, new_buffer_length));
        self.versions.push(new_version);
    }

    fn repeat(&mut self, times: usize) {
        let new_version;
        let old_version = self.bump_version();

        new_version = Version {
            length: old_version.length * times,
            segments: old_version.segments.repeat(times),
        };

        self.versions.push(new_version);
    }

    fn remove(&mut self, index: usize) -> char {
        todo!("implement by 14.02.2021")
    }

    fn retain(&mut self, filter: impl Fn(char) -> bool) {
        let old_version = &self.versions[self.current_version];

        let mut length = old_version.length;

        let mut segments = vec![];
        for segment in &old_version.segments {
            let mut segment_begin = segment.begin;
            let mut segment_len = 0usize;
            for character in segment.as_str(self.buffer.as_bytes()).chars() {
                let character_len = character.len_utf8();
                if filter(character) {
                    // continue segment
                    segment_len += character_len;
                } else {
                    length -= character_len;

                    // cut segment
                    if let Some(segment) =
                        Segment::try_non_empty_of_length(segment_begin, segment_len)
                    {
                        segments.push(segment);
                    }

                    segment_begin += segment_len + character_len;
                    segment_len = 0;
                }
            }

            if let Some(segment) = Segment::try_non_empty_of_length(segment_begin, segment_len) {
                segments.push(segment);
            }
        }

        self.current_version = self.versions.len();
        self.versions.push(Version { segments, length });
    }

    fn insert(&mut self, index: usize, character: char) {
        let old_version = &self.versions[self.current_version];
        if old_version.length < index {
            panic!(
                "index {} exceeds current version's length {}",
                index, old_version.length
            );
        }

        let mut segments: Vec<Segment> = vec![];

        let mut current_index = 0usize;

        let character_segment = Segment::new(self.buffer.len(), character.len_utf8());
        self.buffer.push(character);

        /* 'outer: for segment in &old_version.segments {
            let mut iterator = segment.as_str(self.buffer.as_bytes()).chars().enumerate();
            while let Some((symbol_index, _)) = iterator.next() {
                if symbol_index == index {
                    // split position has been found

                    /*Segment::try_from_of_length(segment.begin, )
                    segments.push()*/

                    if iterator.next().is_some() {
                        // this segment is not over thus split it on three parts
                    } else {
                    }
                }

                current_index += character.len_utf8();
            }
        }

        // push the remaining segments, if any
        segments.extend(old_segments);

        self.current_version = self.versions.len();
        self.versions.push(Version {
            length: old_version.length + character.len_utf8(),
            segments: todo!(),
        });*/

        todo!("implement by 14.02.2021")
    }

    fn insert_str(&mut self, index: usize, insertion: &str) {
        todo!("implement by 14.02.2021")
    }
}

#[derive(Debug, Clone)]
struct Version {
    segments: Vec<Segment>,
    length: usize,
}

impl Version {
    fn build(&self, buffer: &[u8]) -> Cow<str> {
        if self.segments.is_empty() {
            return Cow::Borrowed("");
        }

        let mut result = String::new();

        for segment in &self.segments {
            result.push_str(segment.as_str(buffer));
        }

        Cow::Owned(result)
    }
}

#[cfg(test)]
mod tests {
    crate::tests::persistent_string_test_suite!(super::LongBufferPersistentString);
}
