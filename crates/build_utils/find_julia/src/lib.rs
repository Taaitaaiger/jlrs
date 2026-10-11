//! Detects where Julia is installed, what version is used, and whether BinaryBuilder is used.

#[cfg(target_os = "windows")]
use std::str::FromStr;
use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

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
}

/// Julia root directory
pub struct JuliaInfo {
    is_binary_builder: bool,
    windows: bool,
    path: PathBuf,
    version: Version,
    debug: bool,
    docs: bool,
}

impl JuliaInfo {
    /// Find the Julia root directory.
    ///
    /// Defaults to the value of the environment variable `JLRS_JULIA_DIR` if it is set, otherwise
    /// `which julia` or `where julia` is used to find this directory.
    ///
    /// If `bb_target` and `WORKSPACE` have been set, it is assumed that BinaryBuilder is used.
    pub fn find(windows: bool, debug: bool) -> Option<Self> {
        let is_bb = building_in_binary_builder();
        if is_bb {
            let path = binary_builer_julia_dir();
            let version = Version::detect(path.clone());
            Some(JuliaInfo {
                windows,
                is_binary_builder: is_bb,
                path,
                version,
                debug,
                docs: false,
            })
        } else {
            let path = installed_julia_dir()?;
            let version = Version::detect(path.clone());
            Some(JuliaInfo {
                windows,
                is_binary_builder: is_bb,
                path,
                version,
                debug,
                docs: false,
            })
        }
    }

    pub fn docs(stable_major: u32, stable_minor: u32) -> Self {
        JuliaInfo {
            is_binary_builder: false,
            windows: false,
            path: PathBuf::new(),
            version: Version::new(stable_major, stable_minor, 0, false),
            debug: false,
            docs: true,
        }
    }

    pub fn write_metadata_file(
        &self,
        path: &Path,
        lts_version: u32,
        max_minor_version: u32,
        nightly_minor_version: u32,
    ) -> std::io::Result<()> {
        let mut output = File::create(path)?;
        let bytes = self
            .path
            .as_os_str()
            .as_encoded_bytes()
            .iter()
            .map(|b| format!("{b}"))
            .collect::<Vec<_>>()
            .join(",");

        write!(
            output,
            "pub const MAJOR_VERSION: usize = {};
pub const MIN_MINOR_VERSION: usize = {};
pub const MAX_MINOR_VERSION: usize = {};
pub const NIGHTLY_MINOR_VERSION: usize = {};
pub const SELECTED_MINOR_VERSION: usize = {};
pub const SELECTED_PATCH_VERSION: usize = {};
pub const DEV_BUILD: bool = {};
pub const DOCS_BUILD: bool = {};
pub const DEBUG_BUILD: bool = {};
pub const WINDOWS_BUILD: bool = {};
pub const BINARY_BUILDER_BUILD: bool = {};
pub static JULIA_DIR: &[u8] = &[{}];
",
            self.version.major,
            lts_version,
            max_minor_version,
            nightly_minor_version,
            self.version.minor,
            self.version.patch,
            self.version.is_dev,
            self.docs,
            self.debug,
            self.windows,
            self.is_binary_builder,
            bytes
        )?;

        Ok(())
    }
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
