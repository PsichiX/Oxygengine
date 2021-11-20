use serde::Deserialize;
use std::{fs::read_to_string, path::PathBuf};

pub struct AssetPipelineInput<T> {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub params: T,
}

impl<T> AssetPipelineInput<T> {
    pub fn consume() -> Self
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut args = std::env::args();
        args.next();
        let source = PathBuf::from(args.next().expect("* Could not read source path"));
        let destination = PathBuf::from(args.next().expect("* Could not read destination path"));
        let params = if let Some(arg) = args.next() {
            let (t, i) = arg
                .find('=')
                .map(|i| (&arg[0..i], i + 1))
                .unwrap_or_else(|| ("file", 0));
            let content = match t {
                "file" => {
                    let path = &arg[i..];
                    read_to_string(path).unwrap_or_else(|_| panic!("Could not read file: {}", path))
                }
                "data" => arg[i..].to_owned(),
                name => panic!("Unexpected type: {}", name),
            };
            serde_json::from_str::<T>(&content).expect("Could not deserialize input content")
        } else {
            serde_json::from_reader::<_, T>(std::io::stdin())
                .expect("Could not deserialize input stream")
        };
        Self {
            source,
            destination,
            params,
        }
    }

    pub fn unwrap(self) -> (PathBuf, PathBuf, T) {
        (self.source, self.destination, self.params)
    }
}
