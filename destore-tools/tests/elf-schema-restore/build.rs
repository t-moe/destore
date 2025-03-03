fn main() {
    println!("cargo:rustc-link-arg=-Tdestore.x");
    println!("cargo:rustc-link-arg=-Tlinkall.x");
}
