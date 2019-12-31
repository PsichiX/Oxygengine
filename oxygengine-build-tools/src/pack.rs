use std::{
    collections::HashMap,
    fs::{read, read_dir, write},
    io::{Error, ErrorKind},
    path::Path,
};

pub fn pack_assets<P: AsRef<Path>>(input: P, quiet: bool) -> Result<Vec<u8>, Error> {
    if !quiet {
        println!("* Pack assets: {:?}", input.as_ref());
    }
    let mut filemap = HashMap::new();
    let root = Path::new(input.as_ref());
    scan_dir(&root, &root, &mut filemap, quiet)?;
    match bincode::serialize(&filemap) {
        Ok(contents) => Ok(contents),
        Err(error) => Err(Error::new(ErrorKind::Other, error.to_string())),
    }
}

pub fn pack_assets_and_write_to_file<P: AsRef<Path>>(
    input: P,
    output: P,
    quiet: bool,
) -> Result<(), Error> {
    let contents = pack_assets(input, quiet)?;
    write(output.as_ref(), contents)?;
    if !quiet {
        println!("  Done! packed to file: {:?}", output.as_ref());
    }
    Ok(())
}

fn scan_dir(
    from: &Path,
    root: &Path,
    map: &mut HashMap<String, Vec<u8>>,
    quiet: bool,
) -> Result<(), Error> {
    if from.is_dir() {
        for entry in read_dir(from)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, root, map, quiet)?;
            } else if path.is_file() {
                if let Ok(contents) = read(&path) {
                    if let Some(path) = pathdiff::diff_paths(&path, root) {
                        if let Some(path) = path.to_str() {
                            if !quiet {
                                println!("* Include file: {:?} as: {:?}", root.join(path), path);
                            }
                            map.insert(path.to_owned(), contents);
                        } else if !quiet {
                            println!("* Cannot parse path: {:?}", root.join(path));
                        }
                    } else if !quiet {
                        println!("* Cannot diff path: {:?} from root: {:?}", path, root);
                    }
                } else if !quiet {
                    println!("* Cannot read file: {:?}", path);
                }
            }
        }
    }
    Ok(())
}
