use std::{
    env,
    error::Error,
    fs::{copy, create_dir_all, read_dir},
    path::{Path, PathBuf},
};

pub fn visit(dir: &Path, out: &Path) -> Result<(), Box<dyn Error>> {
    println!("{} ->> {}", dir.display(), out.join(dir).display());
    create_dir_all(out.join(&dir))?;

    for entry in read_dir(dir)? {
        if let Ok(entry) = entry {
            println!("cargo::rerun-if-changed={}", entry.path().display());

            if entry.file_type()?.is_dir() {
                visit(&entry.path(), &out)?;
            } else {
                copy(entry.path(), out.join(entry.path()))?;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir: PathBuf = env::var("OUT_DIR")?.into();
    let web_dir: PathBuf = "web".into();

    visit(&web_dir, &out_dir)?;

    Ok(())
}
