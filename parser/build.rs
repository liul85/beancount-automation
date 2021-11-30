use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::join(Path::new(&out_dir), "config.toml");
    fs::copy("config.toml", dest).unwrap();
}
