use std::{
    collections::HashMap,
    fs::{read, read_dir},
    io::Error,
    path::Path,
};

pub(crate) fn scan_dir(
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
