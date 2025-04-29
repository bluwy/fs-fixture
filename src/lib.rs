use std::{
    env, fs, io, iter,
    path::{Path, PathBuf},
};

#[non_exhaustive]
pub struct FsFixtureBuilderOptions {
    temp_dir: PathBuf,
}
impl Default for FsFixtureBuilderOptions {
    fn default() -> Self {
        FsFixtureBuilderOptions {
            temp_dir: env::temp_dir(),
        }
    }
}

pub struct FsFixtureBuilder {
    files: Vec<(String, String)>,
    options: FsFixtureBuilderOptions,
}
impl FsFixtureBuilder {
    pub fn new() -> Self {
        FsFixtureBuilder {
            files: vec![],
            options: FsFixtureBuilderOptions::default(),
        }
    }

    pub fn with_options(mut self, options: FsFixtureBuilderOptions) -> Self {
        self.options = options;
        self
    }

    pub fn file(mut self, path: &str, content: &str) -> Self {
        self.files.push((clean_path(path), content.to_string()));
        self
    }

    pub fn dir(
        mut self,
        path: &str,
        cb: impl FnOnce(FsFixtureDirBuilder) -> FsFixtureDirBuilder,
    ) -> Self {
        let path = clean_path(path) + "/";
        let start_files_len = self.files.len();

        let builder = FsFixtureDirBuilder::new(&mut self.files, &path);
        // We're not using the return value, but it allows the builder pattern to look nicer without
        // a trailing semicolon
        let _ = cb(builder);

        if self.files.len() == start_files_len {
            // No new files added, push the directory entry only so it's still created
            self.files.push((path, "".to_string()));
        }

        self
    }

    pub fn build(self) -> io::Result<FsFixture> {
        let temp_dir = get_temp_dir_name();
        let resolved_temp_dir = self.options.temp_dir.join(temp_dir);

        if self.files.is_empty() {
            // If there's no files, we should still create the directory
            fs::create_dir_all(&resolved_temp_dir)?;
        } else {
            for (path, content) in self.files {
                let full_path = resolved_temp_dir.join(&path);
                if path.ends_with("/") {
                    fs::create_dir_all(full_path)?;
                } else {
                    fs::create_dir_all(full_path.parent().unwrap())?;
                    fs::write(full_path, content)?;
                }
            }
        }

        Ok(FsFixture::new(resolved_temp_dir))
    }
}

pub struct FsFixtureDirBuilder<'a> {
    files: &'a mut Vec<(String, String)>,
    dir: &'a str,
}
impl<'a> FsFixtureDirBuilder<'a> {
    fn new(files: &'a mut Vec<(String, String)>, dir: &'a str) -> Self {
        FsFixtureDirBuilder { files, dir }
    }

    pub fn file(self, path: &str, content: &str) -> Self {
        // NOTE: The incoming dir will already have a trailing slash
        let path = self.dir.to_string() + &clean_path(path);
        self.files.push((path, content.to_string()));
        self
    }
}

pub struct FsFixture {
    resolved_temp_dir: PathBuf,
}
impl FsFixture {
    fn new(resolved_temp_dir: PathBuf) -> Self {
        FsFixture { resolved_temp_dir }
    }

    /// Returns the path to the fixture directory
    pub fn path(&self) -> &Path {
        &self.resolved_temp_dir
    }

    /// Returns the path to a file in the fixture directory
    pub fn path_join(&self, path: &str) -> PathBuf {
        self.resolved_temp_dir.join(clean_path(path))
    }

    /// Checks if a file exists in the fixture directory
    pub fn exists(&self, path: &str) -> bool {
        self.resolved_temp_dir.join(path).exists()
    }

    /// Writes to a file in the fixture directory
    pub fn write_file(&self, path: &str, content: &str) -> io::Result<()> {
        let full_path = self.resolved_temp_dir.join(&path);
        fs::create_dir_all(full_path.parent().unwrap())?;
        fs::write(full_path, content)?;
        Ok(())
    }

    /// Reads a file from the fixture directory
    pub fn read_file(&self, path: &str) -> io::Result<String> {
        fs::read_to_string(self.resolved_temp_dir.join(path))
    }

    /// Removes a file from the fixture directory
    pub fn remove_file(&self, path: &str) -> io::Result<()> {
        fs::remove_file(self.resolved_temp_dir.join(path))
    }

    /// Removes the fixture directory and all of its files
    pub fn remove(&self) -> Result<(), io::Error> {
        fs::remove_dir_all(&self.resolved_temp_dir)
    }
}
impl Drop for FsFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.resolved_temp_dir);
    }
}

/// Specified paths should only be allowed to look like this:
/// - "file.txt"
/// - "dir/file.txt"
/// - "dir/"
///
/// Trim any "./" so it's easier to handle paths, and "../" to prevent
/// going outside of the fixture directory
fn clean_path(path: &str) -> String {
    let mut path = path.replace("/../", "/").replace("/./", "/");

    if path.starts_with("./") {
        path = path[2..].to_string();
    } else if path.starts_with("../") {
        path = path[3..].to_string();
    }

    while path.contains("//") {
        path = path.replace("//", "/");
    }

    while path.starts_with('/') {
        path = path[1..].to_string();
    }

    path
}

fn get_temp_dir_name() -> String {
    let random_id: String = iter::repeat_with(fastrand::alphanumeric).take(8).collect();
    format!("fs-fixture-{}", random_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_path() {
        assert_eq!(clean_path("./foo.txt"), "foo.txt");
        assert_eq!(clean_path("../foo.txt"), "foo.txt");
        assert_eq!(clean_path("foo.txt"), "foo.txt");
        assert_eq!(clean_path("dir/foo.txt"), "dir/foo.txt");
        assert_eq!(clean_path("dir/../foo.txt"), "dir/foo.txt");
        assert_eq!(clean_path("dir/./foo.txt"), "dir/foo.txt");
        assert_eq!(clean_path("/foo.txt"), "foo.txt");
        assert_eq!(clean_path("/dir/foo.txt"), "dir/foo.txt");
        assert_eq!(clean_path("/dir/../foo.txt"), "dir/foo.txt");
        assert_eq!(clean_path("/dir/./foo.txt"), "dir/foo.txt");
        assert_eq!(clean_path("dir/"), "dir/");
        assert_eq!(clean_path("/dir/"), "dir/");
        assert_eq!(clean_path("/dir/../"), "dir/");
        assert_eq!(clean_path("dir/./"), "dir/");
        assert_eq!(clean_path("./dir/"), "dir/");
        assert_eq!(clean_path("../dir/"), "dir/");
        assert_eq!(clean_path("dir///////foo//////bar"), "dir/foo/bar");
        assert_eq!(clean_path("dir/.././foo/.../bar"), "dir/foo/.../bar");
    }
}
