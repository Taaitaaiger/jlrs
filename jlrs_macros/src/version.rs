use std::ops::RangeInclusive;

use proc_macro::{Delimiter, TokenStream, TokenTree};

const MAJOR_VERSION: usize = 1;
const LTS_MINOR_VERSION: usize = 10;
const NIGHTLY_MINOR_VERSION: usize = 13;

#[cfg(any(
    all(julia_1_10, julia_1_11),
    all(julia_1_10, julia_1_12),
    all(julia_1_10, julia_1_13),
    all(julia_1_11, julia_1_12),
    all(julia_1_11, julia_1_13),
    all(julia_1_12, julia_1_13),
))]
compile_error!("Multiple Julia versions have been detected");

#[cfg(not(any(julia_1_10, julia_1_11, julia_1_12, julia_1_13)))]
const SELECTED_MINOR_VERSION: usize = 10;
#[cfg(julia_1_10)]
const SELECTED_MINOR_VERSION: usize = 10;
#[cfg(julia_1_11)]
const SELECTED_MINOR_VERSION: usize = 11;
#[cfg(julia_1_12)]
const SELECTED_MINOR_VERSION: usize = 12;
#[cfg(julia_1_13)]
const SELECTED_MINOR_VERSION: usize = 13;

pub fn emit_if_compatible(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut tts = attr.into_iter();
    let mut since = None;
    let mut until = None;
    let mut except = None;

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

    if should_emit(since, until, &except) {
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
fn should_emit(since: Version, until: Version, except: &[Version]) -> bool {
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

    true
}
