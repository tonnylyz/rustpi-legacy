use std::process::Command;
use std::env;

fn main() {
  let target = env::var("TARGET").expect("TARGET was not set");
  if target.contains("riscv64") {
    // TODO: generate different user image
    return;
  }
  let out_dir = env::var("OUT_DIR").unwrap();
  Command::new("aarch64-elf-ar")
    .arg("crus")
    .arg(&format!("{}/libuserspace.a", out_dir))
    .args(&["user/user.o"])
    .status().unwrap();
  println!("cargo:rustc-link-search=native={}", out_dir);
  println!("cargo:rustc-link-lib=static=userspace");
}