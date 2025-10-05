fn main() {
    // https://github.com/embassy-rs/embassy/blob/main/examples/stm32f4/build.rs
    println!("cargo:rustc-link-arg=--nmagic");
    println!("cargo:rustc-link-arg=-Tlink.x");
    println!("cargo:rustc-link-arg=-Tdefmt.x");
    println!("cargo:rerun-if-changed=memory.x");
}
