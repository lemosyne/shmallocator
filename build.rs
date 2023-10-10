fn main() {
    println!("cargo:rustc-link-search=./psmalloc/build/");
    println!("cargo:rustc-link-lib=dylib=psm");
}
