use std::ops::RangeInclusive;

use proc_macro::{Delimiter, TokenStream, TokenTree};

const MAJOR_VERSION: usize = 1;
const LTS_MINOR_VERSION: usize = 6;
const NIGHTLY_MINOR_VERSION: usize = 12;

#[cfg(not(any(
    feature = "julia-1-6",
    feature = "julia-1-7",
    feature = "julia-1-8",
    feature = "julia-1-9",
    feature = "julia-1-10",
    feature = "julia-1-11",
    feature = "julia-1-12",
)))]
compile_error!(
    "A Julia version must be selected by enabling exactly one of the following version features:
    julia-1-6
    julia-1-7
    julia-1-8
    julia-1-9
    julia-1-10
    julia-1-11
    julia-1-12"
);

#[cfg(any(
    all(feature = "julia-1-6", feature = "julia-1-7"),
    all(feature = "julia-1-6", feature = "julia-1-8"),
    all(feature = "julia-1-6", feature = "julia-1-9"),
    all(feature = "julia-1-6", feature = "julia-1-10"),
    all(feature = "julia-1-6", feature = "julia-1-11"),
    all(feature = "julia-1-6", feature = "julia-1-12"),
    all(feature = "julia-1-7", feature = "julia-1-8"),
    all(feature = "julia-1-7", feature = "julia-1-9"),
    all(feature = "julia-1-7", feature = "julia-1-10"),
    all(feature = "julia-1-7", feature = "julia-1-11"),
    all(feature = "julia-1-7", feature = "julia-1-12"),
    all(feature = "julia-1-8", feature = "julia-1-9"),
    all(feature = "julia-1-8", feature = "julia-1-10"),
    all(feature = "julia-1-8", feature = "julia-1-11"),
    all(feature = "julia-1-8", feature = "julia-1-12"),
    all(feature = "julia-1-9", feature = "julia-1-10"),
    all(feature = "julia-1-9", feature = "julia-1-11"),
    all(feature = "julia-1-9", feature = "julia-1-12"),
    all(feature = "julia-1-10", feature = "julia-1-11"),
    all(feature = "julia-1-10", feature = "julia-1-12"),
    all(feature = "julia-1-11", feature = "julia-1-12"),
))]
compile_error!("Multiple Julia version features have been enabled");

// Avoid a second error if no version feature is enabled
#[cfg(not(any(
    feature = "julia-1-7",
    feature = "julia-1-8",
    feature = "julia-1-9",
    feature = "julia-1-10",
    feature = "julia-1-11",
    feature = "julia-1-12",
)))]
const SELECTED_MINOR_VERSION: usize = 6;
#[cfg(feature = "julia-1-7")]
const SELECTED_MINOR_VERSION: usize = 7;
#[cfg(feature = "julia-1-8")]
const SELECTED_MINOR_VERSION: usize = 8;
#[cfg(feature = "julia-1-9")]
const SELECTED_MINOR_VERSION: usize = 9;
#[cfg(feature = "julia-1-10")]
const SELECTED_MINOR_VERSION: usize = 10;
#[cfg(feature = "julia-1-11")]
const SELECTED_MINOR_VERSION: usize = 11;
#[cfg(feature = "julia-1-12")]
const SELECTED_MINOR_VERSION: usize = 12;

pub fn emit_if_compatible(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut tts = attr.into_iter();
    let mut since = None;
    let mut until = None;
    let mut except = None;
    let mut windows_lts = None;

    loop {
        match tts.next() {
            Some(TokenTree::Ident(ident)) => match ident.to_string().as_ref() {
                "since" => {
                    expect_punt_eq(&mut tts);
                    since = Some(unwrap_version(&mut tts));
                    if !expect_comma_or_end(&mut tts) {
                        break;
                    }
                }
                "until" => {
                    expect_punt_eq(&mut tts);
                    until = Some(unwrap_version(&mut tts));
                    if !expect_comma_or_end(&mut tts) {
                        break;
                    }
                }
                "windows_lts" => {
                    expect_punt_eq(&mut tts);
                    windows_lts = Some(unwrap_bool(&mut tts));
                    if !expect_comma_or_end(&mut tts) {
                        break;
                    }
                }
                "except" => {
                    expect_punt_eq(&mut tts);
                    except = Some(unwrap_version_group(&mut tts));
                    if !expect_comma_or_end(&mut tts) {
                        break;
                    }
                }
                ident => panic!("Unexpected identifier {}", ident),
            },
            None => break,
            Some(tt) => panic!("Unexpected tokens {}", tt),
        }
    }

    let since = since.unwrap_or(Version::new(MAJOR_VERSION, LTS_MINOR_VERSION));
    let until = until.unwrap_or(Version::new(MAJOR_VERSION, NIGHTLY_MINOR_VERSION));
    let except = except.unwrap_or_default();

    if should_emit(since, until, windows_lts, &except) {
        item
    } else {
        TokenStream::new()
    }
}

fn expect_punt_eq<T: Iterator<Item = TokenTree>>(iter: &mut T) {
    match iter.next() {
        Some(TokenTree::Punct(punct)) => {
            let punct = punct.as_char();
            if punct != '=' {
                panic!("Expected =, got {}", punct)
            }
        }
        Some(other) => panic!("Expected =, got {}", other),
        None => panic!("Expected =, got nothing"),
    }
}

fn expect_comma_or_end<T: Iterator<Item = TokenTree>>(iter: &mut T) -> bool {
    match iter.next() {
        Some(TokenTree::Punct(punct)) => {
            let punct = punct.as_char();
            if punct != ',' {
                panic!("Expected ,, got {}", punct)
            }
            true
        }
        Some(other) => panic!("Expected =, got {}", other),
        None => false,
    }
}

fn unwrap_bool<T: Iterator<Item = TokenTree>>(iter: &mut T) -> bool {
    match iter.next() {
        Some(TokenTree::Ident(ident)) => {
            let ident = ident.to_string();
            match ident.as_ref() {
                "true" => true,
                "false" => false,
                other => panic!("Expected true or false, got {}", other),
            }
        }
        Some(other) => panic!("Expected true or false, got {}", other),
        None => panic!("Expected true or false, got nothing"),
    }
}

#[derive(PartialOrd, PartialEq)]
struct Version {
    major: usize,
    minor: usize,
}

impl Version {
    const fn new(major: usize, minor: usize) -> Self {
        Version { major, minor }
    }

    fn assert_valid(&self, major: usize, minor: RangeInclusive<usize>) {
        if major != self.major {
            panic!("Unsupported major version");
        }

        if !minor.contains(&self.minor) {
            panic!("Unsupported minor version");
        }
    }
}

fn unwrap_version<T: Iterator<Item = TokenTree>>(iter: &mut T) -> Version {
    match iter.next() {
        Some(TokenTree::Literal(lit)) => {
            let lit = lit.to_string();
            let lit = &lit[1..lit.len() - 1];
            let mut iter = lit.split(".");
            let major = iter
                .next()
                .expect("Expected of the form major.minor")
                .parse::<usize>()
                .expect("Not a number");
            let minor = iter
                .next()
                .expect("Expected of the form major.minor")
                .parse::<usize>()
                .expect("Not a number");
            assert!(iter.next().is_none(), "Expected of the form major.minor");

            let version = Version::new(major, minor);
            version.assert_valid(MAJOR_VERSION, LTS_MINOR_VERSION..=NIGHTLY_MINOR_VERSION);

            version
        }
        Some(other) => panic!(
            "Expected a version in the form major.minor, got {:?}",
            other
        ),
        None => panic!("Expected a version in the form major.minor, got nothing"),
    }
}

fn unwrap_version_group<T: Iterator<Item = TokenTree>>(iter: &mut T) -> Vec<Version> {
    match iter.next() {
        Some(TokenTree::Group(group)) => {
            let mut versions = vec![];
            if group.delimiter() != Delimiter::Bracket {
                panic!("Expected a bracketed group of versions")
            }

            let mut group_tts = group.stream().into_iter();
            loop {
                let version = unwrap_version(&mut group_tts);
                versions.push(version);
                if !expect_comma_or_end(&mut group_tts) {
                    break;
                }
            }

            versions
        }
        Some(other) => panic!("Expected a bracketed group of versions, got {:?}", other),
        None => panic!("Expected a bracketed group of versions, got nothing"),
    }
}

fn selected_version() -> Version {
    Version::new(1, SELECTED_MINOR_VERSION)
}

#[allow(unused_variables)]
fn should_emit(
    since: Version,
    until: Version,
    windows_lts: Option<bool>,
    except: &[Version],
) -> bool {
    let selected = selected_version();
    if since > selected {
        return false;
    }

    if selected > until {
        return false;
    }

    if except.contains(&selected) {
        return false;
    }

    if let Some(windows_lts) = windows_lts {
        #[cfg(any(feature = "windows", target_os = "windows"))]
        if selected.minor == LTS_MINOR_VERSION && !windows_lts {
            return false;
        }

        #[cfg(any(feature = "windows", target_os = "windows"))]
        if selected.minor != LTS_MINOR_VERSION && windows_lts {
            return false;
        }

        #[cfg(not(any(feature = "windows", target_os = "windows")))]
        if windows_lts {
            return false;
        }
    }

    true
}
