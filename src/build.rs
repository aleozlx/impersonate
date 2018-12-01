extern crate cc;
fn main() {
    cc::Build::new().file("src/su-exec.c").compile("suexec");
}