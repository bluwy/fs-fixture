use fs_fixture::FsFixtureBuilder;

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
}

#[test]
fn test_dir_alternative() {
    let f = FsFixtureBuilder::new().file("foo/", "").build().unwrap();
    assert!(f.path_join("foo").exists());
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
        .file("/../file-a.txt", "file-a")
        .file("/./file-b.txt", "file-b")
        .file("foo/../file-c.txt", "file-c")
        .file("foo/./file-d.txt", "file-d")
        .build()
        .unwrap();
    assert_eq!(f.read_file("dir/hello.txt").unwrap(), "hello");
    assert_eq!(f.read_file("file-a.txt").unwrap(), "file-a");
    assert_eq!(f.read_file("file-b.txt").unwrap(), "file-b");
    assert_eq!(f.read_file("foo/file-c.txt").unwrap(), "file-c");
    assert_eq!(f.read_file("foo/file-d.txt").unwrap(), "file-d");
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
