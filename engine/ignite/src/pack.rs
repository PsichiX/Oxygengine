use std::{
    collections::HashMap,
    fs::{read, write},
    io::{Error, ErrorKind},
    path::Path,
};

pub fn pack_assets<P: AsRef<Path>>(paths: &[P]) -> Result<Vec<u8>, Error> {
    let files = paths
        .iter()
        .flat_map(|path| {
            let path = path.as_ref();
            let mut walker = ignore::WalkBuilder::new(path);
            walker.add_ignore(".pipelineignore");
            walker.build().filter_map(move |entry| {
                entry
                    .ok()
                    .map(|entry| entry.path().to_path_buf())
                    .filter(|p| p.is_file())
                    .and_then(|p| read(&p).ok().map(|c| (p, c)))
                    .and_then(|(p, c)| pathdiff::diff_paths(p, path).map(|p| (p, c)))
                    .and_then(|(p, c)| p.to_str().map(|p| (p.to_owned(), c)))
                    .map(|(p, c)| {
                        let name = p.replace("\\\\", "/").replace("\\", "/");
                        println!("* Include file: {:?} as: {:?}", path.join(p), name);
                        (name, c)
                    })
            })
        })
        .collect::<HashMap<String, Vec<u8>>>();
    match bincode::serialize(&files) {
        Ok(contents) => Ok(contents),
        Err(error) => Err(Error::new(ErrorKind::Other, error.to_string())),
    }
}

pub fn pack_assets_and_write_to_file<P: AsRef<Path>>(paths: &[P], output: P) -> Result<(), Error> {
    let contents = pack_assets(paths)?;
    write(output.as_ref(), contents)?;
    println!("Done! packed to file: {:?}", output.as_ref());
    Ok(())
}
