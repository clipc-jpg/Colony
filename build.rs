


// by zacooons at https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust
use std::path::Path;
use std::{io, fs};

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}




// Copies assets manually, since asseting in dioxus appears to be not 100% reliable
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=public/images/*");
	// Copy files compatible with compilation without cioxus-cli
    copy_dir_all("public/images", "target/debug/assets");
    copy_dir_all("public/images", "target/release/assets");
}

















