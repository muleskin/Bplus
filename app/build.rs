//! Draws the Bplus icon and embeds it in the Windows program file.

#[path = "src/icon.rs"]
mod icon;

fn main() {
    println!("cargo:rerun-if-changed=src/icon.rs");
    println!("cargo:rerun-if-changed=build.rs");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("bplus.ico");
    std::fs::write(&out, icon::ico()).expect("write icon");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(out.to_str().unwrap());
        res.set("ProductName", "Bplus Rename");
        res.set("FileDescription", "Bplus Rename");
        res.compile().expect("embed icon");
    }
}
