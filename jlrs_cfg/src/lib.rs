pub fn configure(min_version: i32, max_version: i32) {
    enable_julia_cfgs(min_version, max_version);
    emit_julia_cfg(min_version, max_version);
}

fn enable_julia_cfgs(min_version: i32, max_version: i32) {
    let versions: Vec<String> = (min_version..=max_version)
        .map(|minor| format!("julia_1_{minor}"))
        .collect();
    let versions_joined = versions.join(",");
    println!("cargo::rustc-check-cfg=cfg({versions_joined})");
}

fn emit_julia_cfg(min_version: i32, max_version: i32) {
    let version = std::env::var("DEP_JULIA_VERSION").expect("Julia version not set by jl-sys");

    let mut parts = version.split('.');
    let major: i32 = parts
        .next()
        .expect("no major version")
        .parse()
        .expect("major version is not a number");
    let minor: i32 = parts
        .next()
        .expect("no minor version")
        .parse()
        .expect("minor version is not a number");

    if major != 1 {
        println!("cargo::error=Unsupported major version of Julia; expected 1, detected {major}");
        return;
    }

    if minor < min_version {
        println!(
            "cargo::error=Unsupported minor version of Julia; expected at least 1.10, detected {major}.{minor}"
        );
        return;
    }

    if minor > max_version {
        println!(
            "cargo::rustc-warning=\"Detected unsupported Julia version {major}.{minor}, assuming compatibility with {major}.{max_version}. \
        Please report any issues at https://www.github.com/Taaitaaiger/jlrs/issues\""
        );
        println!("cargo::rustc-cfg=julia_{}_{}", major, max_version);
        return;
    }

    println!("cargo::rustc-cfg=julia_{}_{}", major, minor);
}
