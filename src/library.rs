use crate::{config::Config, util::ChadError};

#[cfg(feature = "database")]
use crate::database::DatabaseFetcher;

use futures::future::join_all;
use serde::Serialize;
use std::{
    io::Read,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use titlecase::titlecase;

#[derive(Serialize, Clone, Debug)]
pub struct Game {
    /// Unique identifier, usually the index of the list that contains this `Game`
    pub id: usize,
    /// Name of the game
    pub name: String,
    /// Path to the directory that contains the executable start scripts
    pub executable_dir: PathBuf,
    /// List of start scripts
    pub scripts: Vec<Script>,
    /// Path to a banner image
    pub banner_path: Option<PathBuf>,
    /// base64 encoded data blob: `data:image/png;base64,<base64 string>`
    pub banner: Option<String>,
    /// Directory where extra metadata is stored
    pub data_path: PathBuf,
    /// Path to the log file
    pub log_file: PathBuf,
    /// Path to the configuration file
    pub config_file: PathBuf,
}

#[derive(Serialize, Clone, Debug)]
pub struct Script {
    pub name: String,
    pub script: String,
}

fn load_banner(banner_path: &Path) -> Option<String> {
    std::fs::read(banner_path)
        .ok()
        .map(|b| base64::encode(b))
        .map(|b64| format!("data:image/png;base64,{}", b64))
}

fn prettify_slug(slug: &str) -> String {
    let mut name = slug.replace(".", " ");
    name = name.replace("_", " ");
    name = name.replace("-", " ");
    name = titlecase(&name).trim().into();
    name
}

fn script_name(script_file: &str) -> String {
    if script_file == "start" || script_file == "start.sh" {
        String::from("Start")
    } else {
        let mut res = script_file.strip_prefix("start").unwrap_or(script_file);
        res = res.strip_suffix("sh").unwrap_or(res);
        prettify_slug(res)
    }
}

fn is_start_script(e: &std::fs::DirEntry, blacklist: &[String]) -> bool {
    // Only check files
    let is_file = e
        .file_type()
        .map(|f| f.is_file() || f.is_symlink())
        .unwrap_or(false);
    // Find executable files
    let is_executable = std::fs::metadata(e.path())
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false);
    // Find valid scripts
    let is_valid = e
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name_str| !blacklist.contains(&name_str.into()))
        .unwrap_or(false);
    is_file && is_executable && is_valid
}

fn find_scripts(executable_dir: &Path, blacklist: &[String]) -> Result<Vec<Script>, ChadError> {
    Ok(executable_dir
        // Try to read the directory
        .read_dir()?
        // Filter out errors
        .filter_map(|e| e.ok())
        // Only check files
        .filter(|e| is_start_script(e, &blacklist))
        // Map DirEntry to String
        .filter_map(|d| d.file_name().to_str().map(|s| s.to_string()))
        .map(|script| Script {
            name: script_name(&script),
            script,
        })
        // Collect into a Vec
        .collect())
}

impl Game {
    /// Creates a new `Game` from the given configuration with the given id and path to the
    /// directory that contains the executables of this game.
    pub fn new(config: &Config, id: usize, executable_dir: PathBuf) -> Self {
        let slug: String = executable_dir.file_name().unwrap().to_str().unwrap().into();
        let name = prettify_slug(&slug);

        let data_path = config.data_path().join("library").join(slug);
        let _ = std::fs::create_dir_all(&data_path);

        let banner_path = if data_path.join("banner.png").exists() {
            Some(data_path.join("banner.png"))
        } else {
            None // TODO Fetch banner
        };

        let banner = banner_path.as_ref().and_then(|p| load_banner(&p));

        let config_file = data_path.join("game.yaml");
        let log_file = executable_dir.join("chad.log");
        let scripts = find_scripts(&executable_dir, &config.script_blacklist).unwrap_or(Vec::new());

        Self {
            id,
            name,
            executable_dir,
            scripts,
            banner_path,
            banner,
            data_path,
            log_file,
            config_file,
        }
    }

    pub fn executable_dir(&self) -> &Path {
        &self.executable_dir
    }

    /// Uses the given `DatabaseFetcher` to find a matching banner for the game
    #[cfg(feature = "database")]
    pub async fn get_banner(&mut self, fetcher: &DatabaseFetcher) -> Result<(), ChadError> {
        if let Ok(banner_path) = fetcher.find_banner(&self.name).await {
            let target = format!(
                "https://gitlab.com/chad-productions/chad_launcher_banners/-/raw/master/{}",
                banner_path
            );
            let response = reqwest::get(target).await?;
            let content = response.text().await?;
            std::io::copy(
                &mut content.as_bytes(),
                &mut std::fs::File::create(self.data_path.join("banner.png"))?,
            )?;
            self.banner_path = Some(self.data_path.join("banner.png"));
            self.banner = self.banner_path.as_ref().and_then(|p| load_banner(&p));
        }

        Ok(())
    }

    /// Launches the given script. Returns the receiving end of the stdout from the child process.
    pub fn launch(&self, script: &str) -> Result<Box<dyn Read>, ChadError> {
        let child = Command::new(&self.executable_dir.join(&script))
            .current_dir(&self.executable_dir)
            .stdout(Stdio::piped())
            .spawn()?;
        Ok(Box::new(child.stdout.unwrap()))
    }
}

#[derive(Debug, Default)]
pub struct LibraryFetcher {
    games: Vec<Game>,
}

impl LibraryFetcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load games by scanning library paths. Ignores any game directory that contains a `.ignore` or
    /// `.chadignore` file. Ignores start scripts in `config.script_blacklist`.
    pub fn load_games(&mut self, config: &Config) {
        self.games = config
            // Iterate over all library paths
            .library_paths()
            .into_iter()
            // Read each library path
            .map(|lp| {
                if let Ok(dir) = lp.read_dir() {
                    Box::new(
                        dir
                            // Filter out any errors
                            .filter_map(|e| e.ok())
                            // Find all directories
                            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                            // Filter out ignored directories
                            .filter(|e| {
                                e.path()
                                    .read_dir()
                                    .map(|d| {
                                        d.filter_map(|f| f.ok()).all(|f| {
                                            f.path()
                                                .file_name()
                                                .map(|n| n != ".ignore" && n != ".chadignore")
                                                .unwrap_or(true)
                                        })
                                    })
                                    .unwrap_or(false)
                            })
                            // Find start scripts
                            .filter(|e| {
                                e.path()
                                    .read_dir()
                                    .map(|d| {
                                        d.filter_map(|f| f.ok())
                                            .any(|f| is_start_script(&f, &config.script_blacklist))
                                    })
                                    .unwrap_or(false)
                            }),
                    ) as Box<dyn Iterator<Item = std::fs::DirEntry>>
                } else {
                    Box::new(std::iter::empty())
                }
            })
            // Flatten those nested iterators into a single iterator
            .flatten()
            // Zip it with indices
            .zip(0..)
            // Create games
            .map(|(e, i)| Game::new(&config, i, e.path()))
            // Collect them into a vec
            .collect();
    }

    /// Downloads banners for each game
    #[cfg(feature = "database")]
    pub async fn download_banners(&mut self, fetcher: &DatabaseFetcher) {
        join_all(
            self.games
                .iter_mut()
                .filter(|g| g.banner == None)
                .map(|g| g.get_banner(&fetcher)),
        )
        .await;
    }

    pub fn iter_games<'a>(&'a self) -> impl Iterator<Item = &'a Game> {
        self.games.iter()
    }

    pub fn get_games<'a>(&'a self) -> Vec<&'a Game> {
        self.iter_games().collect()
    }

    pub fn get_games_cloned(&self) -> Vec<Game> {
        self.iter_games().cloned().collect()
    }

    /// Get a game from the given id
    pub fn get_game<'a>(&'a self, id: usize) -> Option<&'a Game> {
        self.games.get(id)
    }
}
