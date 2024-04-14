extern crate winres;
extern crate copy_to_output;

use std::env;
use copy_to_output::copy_to_output;

fn main() {
    println!("cargo:rerun-if-changed=res/*");
    copy_to_output("assets", &env::var("PROFILE").unwrap()).expect("Could not copy");

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico"); // Replace this with the filename of your .ico file.
        res.compile().unwrap();
    }
}