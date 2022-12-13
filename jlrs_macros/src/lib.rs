#![allow(unused_variables)]

use proc_macro::{Delimiter, TokenStream, TokenTree};

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

fn unwrap_version<T: Iterator<Item = TokenTree>>(iter: &mut T) -> String {
    match iter.next() {
        Some(TokenTree::Literal(lit)) => {
            let lit = lit.to_string();
            let lit = (&lit[1..lit.len() - 1]).to_string();
            match lit.as_ref() {
                "1.6" => lit,
                "1.7" => lit,
                "1.8" => lit,
                "1.9" => lit,
                "1.10" => lit,
                other => panic!("Expected a version in the form major.minor, got {}", other),
            }
        }
        Some(other) => panic!(
            "Expected a version in the form major.minor, got {:?}",
            other
        ),
        None => panic!("Expected a version in the form major.minor, got nothing"),
    }
}

fn unwrap_version_group<T: Iterator<Item = TokenTree>>(iter: &mut T) -> Vec<String> {
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

fn should_emit(since: &str, until: &str, windows_lts: bool, except: &[&str]) -> bool {
    #[cfg(feature = "julia-1-6")]
    {
        if since != "1.6" {
            return false;
        }

        if except.contains(&"1.6") {
            return false;
        }

        #[cfg(target_os = "windows")]
        if !windows_lts {
            return false;
        }
    }

    #[cfg(feature = "julia-1-7")]
    {
        if since != "1.6" && since != "1.7" {
            return false;
        }

        if until == "1.6" {
            return false;
        }

        if except.contains(&"1.7") {
            return false;
        }
    }

    #[cfg(any(
        feature = "julia-1-8",
        not(any(
            feature = "julia-1-6",
            feature = "julia-1-7",
            feature = "julia-1-8",
            feature = "julia-1-9",
            feature = "julia-1-10"
        ))
    ))]
    {
        if since != "1.6" && since != "1.7" && since != "1.8" {
            return false;
        }

        if until == "1.6" || until == "1.7" {
            return false;
        }

        if except.contains(&"1.8") {
            return false;
        }
    }

    #[cfg(feature = "julia-1-9")]
    {
        if since != "1.6" && since != "1.7" && since != "1.8" && since != "1.9" {
            return false;
        }

        if until == "1.6" || until == "1.7" || until == "1.8" {
            return false;
        }

        if except.contains(&"1.9") {
            return false;
        }
    }

    #[cfg(feature = "julia-1-10")]
    {
        if since != "1.6" && since != "1.7" && since != "1.8" && since != "1.9" && since != "1.10" {
            return false;
        }

        if until == "1.6" || until == "1.7" || until == "1.8" || until == "1.9" {
            return false;
        }

        if except.contains(&"1.10") {
            return false;
        }
    }

    true
}

// #[jlrs(since = "1.6", until = "1.10", windows-lts = false, except = ["1.7", "1.9"])]
#[proc_macro_attribute]
pub fn julia_version(attr: TokenStream, item: TokenStream) -> TokenStream {
    /*
    for tt in attr.clone() {
        println!("Attr: {:?}", tt)
    }
    */
    let mut tts = attr.into_iter();
    let mut since = None;
    let mut until = None;
    let mut except = None;
    let mut windows_lts = true;

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
                    windows_lts = unwrap_bool(&mut tts);
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

    let since = since.as_ref().map(|s| s.as_str()).unwrap_or("1.6");
    let until = until.as_ref().map(|s| s.as_str()).unwrap_or("1.10");
    let except = except
        .as_ref()
        .map(|x| x.iter().map(|x| x.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    if should_emit(since, until, windows_lts, &except) {
        item
    } else {
        TokenStream::new()
    }
}
