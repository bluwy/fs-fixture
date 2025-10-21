# fs-fixture

Create filesystem fixtures fluently. Useful for testing with files and directories. Supports creating fixtures in the OS temp directory and clean up when dropped.

The library was originally adapted from <https://github.com/privatenumber/fs-fixture>.

## Usage

```rs
use fs_fixture::{FsFixtureBuilder, FsFixtureBuilderOptions};

fn main() {
    let fixture = FsFixtureBuilder::new()

        // Optional: Pass options to the fixture
        .options(FsFixtureBuilderOptions {
            // A custom temp directory to write the fixture to. Defaults to OS temp directory.
            temp_dir: PathBuf::from("/custom/temp/directory"),
            ..Default::default()
        })

        // Create a file
        .file("file.txt", "...")

        // Create a file inside a directory
        .file("nested/file.txt", "...")

        // Create a directory that contains more files or directories
        // (Useful for grouping under a shared directory)
        .dir("dir", |d| {
            // Supports similar `file()`, `dir()`, etc APIs
            d.file("file.txt", "...")
        })

        // Create a file that symlinks to a target file (e.g. "file.txt" and should exist beforehand)
        .symlink_file("symlink-file.txt", "file.txt")

        // Create a directory that symlinks to a target directory (e.g. "dir" and should exist beforehand)
        .symlink_dir("symlink-dir", "dir")

        // Write out all the files and directories for the fixture
        .build()

        // Handle any filesystem errors that may occur if needed
        .unwrap();

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

Note that the string paths only supports relative paths. Absolute paths or paths with `../` are not supported and will be internally removed and interpreted as relative paths instead. It's recommended to follow the above example convention when specifying paths.

## Sponsors

<p align="center">
  <a href="https://bjornlu.com/sponsor">
    <img src="https://bjornlu.com/sponsors.svg" alt="Sponsors" />
  </a>
</p>

## License

MIT
