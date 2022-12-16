extern crate cc;

fn main() {

    println!("cargo:rerun-if-changed=src/titomach.c");
    println!("cargo:rerun-if-changed=src/titoinstr.c");
    println!("cargo:rerun-if-changed=src/titostate.c");

    cc::Build::new()
        .file("titolib/titomach.c")
        .file("titolib/titoinstr.c")
        .file("titolib/titostate.c")
        .compile("titolib");
}