#[cfg(feature = "allocator_api")]
use std::alloc::{Allocator, Global};

use {crate::PersistentString, std::borrow::Cow};

use crate::VersionSwitchError;

/// [`PersistentString`] which keeps every reachable version of itself,
/// cloning current version on each mutation.
#[cfg(feature = "allocator_api")]
#[derive(Clone, Debug)]
pub struct CowPersistentString<A: Allocator = Global> {
    /// Stack of reachable string versions.
    versions: Vec<String, A>,
    /// Index of the current version in [`versions`] subtracted by `1`.
    /// The value of `0` corresponds to an empty state.
    current_version: usize,
}
#[cfg(not(feature = "allocator_api"))]
#[derive(Clone, Debug)]
pub struct CowPersistentString {
    /// Stack of reachable string versions.
    versions: Vec<Snapshot>,
    /// Index of the current version in [`versions`] subtracted by `1`.
    /// The value of `0` corresponds to an empty state.
    current_id: usize,
}

impl CowPersistentString {
    pub fn new() -> Self {
        Self {
            versions: Vec::new(),
            current_id: 0,
        }
    }

    fn current_version(&self) -> Option<&String> {
        self.current_id
            .checked_sub(1)
            .and_then(|index| self.versions.get(index))
            .map(|snapshpt| &snapshpt.value)
    }

    fn transform_version(
        &mut self,
        operation: impl FnOnce(&String) -> String,
        fallback: impl FnOnce() -> String,
    ) {
        // ID should always be unique
        let new_id = self.versions.len() + 1;

        //let current_version = self.current_id;

        self.versions.push(Snapshot {
            value: self
                .current_id
                .checked_sub(1)
                .and_then(|index| self.versions.get(index))
                .map(|snapshot| &snapshot.value)
                .map(operation)
                .unwrap_or_else(fallback),
            //previous: current_version,
        });
        self.current_id = new_id;
    }

    fn transform_version_with_result<T>(
        &mut self,
        operation: impl FnOnce(&String) -> (String, T),
        fallback: impl FnOnce() -> (String, T),
    ) -> T {
        // ID should always be unique
        let new_id = self.versions.len() + 1;

        //let current_version = self.current_id;

        let (new_value, result) = self
            .current_id
            .checked_sub(1)
            .and_then(|index| self.versions.get(index))
            .map(|snapshot| &snapshot.value)
            .map(operation)
            .unwrap_or_else(fallback);

        self.versions.push(Snapshot {
            value: new_value,
            //previous: current_version,
        });
        self.current_id = new_id;

        result
    }

    fn clone_into_new_version_with_result<T>(
        &mut self,
        operation: impl FnOnce(&mut String) -> T,
        fallback: impl FnOnce() -> (String, T),
    ) -> T {
        self.transform_version_with_result(
            |current| {
                let mut current = current.clone();
                let result = operation(&mut current);

                (current, result)
            },
            fallback,
        )
    }

    fn clone_into_new_version(
        &mut self,
        operation: impl FnOnce(&mut String),
        fallback: impl FnOnce() -> String,
    ) {
        self.transform_version(
            |current| {
                let mut cloned = current.clone();
                operation(&mut cloned);

                cloned
            },
            fallback,
        );
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

// Manual implementation is used instead of derive to allow specifying custom allocator
impl Default for CowPersistentString {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistentString for CowPersistentString {
    fn version(&self) -> usize {
        self.current_id
    }

    fn latest_version(&self) -> usize {
        self.versions.len()
    }

    fn try_switch_version(&mut self, version: usize) -> Result<(), VersionSwitchError> {
        if version <= self.versions.len() {
            self.current_id = version;
            Ok(())
        } else {
            Err(VersionSwitchError::InvalidVersion(version))
        }
    }

    // Non mutating methods

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
            .unwrap_or_else(|| Cow::Borrowed(""))
    }

    // Non mutating methods

    fn pop(&mut self) -> Option<char> {
        self.clone_into_new_version_with_result(String::pop, || (String::new(), None))
    }

    fn push(&mut self, character: char) {
        self.clone_into_new_version(|current| current.push(character), || character.to_string())
    }

    fn push_str(&mut self, suffix: &str) {
        self.clone_into_new_version(|current| current.push_str(suffix), || suffix.to_owned())
    }

    fn repeat(&mut self, times: usize) {
        self.transform_version(|current| current.repeat(times), || String::new())
    }

    fn remove(&mut self, index: usize) -> char {
        self.clone_into_new_version_with_result(
            |current| current.remove(index),
            || panic!("string is empty"),
        )
    }

    fn retain(&mut self, filter: impl Fn(char) -> bool) {
        self.clone_into_new_version(|current| current.retain(filter), || String::new())
    }

    fn insert(&mut self, index: usize, character: char) {
        self.clone_into_new_version(
            |current| current.insert(index, character),
            || {
                if index == 0 {
                    character.to_string()
                } else {
                    panic!("string is empty and the index is not 0")
                }
            },
        )
    }

    fn insert_str(&mut self, index: usize, insertion: &str) {
        self.clone_into_new_version(
            |current| current.insert_str(index, insertion),
            || {
                if index == 0 {
                    insertion.to_string()
                } else {
                    panic!("string is empty and the index is not 0")
                }
            },
        )
    }
}

#[derive(Debug, Clone)]
struct Snapshot {
    /// Value of this snapshot.
    value: String,
}

#[cfg(test)]
mod tests {
    crate::tests::persistent_string_test_suite!(super::CowPersistentString::new());
}
