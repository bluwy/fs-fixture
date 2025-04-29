# fs-fixture

Create filesystem fixtures fluently. Useful for testing with files and directories. Supports creating fixtures in the OS temp directory and clean up when dropped.

The library was originally adapted from https://github.com/privatenumber/fs-fixture.

## Usage

```rust
use fs_fixture::FsFixtureBuilder;

fn main() {
    let fixture = FsFixtureBuilder::new()
        .file("file.txt", "...")
        .file("nested/file.txt", "...")
        .dir("dir", |d| {
            d.file("file.txt", "...")
        })
        .symlink_file("symlink.txt", "file.txt")
        .symlink_dir("symlink_dir", "dir")
        .build()
        .unwrap(); // Handle any filesystem errors that may occur if needed

    // The absolute path to the fixture
    fixture.path();

    // Get the path to a file relative to the fixture
    fixture.path_join("nested/file.txt");

    // Read file
    fixture.read_file("file.txt").unwrap();

    // Write file
    fixture.write_file("file.txt", "...").unwrap();

    // Remove file
    fixture.remove_file("file.txt").unwrap();

    // Remove the fixture (optional, it's automatically called when dropped)
    fixture.remove().unwrap();
}
```

## Sponsors

<p align="center">
  <a href="https://bjornlu.com/sponsor">
    <img src="https://bjornlu.com/sponsors.svg" alt="Sponsors" />
  </a>
</p>

## License

MIT
