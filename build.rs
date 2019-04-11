use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("aarch64-elf-gcc").args(&["src/start.S", "-c", "-o"])
        .arg(&format!("{}/start.o", out_dir))
        .status().unwrap();
    Command::new("aarch64-elf-ar").args(&["crus", "libstart.a", "start.o"])
        .current_dir(&Path::new(&out_dir))
        .status().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=start");
}