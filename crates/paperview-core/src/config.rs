use std::{
    env, fmt, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub theme: ThemePreference,
    #[serde(default)]
    pub zen_mode: bool,
    #[serde(default = "default_split_primary_width")]
    pub split_primary_width: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePreference {
    #[default]
    Hybrid,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            theme: ThemePreference::default(),
            zen_mode: false,
            split_primary_width: default_split_primary_width(),
        }
    }
}

fn default_schema_version() -> u32 {
    1
}

fn default_split_primary_width() -> u16 {
    50
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    #[must_use]
    pub fn default_path() -> PathBuf {
        if let Some(path) = env::var_os("PAPERVIEW_CONFIG_PATH") {
            return PathBuf::from(path);
        }

        default_config_dir().join("config.toml")
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Config, ConfigStoreError> {
        match fs::read_to_string(&self.path) {
            Ok(raw) => toml::from_str(&raw).map_err(|source| ConfigStoreError::DecodeFailed {
                path: self.path.clone(),
                source,
            }),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
            Err(source) => Err(ConfigStoreError::ReadFailed {
                path: self.path.clone(),
                source,
            }),
        }
    }

    pub fn save(&self, config: &Config) -> Result<(), ConfigStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigStoreError::CreateDirFailed {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let encoded =
            toml::to_string_pretty(config).map_err(|source| ConfigStoreError::EncodeFailed {
                path: self.path.clone(),
                source,
            })?;

        fs::write(&self.path, encoded).map_err(|source| ConfigStoreError::WriteFailed {
            path: self.path.clone(),
            source,
        })
    }

    pub fn ensure_exists(&self) -> Result<(), ConfigStoreError> {
        if self.path.exists() {
            return Ok(());
        }

        self.save(&Config::default())
    }
}

impl Default for ConfigStore {
    fn default() -> Self {
        Self::new(Self::default_path())
    }
}

#[derive(Debug)]
pub enum ConfigStoreError {
    ReadFailed {
        path: PathBuf,
        source: io::Error,
    },
    DecodeFailed {
        path: PathBuf,
        source: toml::de::Error,
    },
    CreateDirFailed {
        path: PathBuf,
        source: io::Error,
    },
    EncodeFailed {
        path: PathBuf,
        source: toml::ser::Error,
    },
    WriteFailed {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for ConfigStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFailed { path, source } => {
                write!(
                    formatter,
                    "failed to read config {}: {source}",
                    path.display()
                )
            }
            Self::DecodeFailed { path, source } => {
                write!(
                    formatter,
                    "failed to decode config {}: {source}",
                    path.display()
                )
            }
            Self::CreateDirFailed { path, source } => write!(
                formatter,
                "failed to create config directory {}: {source}",
                path.display()
            ),
            Self::EncodeFailed { path, source } => {
                write!(
                    formatter,
                    "failed to encode config {}: {source}",
                    path.display()
                )
            }
            Self::WriteFailed { path, source } => {
                write!(
                    formatter,
                    "failed to write config {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for ConfigStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFailed { source, .. }
            | Self::CreateDirFailed { source, .. }
            | Self::WriteFailed { source, .. } => Some(source),
            Self::DecodeFailed { source, .. } => Some(source),
            Self::EncodeFailed { source, .. } => Some(source),
        }
    }
}

fn default_config_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        home_dir()
            .map(|home| home.join("Library/Application Support/PaperView"))
            .unwrap_or_else(fallback_config_dir)
    } else if cfg!(target_os = "windows") {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|path| path.join("PaperView"))
            .unwrap_or_else(fallback_config_dir)
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|home| home.join(".config")))
            .map(|path| path.join("paperview"))
            .unwrap_or_else(fallback_config_dir)
    }
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

fn fallback_config_dir() -> PathBuf {
    env::temp_dir().join("paperview")
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{Config, ConfigStore, ThemePreference};

    #[test]
    fn missing_config_loads_default() {
        let store = ConfigStore::new(temp_path("missing/config.toml"));

        assert_eq!(
            store.load().expect("load missing config"),
            Config::default()
        );
    }

    #[test]
    fn saves_and_loads_config_file() {
        let path = temp_path("nested/config.toml");
        let store = ConfigStore::new(&path);
        let config = Config {
            schema_version: 7,
            theme: ThemePreference::Hybrid,
            zen_mode: true,
            split_primary_width: 60,
        };

        store.save(&config).expect("save config");

        assert_eq!(store.load().expect("load config"), config);

        fs::remove_file(path).expect("remove config");
    }

    #[test]
    fn serializes_theme_preference_as_kebab_case() {
        let encoded = toml::to_string(&Config::default()).expect("encode config");

        assert!(encoded.contains("theme = \"hybrid\""));
    }

    #[test]
    fn rejects_unknown_theme_preference() {
        let path = temp_path("theme/config.toml");
        let store = ConfigStore::new(&path);
        fs::create_dir_all(path.parent().expect("config parent")).expect("create config parent");
        fs::write(
            &path,
            "schema_version = 1\ntheme = \"solarized\"\nsplit_primary_width = 50\n",
        )
        .expect("write invalid config");

        assert!(store.load().is_err());

        fs::remove_file(path).expect("remove config");
    }

    #[test]
    fn loads_missing_settings_from_defaults() {
        let path = temp_path("partial/config.toml");
        let store = ConfigStore::new(&path);
        fs::create_dir_all(path.parent().expect("config parent")).expect("create config parent");
        fs::write(&path, "schema_version = 1\n").expect("write partial config");

        assert_eq!(store.load().expect("load config"), Config::default());

        fs::remove_file(path).expect("remove config");
    }

    #[test]
    fn ensure_exists_writes_default_config() {
        let path = temp_path("ensure/config.toml");
        let store = ConfigStore::new(&path);

        store.ensure_exists().expect("ensure config exists");

        assert_eq!(store.load().expect("load config"), Config::default());

        fs::remove_file(path).expect("remove config");
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();

        env::temp_dir().join(format!("paperview-config-{nanos}-{name}"))
    }
}
