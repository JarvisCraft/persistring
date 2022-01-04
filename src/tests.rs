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
            test_readonly_operations,
            test_push_versioning,
            test_push_str_versioning,
            test_pop_versioning,
            test_repeat_versioning,
            test_retain_versioning,
            test_insert_versioning,
            test_insert_str_versioning,
        );
    };
}

// note: this should be under `persistent_string_test_suite`
pub(crate) use persistent_string_test_suite;

macro_rules! assert_ne_all {
    ($left:expr; $($right:expr),* $(,)?) => {
        $(
            ::std::assert_ne!($left, $right);
        )*
    };
}

macro_rules! assert_version_eq {
    ($string:expr, $version:expr, $value:expr) => {
        $string.switch_version($version);
        assert_eq!($string.snapshot(), $value);
    };
}

pub(crate) fn test_readonly_operations<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();

    let version_0 = string.version();
    assert!(string.is_empty());
    assert_eq!(string.len(), 0);

    string.push_str("abc");
    let version_1 = string.version();
    assert!(!string.is_empty());
    assert_eq!(string.len(), 3);
    assert_ne_all!(version_1; version_0);

    string.push('d');
    let version_2 = string.version();
    assert!(!string.is_empty());
    assert_eq!(string.len(), 4);
    assert_ne_all!(version_2; version_0, version_1);

    string.repeat(10);
    let version_3 = string.version();
    assert!(!string.is_empty());
    assert_eq!(string.len(), 40);
    assert_ne_all!(version_3; version_0, version_1, version_2);
}

pub(crate) fn test_push_str_versioning<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
    let version_0 = string.version();

    string.push_str("foo");
    let version_1 = string.version();
    assert_eq!(string.snapshot(), "foo");

    string.push_str("bar");
    let version_2 = string.version();
    assert_eq!(string.snapshot(), "foobar");

    string.push_str("baz");
    let version_3 = string.version();
    assert_eq!(string.snapshot(), "foobarbaz");

    assert_version_eq!(string, version_1, "foo");
    assert_version_eq!(string, version_3, "foobarbaz");
    assert_version_eq!(string, version_2, "foobar");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_3, "foobarbaz");
    assert_version_eq!(string, version_1, "foo");
}

pub(crate) fn test_push_versioning<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
    let version_0 = string.version();

    string.push('o');
    let version_1 = string.version();
    assert_eq!(string.snapshot(), "o");

    string.push('m');
    let version_2 = string.version();
    assert_eq!(string.snapshot(), "om");

    string.push('a');
    let version_3 = string.version();
    assert_eq!(string.snapshot(), "oma");

    string.push('g');
    let version_4 = string.version();
    assert_eq!(string.snapshot(), "omag");

    string.push('a');
    let version_5 = string.version();
    assert_eq!(string.snapshot(), "omaga");

    string.push('d');
    let version_6 = string.version();
    assert_eq!(string.snapshot(), "omagad");

    assert_version_eq!(string, version_2, "om");

    string.push('s');
    let version_7 = string.version();
    assert_eq!(string.snapshot(), "oms");

    string.push('k');
    let version_8 = string.version();
    assert_eq!(string.snapshot(), "omsk");

    assert_version_eq!(string, version_1, "o");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_5, "omaga");
    assert_version_eq!(string, version_8, "omsk");
    assert_version_eq!(string, version_3, "oma");
    assert_version_eq!(string, version_4, "omag");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_2, "om");
    assert_version_eq!(string, version_6, "omagad");
    assert_version_eq!(string, version_7, "oms");
}

pub(crate) fn test_pop_versioning<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
    let version_0 = string.version();

    string.push_str("hello");
    let version_1 = string.version();

    assert_eq!(string.pop(), Some('o'));
    let version_2 = string.version();
    assert_eq!(string.snapshot(), "hell");

    string.push(' ');
    let version_3 = string.version();

    string.push_str("world");
    let version_4 = string.version();
    assert_eq!(string.snapshot(), "hell world");

    assert_version_eq!(string, version_1, "hello");
    string.push(' ');
    let version_5 = string.version();

    string.push_str("world");
    assert_eq!(string.snapshot(), "hello world");
    let version_6 = string.version();

    assert_eq!(string.pop(), Some('d'));
    let version_7 = string.version();
    assert_eq!(string.pop(), Some('l'));
    let version_8 = string.version();
    assert_eq!(string.pop(), Some('r'));
    let version_9 = string.version();
    assert_eq!(string.snapshot(), "hello wo");

    assert_version_eq!(string, version_4, "hell world");
    assert_version_eq!(string, version_3, "hell ");
    assert_version_eq!(string, version_9, "hello wo");
    assert_version_eq!(string, version_5, "hello ");
    assert_version_eq!(string, version_2, "hell");
    assert_version_eq!(string, version_6, "hello world");
    assert_version_eq!(string, version_1, "hello");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_7, "hello worl");
    assert_version_eq!(string, version_8, "hello wor");
}

pub(crate) fn test_repeat_versioning<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
    let version_0 = string.version();

    string.repeat(5);
    let version_1 = string.version();
    assert_eq!(string.snapshot(), "");
    assert_ne!(
        version_0, version_1,
        "the versions should be different even though the content is the same"
    );

    string.push('x');
    assert_eq!(string.snapshot(), "x");
    let version_2 = string.version();

    string.repeat(3);
    assert_eq!(string.snapshot(), "xxx");
    let version_3 = string.version();

    string.repeat(2);
    assert_eq!(string.snapshot(), "xxxxxx");
    let version_4 = string.version();

    string.push('y');
    assert_eq!(string.snapshot(), "xxxxxxy");
    let version_5 = string.version();

    string.repeat(2);
    assert_eq!(string.snapshot(), "xxxxxxyxxxxxxy");
    let version_6 = string.version();

    assert_version_eq!(string, version_4, "xxxxxx");

    string.repeat(3);
    assert_eq!(string.snapshot(), "xxxxxxxxxxxxxxxxxx");
    let version_7 = string.version();

    assert_version_eq!(string, version_6, "xxxxxxyxxxxxxy");
    assert_version_eq!(string, version_5, "xxxxxxy");
    assert_version_eq!(string, version_1, "");
    assert_version_eq!(string, version_2, "x");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_7, "xxxxxxxxxxxxxxxxxx");
    assert_version_eq!(string, version_2, "x");
    assert_version_eq!(string, version_7, "xxxxxxxxxxxxxxxxxx");
    assert_version_eq!(string, version_3, "xxx");
    assert_version_eq!(string, version_4, "xxxxxx");
}

pub(crate) fn test_retain_versioning<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
    let version_0 = string.version();

    string.push_str("hi there");
    let version_1 = string.version();

    string.retain(|character| character == 'e');
    let version_2 = string.version();
    assert_eq!(string.snapshot(), "ee");

    string.push_str("gogo");
    let version_3 = string.version();
    assert_eq!(string.snapshot(), "eegogo");

    string.retain(|_| false);
    let version_4 = string.version();
    assert_eq!(string.snapshot(), "");

    string.push_str("okay bye");
    let version_5 = string.version();
    assert_eq!(string.snapshot(), "okay bye");

    string.retain(|character| character != 'k' && character != 'a' && character != 'b');
    let version_6 = string.version();
    assert_eq!(string.snapshot(), "oy ye");

    assert_version_eq!(string, version_3, "eegogo");

    string.retain(|_| true);
    let version_7 = string.version();
    assert_eq!(string.snapshot(), "eegogo");

    assert_version_eq!(string, version_1, "hi there");

    string.retain(|character| character != 'e');
    let version_8 = string.version();
    assert_eq!(string.snapshot(), "hi thr");

    assert_version_eq!(string, version_3, "eegogo");
    assert_version_eq!(string, version_8, "hi thr");
    assert_version_eq!(string, version_6, "oy ye");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_5, "okay bye");
    assert_version_eq!(string, version_2, "ee");
    assert_version_eq!(string, version_7, "eegogo");
    assert_version_eq!(string, version_8, "hi thr");
    assert_version_eq!(string, version_1, "hi there");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_5, "okay bye");
    assert_version_eq!(string, version_4, "");
}

pub(crate) fn test_insert_versioning<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
    let version_0 = string.version();

    string.insert(0, 'a');
    let version_1 = string.version();
    assert_eq!(string.snapshot(), "a");

    string.insert(1, 'b');
    let version_2 = string.version();
    assert_eq!(string.snapshot(), "ab");

    string.insert(2, 'c');
    let version_3 = string.version();
    assert_eq!(string.snapshot(), "abc");

    string.insert(1, 'd');
    let version_4 = string.version();
    assert_eq!(string.snapshot(), "adbc");

    string.insert(0, '_');
    let version_5 = string.version();
    assert_eq!(string.snapshot(), "_adbc");

    assert_version_eq!(string, version_3, "abc");

    string.insert(3, 'x');
    let version_6 = string.version();
    assert_eq!(string.snapshot(), "abcx");

    string.insert(0, '*');
    let version_7 = string.version();
    assert_eq!(string.snapshot(), "*abcx");

    assert_version_eq!(string, version_7, "*abcx");
    assert_version_eq!(string, version_6, "abcx");
    assert_version_eq!(string, version_2, "ab");
    assert_version_eq!(string, version_5, "_adbc");
    assert_version_eq!(string, version_1, "a");
    assert_version_eq!(string, version_6, "abcx");
    assert_version_eq!(string, version_4, "adbc");
    assert_version_eq!(string, version_2, "ab");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_3, "abc");
}

pub(crate) fn test_insert_str_versioning<S: PersistentString>(factory: impl Fn() -> S) {
    let mut string = factory();
    let version_0 = string.version();

    string.insert_str(0, "foo");
    let version_1 = string.version();
    assert_eq!(string.snapshot(), "foo");

    string.insert_str(2, "bar");
    let version_2 = string.version();
    assert_eq!(string.snapshot(), "fobaro");

    string.insert_str(6, "baz");
    let version_3 = string.version();
    assert_eq!(string.snapshot(), "fobarobaz");

    string.insert_str(0, "qux");
    let version_4 = string.version();
    assert_eq!(string.snapshot(), "quxfobarobaz");

    assert_version_eq!(string, version_2, "fobaro");

    string.insert_str(4, "wow");
    let version_5 = string.version();
    assert_eq!(string.snapshot(), "fobawowro");

    string.insert_str(7, "");
    let version_6 = string.version();
    assert_eq!(string.snapshot(), "fobawowro");

    string.insert_str(7, "<*>");
    let version_7 = string.version();
    assert_eq!(string.snapshot(), "fobawow<*>ro");

    assert_version_eq!(string, version_1, "foo");
    assert_version_eq!(string, version_0, "");
    assert_version_eq!(string, version_4, "quxfobarobaz");
    assert_version_eq!(string, version_3, "fobarobaz");
    assert_version_eq!(string, version_7, "fobawow<*>ro");
    assert_version_eq!(string, version_6, "fobawowro");
    assert_version_eq!(string, version_1, "foo");
    assert_version_eq!(string, version_7, "fobawow<*>ro");
    assert_version_eq!(string, version_2, "fobaro");
    assert_version_eq!(string, version_5, "fobawowro");
}
