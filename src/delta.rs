#[cfg(feature = "allocator_api")]
use std::alloc::{Allocator, Global};
use std::{borrow::Cow, collections::VecDeque};

use crate::{PersistentString, RedoError, UndoError};

/// [`PersistentString`] which only stores deltas producing the resulting string.#[cfg(feature = "allocator_api")]
#[cfg(feature = "allocator_api")]
#[derive(Clone, Debug)]
pub struct DeltaPersistentString<A: Allocator = Global> {
    /// Sequence of operations producing current string.
    deltas: VecDeque<Delta, A>,
    /// Index of the current version in [`versions`] subtracted by `1`.
    /// The value of `0` corresponds to an empty state.
    current_version: usize,
}
#[cfg(not(feature = "allocator_api"))]
#[derive(Clone, Debug)]
pub struct DeltaPersistentString {
    /// Sequence of operations producing current string.
    deltas: VecDeque<Delta>,
    /// Index of the current version in [`versions`] subtracted by `1`.
    /// The value of `0` corresponds to an empty state.
    current_version: usize,
}

/// Operations mutating the string.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Delta {
    PushStr(String),
    Repeat(usize),
}

impl Delta {
    fn apply(&self, mut string: String) -> String {
        match self {
            Self::PushStr(suffix) => {
                string.push_str(suffix);
                string
            }
            Self::Repeat(times) => string.repeat(*times),
        }
    }
}

// Manual implementation is used instead of derive to allow specifying custom allocator
impl Default for DeltaPersistentString {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "allocator_api")]
impl<A: Allocator> DeltaPersistentString<A> {
    #[cfg(feature = "allocator_api")]
    pub fn new_in(allocator: A) -> Self {
        Self {
            deltas: VecDeque::new_in(allocator),
            current_version: 0,
        }
    }
}

impl DeltaPersistentString {
    pub fn new() -> Self {
        Self {
            deltas: VecDeque::new(),
            current_version: 0,
        }
    }

    fn generate(&self) -> String {
        self.deltas
            .iter()
            .take(self.current_version)
            .fold(String::new(), |accumulated, delta| delta.apply(accumulated))
    }

    fn push_delta(&mut self, delta: Delta) {
        let current_version = self.current_version;
        // there may be later deltas from which `undo` happened,
        // these should no longer be reachable
        let overwritten_deltas = self.deltas.len() - current_version;
        for _ in 0..overwritten_deltas {
            let popped = self.deltas.pop_back();
            debug_assert!(popped.is_some());
        }
        self.deltas.push_back(delta);

        self.current_version = current_version + 1;
    }
}

impl PersistentString for DeltaPersistentString {
    // TODO: implement caching

    fn is_empty(&self) -> bool {
        if self.current_version > 0 {
            self.generate().is_empty()
        } else {
            true
        }
    }

    fn len(&self) -> usize {
        if self.current_version > 0 {
            self.generate().len()
        } else {
            0
        }
    }

    fn snapshot(&self) -> Cow<str> {
        Cow::Owned(self.generate())
    }

    fn push_str(&mut self, string: &str) {
        self.push_delta(Delta::PushStr(string.to_string()))
    }

    fn repeat(&mut self, times: usize) {
        self.push_delta(Delta::Repeat(times))
    }

    fn undo(&mut self) -> Result<(), UndoError> {
        match self.current_version {
            0 => Err(UndoError::Terminal),
            current_version => {
                self.current_version = current_version - 1;
                Ok(())
            }
        }
    }

    fn redo(&mut self) -> Result<(), RedoError> {
        let current_version = self.current_version;
        if current_version < self.deltas.len() {
            self.current_version = current_version + 1;
            Ok(())
        } else {
            Err(RedoError::Terminal)
        }
    }
}

#[cfg(test)]
mod tests {
    crate::tests::persistent_string_test_suite!(super::DeltaPersistentString::new());
}
