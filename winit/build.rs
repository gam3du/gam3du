#![allow(missing_docs, reason = "just a demo build script")]
fn main() {
    println!("cargo::rustc-check-cfg=cfg(web_platform, macos_platform, android_platform, x11_platform, wayland_platform)");
}
