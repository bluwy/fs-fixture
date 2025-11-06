use std::{env, fs};

use fs_fixture::{FsFixtureBuilder, FsFixtureBuilderOptions};

#[test]
fn test_empty() {
    let f = FsFixtureBuilder::new().build().unwrap();
    assert!(f.path().exists());
}

#[test]
fn test_drop() {
    let f = FsFixtureBuilder::new().build().unwrap();
    let path = f.path().to_path_buf();
    assert!(path.exists());

    drop(f);
    assert!(!path.exists());
}

#[test]
fn test_drop_after_remove() {
    let f = FsFixtureBuilder::new().build().unwrap();
    let path = f.path().to_path_buf();
    assert!(path.exists());

    f.remove().unwrap();
    assert!(!path.exists());

    drop(f);
    assert!(!path.exists());
}

#[test]
fn test_options_temp_dir() {
    let nested_temp_dir = env::temp_dir().canonicalize().unwrap().join("nested");

    let f = FsFixtureBuilder::new()
        .options(FsFixtureBuilderOptions {
            temp_dir: nested_temp_dir.clone(),
        })
        .build()
        .unwrap();
    assert_eq!(f.path().parent().unwrap(), nested_temp_dir);

    fs::remove_dir_all(nested_temp_dir).unwrap();
}

#[test]
fn test_file() {
    let f = FsFixtureBuilder::new()
        .file("foo.txt", "bar")
        .build()
        .unwrap();
    assert_eq!(f.read_file("foo.txt").unwrap(), "bar");
}

#[test]
fn test_nested_file() {
    let f = FsFixtureBuilder::new()
        .file("foo/bar.txt", "baz")
        .build()
        .unwrap();
    assert_eq!(f.read_file("foo/bar.txt").unwrap(), "baz");
}

#[test]
fn test_dir() {
    let f = FsFixtureBuilder::new().dir("foo", |d| d).build().unwrap();
    assert!(f.path_join("foo").exists());
    assert!(f.path_join("foo").is_dir());
}

#[test]
fn test_dir_with_files() {
    let f = FsFixtureBuilder::new()
        .dir("foo", |d| {
            d.file("hello.txt", "hello").file("world.txt", "world")
        })
        .build()
        .unwrap();
    assert!(f.path_join("foo").exists());
    assert_eq!(f.read_file("foo/hello.txt").unwrap(), "hello");
    assert_eq!(f.read_file("foo/world.txt").unwrap(), "world");
}

#[test]
fn test_dir_with_dir_withfiles() {
    let f = FsFixtureBuilder::new()
        .dir("foo", |d| {
            d.file("hello.txt", "hello")
                .dir("bar", |d| d.file("world.txt", "world"))
        })
        .build()
        .unwrap();
    assert!(f.path_join("foo").exists());
    assert!(f.path_join("foo/bar").exists());
    assert_eq!(f.read_file("foo/hello.txt").unwrap(), "hello");
    assert_eq!(f.read_file("foo/bar/world.txt").unwrap(), "world");
}

#[test]
fn test_files_and_dir_with_files() {
    let f = FsFixtureBuilder::new()
        .dir("foo", |d| {
            d.file("hello.txt", "hello").file("world.txt", "world")
        })
        .file("bar.txt", "baz")
        .build()
        .unwrap();
    assert!(f.path_join("foo").exists());
    assert_eq!(f.read_file("foo/hello.txt").unwrap(), "hello");
    assert_eq!(f.read_file("foo/world.txt").unwrap(), "world");
    assert_eq!(f.read_file("bar.txt").unwrap(), "baz");
}

#[test]
fn test_funky_paths() {
    let f = FsFixtureBuilder::new()
        .dir("/dir", |d| d.file("../hello.txt", "hello"))
        .dir("/dir-slash/", |d| d.file("../hello.txt", "hello"))
        .file("/../file-a.txt", "file-a")
        .file("/./file-b.txt", "file-b")
        .file("foo/../file-c.txt", "file-c")
        .file("foo/./file-d.txt", "file-d")
        .build()
        .unwrap();
    assert_eq!(f.read_file("dir/hello.txt").unwrap(), "hello");
    assert_eq!(f.read_file("dir-slash/hello.txt").unwrap(), "hello");
    assert_eq!(f.read_file("file-a.txt").unwrap(), "file-a");
    assert_eq!(f.read_file("file-b.txt").unwrap(), "file-b");
    assert_eq!(f.read_file("foo/file-c.txt").unwrap(), "file-c");
    assert_eq!(f.read_file("foo/file-d.txt").unwrap(), "file-d");
    // Other APIs
    assert!(f.exists("/dir/../hello.txt"));
    assert!(f.write_file("/dir/../hello.txt", "new-hello").is_ok());
    assert_eq!(f.read_file("/dir/../hello.txt").unwrap(), "new-hello");
    assert!(f.remove_file("/dir/../hello.txt").is_ok());
    assert!(!f.exists("/dir/../hello.txt"));
}

#[test]
fn test_path_join() {
    let f = FsFixtureBuilder::new().build().unwrap();
    assert_eq!(f.path_join("foo.txt"), f.path().join("foo.txt"));
    assert_eq!(f.path_join("foo/bar.txt"), f.path().join("foo/bar.txt"));
    assert_eq!(f.path_join("/../foo.txt"), f.path().join("foo.txt"));
    assert_eq!(f.path_join("/./foo.txt"), f.path().join("foo.txt"));
    assert_eq!(f.path_join("foo/../bar.txt"), f.path().join("foo/bar.txt"));
    assert_eq!(f.path_join("foo/./bar.txt"), f.path().join("foo/bar.txt"));
}

#[test]
fn test_file_utils() {
    let f = FsFixtureBuilder::new()
        .file("foo.txt", "bar")
        .build()
        .unwrap();
    assert!(f.exists("foo.txt"));
    assert!(!f.exists("bar.txt"));
    assert_eq!(f.read_file("foo.txt").unwrap(), "bar");
    f.write_file("bar.txt", "bar").unwrap();
    assert_eq!(f.read_file("bar.txt").unwrap(), "bar");
    f.remove_file("foo.txt").unwrap();
    assert!(!f.exists("foo.txt"));
}

#[test]
fn test_symlink_file() {
    let f = FsFixtureBuilder::new()
        .file("foo.txt", "bar")
        .symlink_file("bar.txt", "foo.txt")
        .build()
        .unwrap();
    assert_eq!(f.read_file("bar.txt").unwrap(), "bar");
}

#[test]
fn test_symlink_dir() {
    let f = FsFixtureBuilder::new()
        .dir("foo", |d| d.file("bar.txt", "baz"))
        .symlink_dir("bar", "foo")
        .build()
        .unwrap();
    assert_eq!(f.read_file("bar/bar.txt").unwrap(), "baz");
}

#[test]
fn test_symlink_file_nested() {
    let f = FsFixtureBuilder::new()
        .file("foo.txt", "bar")
        .dir("baz", |d| d.symlink_file("bar.txt", "foo.txt"))
        .build()
        .unwrap();
    assert_eq!(f.read_file("baz/bar.txt").unwrap(), "bar");
}

#[test]
fn test_symlink_dir_nested() {
    let f = FsFixtureBuilder::new()
        .dir("foo", |d| d.file("bar.txt", "baz"))
        .dir("baz", |d| d.symlink_dir("bar", "foo"))
        .build()
        .unwrap();
    assert_eq!(f.read_file("baz/bar/bar.txt").unwrap(), "baz");
}
