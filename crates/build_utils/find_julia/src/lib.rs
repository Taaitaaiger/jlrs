//! Detects where Julia is installed, what version is used, and whether BinaryBuilder is used.

#[cfg(target_os = "windows")]
use std::str::FromStr;
use std::{env, io::Read, path::PathBuf, process::Command};

/// Detected Julia version
#[derive(Clone)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    is_dev: bool,
}

impl Version {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32, is_dev: bool) -> Self {
        Version {
            major,
            minor,
            patch,
            is_dev,
        }
    }

    /// The major version
    pub fn major(&self) -> u32 {
        self.major
    }

    /// The minor version
    pub fn minor(&self) -> u32 {
        self.minor
    }

    /// The patch version
    pub fn patch(&self) -> u32 {
        self.patch
    }

    /// Is the version a dev build or not
    pub fn is_dev(&self) -> bool {
        self.is_dev
    }

    /// Load the version detected when jl-sys was built.
    pub fn from_detected() -> Option<Self> {
        let version = env::var("DEP_JULIA_VERSION").ok()?;
        let dev = env::var("DEP_JULIA_IS_DEV").ok()?;

        let mut parts = version.split('.');
        let major: u32 = parts.next()?.parse().ok()?;
        let minor: u32 = parts.next()?.parse().ok()?;
        let patch: u32 = parts.next()?.parse().ok()?;
        let is_dev: u32 = dev.parse().ok()?;

        Some(Version {
            major,
            minor,
            patch,
            is_dev: is_dev != 0,
        })
    }

    /// Emit detected version metadata.
    pub fn emit_metadata_unchecked(&self) {
        let major = self.major();
        let minor = self.minor();
        let patch = self.patch();
        let is_dev = if self.is_dev() { 1 } else { 0 };
        println!("cargo::metadata=version={major}.{minor}.{patch}");
        println!("cargo::metadata=is_dev={is_dev}");
        println!("cargo::rustc-cfg=julia_{major}_{minor}");
    }

    /// Detect the installed version of julia at `julia_dir`.
    fn detect(mut julia_dir: PathBuf) -> Self {
        julia_dir.push("include/julia/julia_version.h");

        let mut julia_version_file =
            std::fs::File::open(&julia_dir).expect("Cannot find julia_version.h");

        let mut buf = Vec::new();
        julia_version_file
            .read_to_end(&mut buf)
            .expect("Cannot read julia_version.h");

        let julia_version_content = String::from_utf8_lossy(&buf);
        let mut major = -1;
        let mut minor = -1;
        let mut patch = -1;
        let mut is_dev = false;

        for line in julia_version_content.lines() {
            if let Some(m) = line.strip_prefix("#define JULIA_VERSION_STRING ") {
                is_dev = m.contains("DEV");
                continue;
            }
            if let Some(m) = line.strip_prefix("#define JULIA_VERSION_MAJOR ") {
                major = m.parse().unwrap();
                continue;
            }
            if let Some(m) = line.strip_prefix("#define JULIA_VERSION_MINOR ") {
                minor = m.parse().unwrap();
                continue;
            }
            if let Some(m) = line.strip_prefix("#define JULIA_VERSION_PATCH ") {
                patch = m.parse().unwrap();
                continue;
            }
        }

        if major == -1 || minor == -1 || patch == -1 {
            panic!(
                "Could not extract julia version from {}",
                julia_dir.display()
            );
        }

        Version {
            major: major as _,
            minor: minor as _,
            patch: patch as _,
            is_dev,
        }
    }

    fn emit_metadata(&self, min_minor_version: u32, max_minor_version: u32) {
        let major = self.major();
        let minor = self.minor();
        let patch = self.patch();

        if major != 1 {
            panic!("Detected unsupported version of Julia {major}.{minor}.{patch}.");
        }

        if self.is_dev() {
            println!(
                "cargo::warning=Detected development version of Julia {major}.{minor}.{patch}, \
            bindings may not be up-to-date. Please report any issues you encounter at \
            https://www.github.com/Taaitaaiger/jlrs/issues"
            );
        }

        if minor > max_minor_version {
            println!(
                "cargo::warning=Detected unsupported version of Julia {major}.{minor}.{patch}, \
                    assuming compatibility with 1.{max_minor_version}. Please report any issues you
                    encounter at https://www.github.com/Taaitaaiger/jlrs/issues"
            );

            let mut version = self.clone();
            version.minor = max_minor_version;
            version.patch = 0;
            version.emit_metadata_unchecked();
        } else if minor < min_minor_version {
            panic!(
                "Detected unsupported version of Julia {major}.{minor}.{patch}. Minimum supported version is 1.{min_minor_version}"
            );
        } else {
            // Supported version
            self.emit_metadata_unchecked();
        }
    }
}

/// Julia root directory
pub struct JuliaDir {
    is_binary_builder: bool,
    path: PathBuf,
    version: Version,
}

impl JuliaDir {
    /// Find the Julia root directory.
    ///
    /// Defaults to the value of the environment variable `JLRS_JULIA_DIR` if it is set, otherwise
    /// `which julia` or `where julia` is used to find this directory.
    ///
    /// If `bb_target` and `WORKSPACE` have been set, it is assumed that BinaryBuilder is used.
    pub fn find() -> Option<Self> {
        let is_bb = building_in_binary_builder();
        if is_bb {
            let path = binary_builer_julia_dir();
            let version = Version::detect(path.clone());
            Some(JuliaDir {
                is_binary_builder: is_bb,
                path,
                version,
            })
        } else {
            let path = installed_julia_dir()?;
            let version = Version::detect(path.clone());
            Some(JuliaDir {
                is_binary_builder: is_bb,
                path,
                version,
            })
        }
    }

    /// The version of this Julia installation
    pub fn version(&self) -> Version {
        self.version.clone()
    }

    /// The `lib` directory of this Julia installation
    pub fn lib_dir(&self) -> PathBuf {
        let mut julia_dir = self.path.clone();
        julia_dir.push("lib");
        julia_dir
    }

    /// The `bin` directory of this Julia installation
    pub fn bin_dir(&self) -> PathBuf {
        let mut julia_dir = self.path.clone();
        julia_dir.push("bin");
        julia_dir
    }

    /// The `include/julia` directory of this Julia installation
    pub fn include_dir(&self) -> PathBuf {
        let mut julia_dir = self.path.clone();
        julia_dir.push("include/julia");
        julia_dir
    }

    /// Instruct `rustc` to link this installation of Julia
    pub fn link(&self) {
        let lib_dir = self.lib_dir();
        println!("cargo::rustc-link-search={}", lib_dir.display());
        println!("cargo::rustc-link-lib=julia");
    }

    /// Whether or not BinaryBuilder is currently used
    ///
    /// If `bb_target` and `WORKSPACE` have been set, it is assumed that BinaryBuilder is used.
    pub fn is_binary_builder(&self) -> bool {
        self.is_binary_builder
    }

    /// Emit detected Julia installation metadata.
    pub fn emit_metadata(&self, min_minor_version: u32, max_minor_version: u32) {
        println!("cargo::metadata=julia_dir={}", self.path.display());
        self.version
            .emit_metadata(min_minor_version, max_minor_version);
    }

    /// Load the installation detected when jl-sys was built.
    pub fn from_detected() -> Option<Self> {
        let version = Version::from_detected()?;

        if building_in_binary_builder() {
            let path = binary_builer_julia_dir();
            Some(JuliaDir {
                is_binary_builder: true,
                path,
                version,
            })
        } else {
            let path = PathBuf::from(env::var("DEP_JULIA_JULIA_DIR").ok()?);
            Some(JuliaDir {
                is_binary_builder: false,
                path,
                version,
            })
        }
    }
}

/// Enable the `julia_1_x` version configs from `min_version..=max_version`.
pub fn enable_version_cfgs(min_version: u32, max_version: u32) {
    let versions: Vec<String> = (min_version..=max_version)
        .map(|minor| format!("julia_1_{minor}"))
        .collect();
    let versions_joined = versions.join(",");
    println!("cargo::rustc-check-cfg=cfg({versions_joined})");
}

fn building_in_binary_builder() -> bool {
    env::var_os("bb_target").is_some() && env::var_os("WORKSPACE").is_some()
}

fn binary_builer_julia_dir() -> PathBuf {
    let mut path = PathBuf::from(env::var_os("WORKSPACE").unwrap());
    path.push("destdir");
    path
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
fn installed_julia_dir() -> Option<PathBuf> {
    use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

    if let Some(path) = env::var_os("JLRS_JULIA_DIR") {
        return Some(PathBuf::from(path));
    }

    let out = Command::new("which").arg("julia").output().ok()?.stdout;
    let mut julia_path = PathBuf::from(OsStr::from_bytes(out.as_ref()));

    if !julia_path.pop() {
        return None;
    }

    if !julia_path.pop() {
        return None;
    }

    Some(julia_path)
}

#[cfg(target_os = "windows")]
fn installed_julia_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("JLRS_JULIA_DIR") {
        return Some(PathBuf::from(path));
    }

    let out = Command::new("cmd")
        .args(["/C", "where", "julia"])
        .output()
        .ok()?
        .stdout;

    let out = Command::new("cmd")
        .args(["/C", "where", "julia"])
        .output()
        .ok()?;
    let results = String::from_utf8(out.stdout).ok()?;

    let mut lines = results.lines();
    let first = lines.next()?;

    let mut julia_path = PathBuf::from_str(first).unwrap();

    if !julia_path.pop() {
        return None;
    }

    if !julia_path.pop() {
        return None;
    }

    Some(julia_path)
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "windows"
)))]
fn installed_julia_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("JLRS_JULIA_DIR") {
        return Some(PathBuf::from(path));
    }

    unimplemented!(
        "Julia detection not implemented for this platform, set the JLRS_JULIA_DIR environment variable"
    )
}
