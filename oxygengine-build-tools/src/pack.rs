use crate::utils::scan_dir;
use std::{
    collections::HashMap,
    fs::{read, write},
    io::{Error, ErrorKind},
    path::Path,
};

pub fn pack_assets<P: AsRef<Path>>(paths: &[P], quiet: bool) -> Result<Vec<u8>, Error> {
    let mut files = HashMap::new();
    for path in paths {
        let path = path.as_ref();
        if path.is_file() {
            if let Ok(contents) = read(&path) {
                if let Some(path) = path.to_str() {
                    if !quiet {
                        println!("* Include file: {:?}", path);
                    }
                    let name = path.to_owned().replace("\\\\", "/").replace("\\", "/");
                    files.insert(name, contents);
                } else if !quiet {
                    println!("* Cannot parse path: {:?}", path);
                }
            } else if !quiet {
                println!("* Cannot read file: {:?}", path);
            }
        } else {
            scan_dir(path, path, &mut files, quiet)?;
        }
    }
    match bincode::serialize(&files) {
        Ok(contents) => Ok(contents),
        Err(error) => Err(Error::new(ErrorKind::Other, error.to_string())),
    }
}

pub fn pack_assets_and_write_to_file<P: AsRef<Path>>(
    paths: &[P],
    output: P,
    quiet: bool,
) -> Result<(), Error> {
    let contents = pack_assets(paths, quiet)?;
    write(output.as_ref(), contents)?;
    if !quiet {
        println!("  Done! packed to file: {:?}", output.as_ref());
    }
    Ok(())
}
