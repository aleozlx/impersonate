extern crate cc;

fn main() {
    cc::Build::new()
        .file("src/su-exec.c")
        .compile("suexec");
}

// fn main() {
//     let out_dir = "target";
//     Command::new("gcc").args(&["-c", "-o", &format!("{}/su-exec.o", out_dir), "src/su-exec.c"])
//         .status().unwrap();
//     Command::new("ar").args(&["rcs", "libsuexec.a", "su-exec.o"])
//         .current_dir(&Path::new(&out_dir))
//         .status().unwrap();
//     println!("cargo:rustc-link-search=native={}", out_dir);
//     println!("cargo:rustc-link-lib=static=suexec");
// }
