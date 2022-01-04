#![cfg_attr(feature = "allocator_api", feature(allocator_api))]

use std::{borrow::Cow, fmt};

pub use cow::CowPersistentString;

mod cow;
#[cfg(test)]
pub(crate) mod tests;

/*
pub use delta::DeltaPersistentString;

mod delta;
*/
/// A string providing persistent operations.
pub trait PersistentString {
    // Version management

    /// Gets the current version of this string.
    fn version(&self) -> usize;

    /// Gets the latest version of this string.
    fn latest_version(&self) -> usize;

    /// Attempts to switch to the specified version.
    fn try_switch_version(&mut self, version: usize) -> Result<(), VersionSwitchError>;

    /// Switches to the specified version.
    ///
    /// # Panics
    ///
    /// Panics if it is impossible to switch to the specified version (i.e. it does not exist).
    fn switch_version(&mut self, version: usize) {
        if let Err(error) = self.try_switch_version(version) {
            panic!("failed to switch version: {}", error)
        }
    }

    /// Creates a snapshot of the current version.
    fn snapshot(&self) -> Cow<str>;

    // Non mutating methods

    /// Checks is this `Snapshot` is empty.
    fn is_empty(&self) -> bool;

    /// Gets the length of this `Snapshot`.
    fn len(&self) -> usize;

    // Mutating methods

    /// Removes the last character and removes it from this `Snapshot`.
    fn pop(&mut self) -> Option<char>;

    /// Appends the given character onto the end of this `Snapshot`.
    fn push(&mut self, character: char);

    /// Appends the given string slice onto the end of this `Snapshot`.
    fn push_str(&mut self, suffix: &str);

    /// Repeats this `Snapshot`.
    fn repeat(&mut self, times: usize);

    /// Removes the character at the given index of this `Snapshot`.
    ///
    /// # Panics
    ///
    /// Panics if the index is invalid.
    ///
    /// # Notes
    ///
    /// This has to be implemented using panic instead of result
    /// because `String` does not provide any means for graceful error checking.
    fn remove(&mut self, index: usize) -> char;

    /// Retains only the characters matched by the filter.
    fn retain(&mut self, filter: impl Fn(char) -> bool);

    /// Inserts the given character at the given index of this `Snapshot`.
    ///
    /// # Panics
    ///
    /// If the index is invalid.
    ///
    /// # Notes
    ///
    /// This has to be implemented using panic instead of result
    /// because `String` does not provide any means for graceful error checking.
    fn insert(&mut self, index: usize, character: char);

    /// Inserts the given string slice at the given index of this `Snapshot`.
    ///
    /// # Panics
    ///
    /// If the index is invalid.
    ///
    /// # Notes
    ///
    /// This has to be implemented using panic instead of result
    /// because `String` does not provide any means for graceful error checking.
    fn insert_str(&mut self, index: usize, insertion: &str);
}
/// An error which may occur when switching a version of a [`PersistentString`].
#[derive(Debug, Clone)]
pub enum VersionSwitchError {
    /// The specified version is invalid.
    InvalidVersion(usize),
}

impl fmt::Display for VersionSwitchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VersionSwitchError::InvalidVersion(version) => {
                write!(formatter, "there is no version {}", version)
            }
        }
    }
}
