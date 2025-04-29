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

enum FileValue {
    File(String),
    Dir,
    SymlinkFile(String),
    SymlinkDir(String),
}

trait FileTreeBuilder {
    fn get_files_vec(&mut self) -> &mut Vec<(String, FileValue)>;
    fn get_prefix(&self) -> String;

    fn get_path(&self, path: &str) -> String {
        format!("{}{}", self.get_prefix(), &clean_path(path))
    }

    fn add_file(&mut self, path: &str, content: &str) {
        let path = self.get_path(path);
        self.get_files_vec()
            .push((path, FileValue::File(content.to_string())));
    }

    fn add_dir(&mut self, path: &str, cb: impl FnOnce(FsFixtureDirBuilder) -> FsFixtureDirBuilder) {
        let path = self.get_path(path);
        let files = self.get_files_vec();
        let start_files_len = files.len();

        let builder = FsFixtureDirBuilder::new(files, &path);
        let _ = cb(builder);

        if files.len() == start_files_len {
            // No new files added, push the directory entry only so it's still created
            files.push((path, FileValue::Dir));
        }
    }

    fn add_symlink_file(&mut self, path: &str, target: &str) {
        let path = self.get_path(path);
        self.get_files_vec()
            .push((path, FileValue::SymlinkFile(target.to_string())));
    }

    fn add_symlink_dir(&mut self, path: &str, target: &str) {
        let dir_path = self.get_path(path);
        self.get_files_vec()
            .push((dir_path, FileValue::SymlinkDir(target.to_string())));
    }
}

pub struct FsFixtureBuilder {
    files: Vec<(String, FileValue)>,
    options: FsFixtureBuilderOptions,
}
impl FileTreeBuilder for FsFixtureBuilder {
    fn get_files_vec(&mut self) -> &mut Vec<(String, FileValue)> {
        &mut self.files
    }

    fn get_prefix(&self) -> String {
        "".to_string()
    }
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

    /// Creates a file
    pub fn file(mut self, path: &str, content: &str) -> Self {
        self.add_file(path, content);
        self
    }

    /// Creates a directory and receives a callback to create more files or directories within it
    pub fn dir(
        mut self,
        path: &str,
        cb: impl FnOnce(FsFixtureDirBuilder) -> FsFixtureDirBuilder,
    ) -> Self {
        self.add_dir(path, cb);
        self
    }

    /// Creates a symlink to a file
    pub fn symlink_file(mut self, path: &str, target: &str) -> Self {
        self.add_symlink_file(path, target);
        self
    }

    /// Creates a symlink to a directory
    pub fn symlink_dir(mut self, path: &str, target: &str) -> Self {
        self.add_symlink_dir(path, target);
        self
    }

    pub fn build(self) -> io::Result<FsFixture> {
        let temp_dir = get_temp_dir_name();
        let resolved_temp_dir = self.options.temp_dir.join(temp_dir);

        if self.files.is_empty() {
            // If there's no files, we should still create the directory
            fs::create_dir_all(&resolved_temp_dir)?;
        } else {
            for (path, value) in self.files {
                let full_path = resolved_temp_dir.join(&path);
                match value {
                    FileValue::File(content) => {
                        fs::create_dir_all(full_path.parent().unwrap())?;
                        fs::write(full_path, content)?;
                    }
                    FileValue::Dir => {
                        fs::create_dir_all(full_path)?;
                    }
                    FileValue::SymlinkFile(target) => {
                        let target = resolved_temp_dir.join(&target);
                        fs::create_dir_all(full_path.parent().unwrap())?;
                        symlink_file(&target, &full_path)?;
                    }
                    FileValue::SymlinkDir(target) => {
                        let target = resolved_temp_dir.join(&target);
                        fs::create_dir_all(full_path.parent().unwrap())?;
                        symlink_dir(&target, &full_path)?;
                    }
                }
            }
        }

        Ok(FsFixture::new(resolved_temp_dir))
    }
}

pub struct FsFixtureDirBuilder<'a> {
    files: &'a mut Vec<(String, FileValue)>,
    dir: &'a str,
}
impl<'a> FileTreeBuilder for FsFixtureDirBuilder<'a> {
    fn get_files_vec(&mut self) -> &mut Vec<(String, FileValue)> {
        self.files
    }

    fn get_prefix(&self) -> String {
        format!("{}/", self.dir)
    }
}
impl<'a> FsFixtureDirBuilder<'a> {
    fn new(files: &'a mut Vec<(String, FileValue)>, dir: &'a str) -> Self {
        FsFixtureDirBuilder { files, dir }
    }

    /// Creates a file
    pub fn file(mut self, path: &str, content: &str) -> Self {
        self.add_file(path, content);
        self
    }

    /// Creates a directory and receives a callback to create more files or directories within it
    pub fn dir(
        mut self,
        path: &str,
        cb: impl FnOnce(FsFixtureDirBuilder) -> FsFixtureDirBuilder,
    ) -> Self {
        self.add_dir(path, cb);
        self
    }

    /// Creates a symlink to a file
    pub fn symlink_file(mut self, path: &str, target: &str) -> Self {
        self.add_symlink_file(path, target);
        self
    }

    /// Creates a symlink to a directory
    pub fn symlink_dir(mut self, path: &str, target: &str) -> Self {
        self.add_symlink_dir(path, target);
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

    while path.ends_with('/') {
        path = path[0..path.len() - 1].to_string();
    }

    path
}

fn get_temp_dir_name() -> String {
    let random_id: String = iter::repeat_with(fastrand::alphanumeric).take(8).collect();
    format!("fs-fixture-{}", random_id)
}

fn symlink_file(original: &Path, link: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(original, link)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_file;
        symlink_file(original, link)
    }
}

fn symlink_dir(original: &Path, link: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(original, link)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::symlink_dir;
        symlink_dir(original, link)
    }
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
        assert_eq!(clean_path("dir/"), "dir");
        assert_eq!(clean_path("/dir/"), "dir");
        assert_eq!(clean_path("/dir/../"), "dir");
        assert_eq!(clean_path("dir/./"), "dir");
        assert_eq!(clean_path("./dir/"), "dir");
        assert_eq!(clean_path("../dir/"), "dir");
        assert_eq!(clean_path("dir///////foo//////bar"), "dir/foo/bar");
        assert_eq!(clean_path("dir/.././foo/.../bar"), "dir/foo/.../bar");
    }
}
