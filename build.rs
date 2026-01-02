fn main() {
    println!("cargo:rustc-link-search=native={}", "preloader");
    println!("cargo:rustc-link-lib=preloader");
    cc::Build::new()
        .cpp(true)
        .file("src/string.cpp")
        .compile("stringstub");
}