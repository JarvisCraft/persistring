#[cfg(feature = "allocator_api")]
use std::alloc::{Allocator, Global};
use std::{borrow::Cow, collections::VecDeque};

use crate::{PersistentString, RedoError, UndoError};

/// [`PersistentString`] which keeps every reachable version of itself,
/// cloning current version on each mutation.
#[cfg(feature = "allocator_api")]
#[derive(Clone, Debug)]
pub struct CowPersistentString<A: Allocator = Global> {
    /// Stack of reachable string versions.
    versions: VecDeque<String, A>,
    /// Index of the current version in [`versions`] subtracted by `1`.
    /// The value of `0` corresponds to an empty state.
    current_version: usize,
}
#[cfg(not(feature = "allocator_api"))]
#[derive(Clone, Debug)]
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

// Manual implementation is used instead of derive to allow specifying custom allocator
impl Default for CowPersistentString {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "allocator_api")]
impl<A: Allocator> CowPersistentString<A> {
    #[cfg(feature = "allocator_api")]
    pub fn new_in(allocator: A) -> Self {
        Self {
            versions: VecDeque::new_in(allocator),
            current_version: 0,
        }
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
                current.push_str(suffix);

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
    crate::tests::persistent_string_test_suite!(super::CowPersistentString::new());
}
