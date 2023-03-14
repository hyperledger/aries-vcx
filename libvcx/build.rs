use std::{env, fs, path::Path};

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("target={}", target);

    if target.contains("-windows-") {
        let profile = env::var("PROFILE").unwrap();
        println!("profile={}", profile);

        let output_dir = env::var("OUT_DIR").unwrap();
        println!("output_dir={}", output_dir);
        let output_dir = Path::new(output_dir.as_str());

        let indy_dir = env::var("INDY_DIR").unwrap_or(format!("..\\..\\libvdrtools\\target\\{}", profile));
        println!("indy_dir={}", indy_dir);
        let indy_dir = Path::new(indy_dir.as_str());

        let dst = output_dir.join("..\\..\\..\\..");
        println!("cargo:rustc-flags=-L {}", indy_dir.as_os_str().to_str().unwrap());

        let files = vec!["libeay32md.dll", "libsodium.dll", "libzmq.dll", "ssleay32md.dll"];
        for f in files.iter() {
            if fs::copy(&indy_dir.join(f), &dst.join(f)).is_ok() {
                println!("copy {} -> {}", &indy_dir.join(f).display(), &dst.join(f).display());
            }
        }
    }
}
