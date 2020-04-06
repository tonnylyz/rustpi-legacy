use std::process::Command;
use std::env;

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();
  Command::new("aarch64-elf-ar")
    .arg("crus")
    .arg(&format!("{}/libuserspace.a", out_dir))
    .args(&["user/user.o"])
    .status().unwrap();
  println!("cargo:rustc-link-search=native={}", out_dir);
  println!("cargo:rustc-link-lib=static=userspace");
}