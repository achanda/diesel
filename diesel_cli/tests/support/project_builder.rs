#[cfg(not(feature = "sqlite"))]
extern crate dotenvy;
#[cfg(not(feature = "sqlite"))]
extern crate url;

use std::fs::{self, File, ReadDir};
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

use super::command::TestCommand;

pub fn project(name: &str) -> ProjectBuilder {
    ProjectBuilder::new(name)
}

pub struct ProjectBuilder {
    name: String,
    folders: Vec<String>,
    files: Vec<(PathBuf, String)>,
}

impl ProjectBuilder {
    fn new(name: &str) -> Self {
        ProjectBuilder {
            name: name.into(),
            folders: Vec::new(),
            files: Vec::new(),
        }
    }

    pub fn folder(mut self, name: &str) -> Self {
        self.folders.push(name.into());
        self
    }

    pub fn file(mut self, name: &str, contents: &str) -> Self {
        self.files.push((name.into(), contents.into()));
        self
    }

    pub fn build(self) -> Project {
        let tempdir = Builder::new().prefix(&self.name).tempdir().unwrap();

        File::create(tempdir.path().join("Cargo.toml")).unwrap();

        for folder in self.folders {
            fs::create_dir_all(tempdir.path().join(folder)).unwrap();
        }

        for (file, contents) in self.files {
            fs::File::create(tempdir.path().join(file))
                .unwrap()
                .write_all(contents.as_bytes())
                .unwrap()
        }

        Project {
            directory: tempdir,
            name: self.name,
            skip_drop_db: false,
        }
    }
}

pub struct Project {
    directory: TempDir,
    pub name: String,
    skip_drop_db: bool,
}

impl Project {
    pub fn command(&self, name: &str) -> TestCommand {
        self.command_without_database_url(name)
            .env("DATABASE_URL", &self.database_url())
    }

    pub fn command_without_database_url(&self, name: &str) -> TestCommand {
        TestCommand::new(self.directory.path(), name)
    }

    pub fn migrations(&self) -> Vec<Migration> {
        self.directory
            .path()
            .join("migrations")
            .read_dir()
            .expect("Error reading directory")
            .filter_map(|e| {
                if let Ok(e) = e {
                    if e.path().is_dir() {
                        return Some(Migration { path: e.path() });
                    }
                }
                None
            })
            .collect()
    }

    pub fn delete_single_file<P: AsRef<Path>>(&self, path: P) {
        let file = self.directory.path().join(path);
        fs::remove_file(file).unwrap();
    }

    #[cfg(any(feature = "postgres", feature = "mysql"))]
    fn database_url_from_env(&self, var: &str) -> url::Url {
        use self::dotenvy::dotenv;
        use std::env;
        dotenv().ok();

        let var_os = env::var(var).or_else(|_| env::var("DATABASE_URL")).unwrap();
        let mut db_url = url::Url::parse(&var_os).unwrap();
        db_url.set_path(&format!("/diesel_{}", &self.name));
        db_url
    }

    #[cfg(feature = "postgres")]
    pub fn database_url(&self) -> String {
        self.database_url_from_env("PG_DATABASE_URL").to_string()
    }

    #[cfg(feature = "mysql")]
    pub fn database_url(&self) -> String {
        self.database_url_from_env("MYSQL_DATABASE_URL").to_string()
    }

    #[cfg(feature = "sqlite")]
    pub fn database_url(&self) -> String {
        self.directory
            .path()
            .join(&self.name)
            .into_os_string()
            .into_string()
            .unwrap()
    }

    pub fn has_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.directory.path().join(path).exists()
    }

    pub fn directory_entries<P: AsRef<Path>>(&self, path: P) -> Result<ReadDir, io::Error> {
        fs::read_dir(self.directory.path().join(path))
    }

    pub fn file_contents<P: AsRef<Path>>(&self, path: P) -> String {
        let mut f = File::open(self.directory.path().join(path)).expect("Could not open file");
        let mut result = String::new();
        f.read_to_string(&mut result).expect("Could not read file");
        result
    }

    #[cfg(feature = "postgres")]
    pub fn delete_file<P: AsRef<Path>>(&self, path: P) {
        let file = self.directory.path().join(path);
        fs::remove_dir_all(file).unwrap();
    }

    pub fn migration_dir_in_directory(&self, directory: &str) -> String {
        let migration_path = self.directory.path().join(directory);
        migration_path.display().to_string()
    }

    pub fn create_migration(&self, name: &str, up: &str, down: Option<&str>, config: Option<&str>) {
        self.create_migration_in_directory("migrations", name, up, down, config);
    }

    pub fn create_migration_in_directory(
        &self,
        directory: &str,
        name: &str,
        up: &str,
        down: Option<&str>,
        config: Option<&str>,
    ) {
        let migration_path = self.directory.path().join(directory).join(name);
        fs::create_dir(&migration_path)
            .expect("Migrations folder must exist to create a migration");
        let mut up_file = fs::File::create(migration_path.join("up.sql")).unwrap();
        up_file.write_all(up.as_bytes()).unwrap();

        if let Some(down) = down {
            let mut down_file = fs::File::create(migration_path.join("down.sql")).unwrap();
            down_file.write_all(down.as_bytes()).unwrap();
        }

        if let Some(config) = config {
            let mut metadata_file = fs::File::create(migration_path.join("metadata.toml")).unwrap();
            metadata_file.write_all(config.as_bytes()).unwrap();
        }
    }

    pub fn skip_drop_db(&mut self) {
        self.skip_drop_db = true;
    }
}

#[cfg(not(feature = "sqlite"))]
impl Drop for Project {
    fn drop(&mut self) {
        if !self.skip_drop_db {
            try_drop!(
                self.command("database").arg("drop").run().result(),
                "Couldn't drop database"
            );
        }
    }
}

pub struct Migration {
    path: PathBuf,
}

impl Migration {
    pub fn name(&self) -> &str {
        let name_start_index = self.file_name().find('_').unwrap() + 1;
        &self.file_name()[name_start_index..]
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn file_name(&self) -> &str {
        self.path
            .file_name()
            .expect("migration should have a file name")
            .to_str()
            .expect("Directory was not valid UTF-8")
    }
}
