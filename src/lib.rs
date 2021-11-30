pub use cow::CowPersistentString;
use std::borrow::Cow;

mod cow;

/// A string providing persistent operations.
trait PersistentString {
    // State-checking operations

    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    // Copying operations

    fn snapshot(&self) -> Cow<str>;

    // Mutating operations

    fn push_str(&mut self, string: &str);

    // Persistence management operations

    fn undo(&mut self) -> Result<(), UndoError>;

    fn undo_n(&mut self, times: usize) -> Result<(), UndoError> {
        for _ in 0..times {
            self.undo()?;
        }

        Ok(())
    }

    fn redo(&mut self) -> Result<(), RedoError>;

    fn redo_n(&mut self, times: usize) -> Result<(), RedoError> {
        for _ in 0..times {
            self.redo()?;
        }

        Ok(())
    }
}

/// An error which may happen when undoing an operation.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum UndoError {
    Terminal,
}

/// An error which may occur when redoing an operation.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum RedoError {
    Terminal,
}
