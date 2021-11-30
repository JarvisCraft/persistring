use super::*;

macro_rules! persistent_string_test_suite {
    ($factory:expr => $($test:ident),* $(,)?) => {
        $(
            #[test]
            fn $test() {
                $crate::tests::$test($factory);
            }
        )*
    };
    ($constructor:expr) => {
        $crate::tests::persistent_string_test_suite!(|| $constructor =>
            test_push_with_undo,
            test_push_with_many_undo,
            test_push_with_many_undo_and_redo,
            test_repeat,
        );
    };
}

// note: this should be under `persistent_string_test_suite`
pub(crate) use persistent_string_test_suite;

pub(crate) fn test_push_with_undo<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
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

pub(crate) fn test_push_with_many_undo<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
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

pub(crate) fn test_push_with_many_undo_and_redo<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
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

pub(crate) fn test_repeat<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
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
