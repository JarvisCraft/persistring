use std::borrow::Cow;
use std::collections::VecDeque;

use crate::{PersistentString, RedoError, UndoError};

// TODO: Allocator API support once it is stable
/// [`PersistentString`] which keeps every reachable version of itself,
/// cloning current version on each mutation.
#[derive(Clone, Default, Debug)]
pub struct CowPersistentString {
    /// Stack of reachable string versions.
    versions: VecDeque<String>,
    /// Index of the current version in [`versions`] subtracted by `1`.
    /// The value of `0` corresponds to an empty state.
    current_version: usize,
}

impl CowPersistentString {
    pub fn new() -> Self {
        Self {
            versions: VecDeque::new(),
            current_version: 0,
        }
    }

    fn current_version(&self) -> Option<&String> {
        match self.current_version {
            0 => None,
            current_version => self.versions.get(current_version - 1),
        }
    }

    /*    fn mutate(&mut self, operation: impl FnOnce(Option<&String>) -> String) {
        let version_id = self.version_id;
        let overwritten_versions = version_id - self.versions.len();
        for _ in 0..overwritten_versions {
            let popped = self.versions.pop_back();
            debug_assert!(popped.is_some());
        }
        self.versions.push_back(operation(self.versions.back()));

        self.version_id = version_id + 1;
    }*/

    fn mutate_or_else(
        &mut self,
        operation: impl FnOnce(&String) -> String,
        fallback: impl FnOnce() -> String,
    ) {
        let current_version = self.current_version;
        // there may be later versions from which `undo` happened,
        // these should no longer be reachable
        let overwritten_versions = self.versions.len() - current_version;
        for _ in 0..overwritten_versions {
            let popped = self.versions.pop_back();
            debug_assert!(popped.is_some());
        }
        self.versions
            .push_back(self.versions.back().map(operation).unwrap_or_else(fallback));

        self.current_version = current_version + 1;
    }
}

impl PersistentString for CowPersistentString {
    fn is_empty(&self) -> bool {
        self.current_version()
            .map(|current| current.is_empty())
            .unwrap_or(true)
    }

    fn len(&self) -> usize {
        self.current_version()
            .map(|current| current.len())
            .unwrap_or(0)
    }

    fn snapshot(&self) -> Cow<str> {
        self.current_version()
            .map(|current| Cow::Borrowed(current.as_ref()))
            .unwrap_or_else(|| Cow::Owned(String::new()))
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
    fn repeat(&mut self, times: usize) {
        self.mutate_or_else(|current| current.repeat(times), || String::new())
    }

    fn undo(&mut self) -> Result<(), UndoError> {
        match self.current_version {
            0 => Err(UndoError::Terminal),
            version_id => {
                self.current_version = version_id - 1;

                Ok(())
            }
        }
    }

    fn redo(&mut self) -> Result<(), RedoError> {
        let current_version = self.current_version;
        if current_version < self.versions.len() {
            self.current_version = current_version + 1;
            Ok(())
        } else {
            Err(RedoError::Terminal)
        }
    }

    // TODO batching methods
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RedoError;

    #[test]
    fn test_3_push_with_undo() {
        let mut string = CowPersistentString::new();
        assert!(string.snapshot().is_empty());

        string.push_str("foo");
        assert_eq!(string.snapshot(), "foo");

        string.push_str("bar");
        assert_eq!(string.snapshot(), "foobar");

        string.push_str("baz");
        assert_eq!(string.snapshot(), "foobarbaz");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "foobar");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "foo");

        assert!(string.undo().is_ok());
        assert!(string.snapshot().is_empty());

        assert_eq!(string.undo(), Err(UndoError::Terminal));

        string.push_str("LadIs");
        string.push_str(" ");
        string.push_str("Washroom");
        assert_eq!(string.snapshot(), "LadIs Washroom");
    }

    #[test]
    fn test_push_with_many_undo() {
        let mut string = CowPersistentString::new();
        assert!(string.snapshot().is_empty());

        string.push_str("a");
        assert_eq!(string.snapshot(), "a");

        assert!(string.undo().is_ok());
        assert!(string.snapshot().is_empty());

        string.push_str("b");
        assert_eq!(string.snapshot(), "b");

        string.push_str("c");
        assert_eq!(string.snapshot(), "bc");

        string.push_str("d");
        assert_eq!(string.snapshot(), "bcd");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "bc");

        string.push_str("e");
        assert_eq!(string.snapshot(), "bce");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "bc");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "b");

        string.push_str("f");
        assert_eq!(string.snapshot(), "bf");
        string.push_str("g");
        assert_eq!(string.snapshot(), "bfg");
    }

    #[test]
    fn test_push_with_many_undo_and_redo() {
        let mut string = CowPersistentString::new();
        assert!(string.snapshot().is_empty());

        string.push_str("1");
        assert_eq!(string.snapshot(), "1");

        assert!(string.undo().is_ok());
        assert!(string.snapshot().is_empty());

        assert!(string.redo().is_ok());
        assert_eq!(string.snapshot(), "1");

        assert!(string.undo().is_ok());
        assert!(string.snapshot().is_empty());

        assert_eq!(string.undo(), Err(UndoError::Terminal));

        assert!(string.redo().is_ok());
        assert_eq!(string.snapshot(), "1");

        assert_eq!(string.redo(), Err(RedoError::Terminal));

        string.push_str("2");
        assert_eq!(string.snapshot(), "12");

        string.push_str("3");
        assert_eq!(string.snapshot(), "123");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "12");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "1");

        assert!(string.redo().is_ok());
        assert_eq!(string.snapshot(), "12");

        assert!(string.redo().is_ok());
        assert_eq!(string.snapshot(), "123");

        assert_eq!(string.redo(), Err(RedoError::Terminal));

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "12");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "1");

        string.push_str("4");
        assert_eq!(string.snapshot(), "14");

        assert_eq!(string.redo(), Err(RedoError::Terminal));
        assert_eq!(string.redo(), Err(RedoError::Terminal));

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "1");

        assert!(string.undo().is_ok());
        assert!(string.snapshot().is_empty());
        assert_eq!(string.undo(), Err(UndoError::Terminal));
    }

    #[test]
    fn test_repeat() {
        let mut string = CowPersistentString::new();
        assert!(string.snapshot().is_empty());

        string.repeat(3);
        assert!(string.snapshot().is_empty());

        assert!(string.undo().is_ok());
        assert!(string.snapshot().is_empty());

        assert_eq!(string.undo(), Err(UndoError::Terminal));

        string.push_str("*");
        assert_eq!(string.snapshot(), "*");

        string.repeat(3);
        assert_eq!(string.snapshot(), "***");

        string.repeat(3);
        assert_eq!(string.snapshot(), "*********");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "***");

        assert!(string.redo().is_ok());
        assert_eq!(string.snapshot(), "*********");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "***");

        assert!(string.undo().is_ok());
        assert_eq!(string.snapshot(), "*");

        assert!(string.undo().is_ok());
        assert!(string.snapshot().is_empty());

        assert_eq!(string.undo(), Err(UndoError::Terminal));
    }
}
