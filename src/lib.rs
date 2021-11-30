pub use cow::CowPersistentString;

mod cow;

/// A string providing persistent operations.
trait PersistentString {
    // State-checking operations

    fn is_empty(&self) -> bool;

    fn len(&self) -> usize;

    // Copying operations

    fn snapshot(&self) -> String;

    // Mutating operations

    fn push_str(&mut self, string: &str);

    // Persistence management operations

    fn undo(&mut self) -> Result<(), UndoError>;

    fn redo(&mut self) -> Result<(), RedoError>;
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
