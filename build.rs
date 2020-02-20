use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();

  Command::new("aarch64-elf-gcc").args(&["boot/start.S", "-c", "-o"])
    .arg(&format!("{}/start.o", out_dir))
    .status().unwrap();
  Command::new("aarch64-elf-gcc").args(&["boot/vm.c", "-c", "-o"])
    .arg(&format!("{}/vm.o", out_dir))
    .status().unwrap();
  Command::new("aarch64-elf-ar").args(&["cr", "libstart.a", "start.o", "vm.o"])
    .current_dir(&Path::new(&out_dir))
    .status().unwrap();

  println!("cargo:rustc-link-search=native={}", out_dir);
  println!("cargo:rustc-link-lib=static=start");
}