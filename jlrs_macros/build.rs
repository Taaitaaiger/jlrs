fn main() {
    let version = std::env::var("DEP_JULIA_VERSION").expect("Julia version not set");
    let parts: Vec<&str> = version.split('.').collect();

    let major: i32 = parts[0].parse().expect("major version is not a number");
    let minor: i32 = parts[1].parse().expect("minor version is not a number");

    println!("cargo::rustc-cfg=feature=\"julia-{}-{}\"", major, minor);
}
