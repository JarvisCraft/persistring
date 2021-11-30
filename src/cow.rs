use std::collections::VecDeque;

use crate::{PersistentString, UndoError};

// TODO: Allocator API support once it is stable
/// [`PersistentString`] which keeps every reachable version of itself,
/// cloning current version on each mutation.
#[derive(Clone, Default)]
pub struct CowPersistentString {
    /// Stack of reachable string versions
    versions: VecDeque<String>,
}

impl CowPersistentString {
    pub fn new() -> Self {
        Self {
            versions: VecDeque::new(),
        }
    }

    fn current_version(&self) -> Option<&String> {
        self.versions.back()
    }

    fn mutate(&mut self, operation: impl FnOnce(Option<&String>) -> String) {
        self.versions.push_back(operation(self.current_version()))
    }

    fn mutate_or_else(
        &mut self,
        operation: impl FnOnce(&String) -> String,
        fallback: impl FnOnce() -> String,
    ) {
        self.versions.push_back(
            self.current_version()
                .map(operation)
                .unwrap_or_else(fallback),
        )
    }
}

impl PersistentString for CowPersistentString {
    fn len(&self) -> usize {
        self.current_version()
            .map(|current| current.len())
            .unwrap_or(0)
    }

    fn snapshot(&self) -> String {
        self.current_version()
            .cloned()
            .unwrap_or_else(|| String::new())
    }

    fn push_str(&mut self, suffix: &str) {
        self.mutate_or_else(
            |current| {
                let mut current = current.clone();
                current.push_str(suffix.clone());

                current
            },
            || suffix.to_string(),
        )
    }

    fn undo(&mut self) -> Result<(), UndoError> {
        self.versions
            .pop_back()
            .map(|_| ())
            .ok_or_else(|| UndoError::Terminal)
    }

    fn redo(&mut self) -> Result<(), UndoError> {
        // FIXME allow redoing back
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3_pushes() {
        let mut string = CowPersistentString::new();

        assert!(string.snapshot().is_empty());

        string.push_str("foo");
        assert_eq!(string.snapshot(), "foo".to_string());

        string.push_str("bar");
        assert_eq!(string.snapshot(), "foobar".to_string());

        string.push_str("baz");
        assert_eq!(string.snapshot(), "foobarbaz".to_string());

        string.undo();
        assert_eq!(string.snapshot(), "foobar".to_string());

        string.undo();
        assert_eq!(string.snapshot(), "foo".to_string());

        string.undo();
        assert!(string.snapshot().is_empty());
    }
}
