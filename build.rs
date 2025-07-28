use std::{
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};

use anyhow::{anyhow, bail, Context, Result};

/// Finds the location of an executable `name`.
#[must_use]
pub fn find_executable(name: &str) -> Option<PathBuf> {
    const WHICH: &str = if cfg!(windows) { "where" } else { "which" };
    let cmd = Command::new(WHICH).arg(name).output().ok()?;
    if cmd.status.success() {
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        stdout.trim().lines().next().map(|l| l.trim().into())
    } else {
        None
    }
}

/// Returns an environment variable's value as a `PathBuf`
pub fn path_from_env(key: &str) -> Option<PathBuf> {
    std::env::var_os(key).map(PathBuf::from)
}

/// Finds the location of the PHP executable.
fn find_php() -> Result<PathBuf> {
    // If path is given via env, it takes priority.
    if let Some(path) = path_from_env("PHP") {
        if !path.try_exists()? {
            // If path was explicitly given and it can't be found, this is a hard error
            bail!("php executable not found at {:?}", path);
        }
        return Ok(path);
    }
    find_executable("php").with_context(|| {
        "Could not find PHP executable. \
        Please ensure `php` is in your PATH or the `PHP` environment variable is set."
    })
}

/// Output of `php -i`.
pub struct PHPInfo(String);

impl PHPInfo {
    /// Get the PHP info.
    ///
    /// # Errors
    /// - `php -i` command failed to execute successfully
    pub fn get(php: &Path) -> Result<Self> {
        let cmd = Command::new(php)
            .arg("-i")
            .output()
            .context("Failed to call `php -i`")?;
        if !cmd.status.success() {
            bail!("Failed to call `php -i` status code {}", cmd.status);
        }
        let stdout = String::from_utf8_lossy(&cmd.stdout);
        Ok(Self(stdout.to_string()))
    }

    /// Get the zend version.
    ///
    /// # Errors
    /// - `PHPInfo` does not contain php api version
    pub fn zend_version(&self) -> Result<u32> {
        self.get_key("PHP API")
            .context("Failed to get Zend version")
            .and_then(|s| u32::from_str(s).context("Failed to convert Zend version to integer"))
    }

    fn get_key(&self, key: &str) -> Option<&str> {
        let split = format!("{key} => ");
        for line in self.0.lines() {
            let components: Vec<_> = line.split(&split).collect();
            if components.len() > 1 {
                return Some(components[1]);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
enum ApiVersion {
    Php80 = 2020_09_30,
    Php81 = 2021_09_02,
    Php82 = 2022_08_29,
    Php83 = 2023_08_31,
    Php84 = 2024_09_24,
}

impl ApiVersion {
    /// Returns the minimum API version supported.
    pub const fn min() -> Self {
        ApiVersion::Php80
    }

    /// Returns the maximum API version supported.
    pub const fn max() -> Self {
        ApiVersion::Php84
    }

    pub fn versions() -> Vec<Self> {
        vec![
            ApiVersion::Php80,
            ApiVersion::Php81,
            ApiVersion::Php82,
            ApiVersion::Php83,
            ApiVersion::Php84,
        ]
    }

    /// Returns the API versions that are supported by this version.
    pub fn supported_apis(self) -> Vec<ApiVersion> {
        ApiVersion::versions()
            .into_iter()
            .filter(|&v| v <= self)
            .collect()
    }

    pub fn cfg_name(self) -> &'static str {
        match self {
            ApiVersion::Php80 => "php80",
            ApiVersion::Php81 => "php81",
            ApiVersion::Php82 => "php82",
            ApiVersion::Php83 => "php83",
            ApiVersion::Php84 => "php84",
        }
    }
}

impl TryFrom<u32> for ApiVersion {
    type Error = anyhow::Error;

    fn try_from(version: u32) -> Result<Self, Self::Error> {
        match version {
            x if ((ApiVersion::Php80 as u32)..(ApiVersion::Php81 as u32)).contains(&x) => Ok(ApiVersion::Php80),
            x if ((ApiVersion::Php81 as u32)..(ApiVersion::Php82 as u32)).contains(&x) => Ok(ApiVersion::Php81),
            x if ((ApiVersion::Php82 as u32)..(ApiVersion::Php83 as u32)).contains(&x) => Ok(ApiVersion::Php82),
            x if ((ApiVersion::Php83 as u32)..(ApiVersion::Php84 as u32)).contains(&x) => Ok(ApiVersion::Php83),
            x if (ApiVersion::Php84 as u32) == x => Ok(ApiVersion::Php84),
            version => Err(anyhow!(
              "The current version of PHP is not supported. Current PHP API version: {}, requires a version between {} and {}",
              version,
              ApiVersion::min() as u32,
              ApiVersion::max() as u32
            ))
        }
    }
}

/// Checks the PHP Zend API version for compatibility, setting
/// any configuration flags required.
fn check_php_version(info: &PHPInfo) -> Result<()> {
    let version = info.zend_version()?;
    let version: ApiVersion = version.try_into()?;

    // Set up cfg flags like ext-php-rs does
    println!("cargo::rustc-check-cfg=cfg(php80, php81, php82, php83, php84)");

    if version == ApiVersion::Php80 {
        println!("cargo:warning=PHP 8.0 is EOL and will no longer be supported in a future release. Please upgrade to a supported version of PHP. See https://www.php.net/supported-versions.php for information on version support timelines.");
    }

    // Set cfg flags for all supported versions up to the current one
    for supported_version in version.supported_apis() {
        println!("cargo:rustc-cfg={}", supported_version.cfg_name());
    }

    Ok(())
}

fn main() -> Result<()> {
    // Rerun if PHP changes
    for env_var in ["PHP", "PHP_CONFIG", "PATH"] {
        println!("cargo:rerun-if-env-changed={env_var}");
    }

    println!("cargo:rerun-if-changed=build.rs");

    let php = find_php()?;
    let info = PHPInfo::get(&php)?;

    check_php_version(&info)?;

    Ok(())
}
