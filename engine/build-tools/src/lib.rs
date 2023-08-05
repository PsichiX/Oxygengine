use serde::Deserialize;
use std::{collections::HashMap, fs::read_to_string, io::Write, path::PathBuf};

pub trait ParamsFromArgs: Sized {
    fn params_from_args(_args: impl Iterator<Item = String>) -> Option<Self> {
        None
    }
}

#[derive(Debug, Default, Clone)]
pub struct StructuredArguments(HashMap<String, Vec<String>>);

impl StructuredArguments {
    pub fn read(&self, name: &'static str) -> Option<impl Iterator<Item = &str>> {
        Some(self.0.get(name)?.iter().map(|item| item.as_str()))
    }

    pub fn read_many(&self, name: &'static [&'static str]) -> impl Iterator<Item = &str> {
        name.iter().filter_map(|name| self.read(name)).flatten()
    }

    pub fn read_default(&self) -> impl Iterator<Item = &str> {
        self.read("").unwrap()
    }

    pub fn consume(&mut self, name: &'static str) -> Option<impl Iterator<Item = String>> {
        Some(self.0.remove(name)?.into_iter())
    }

    pub fn consume_many<'a>(
        &'a mut self,
        name: &'static [&'static str],
    ) -> impl Iterator<Item = String> + 'a {
        name.iter().filter_map(|name| self.consume(name)).flatten()
    }

    pub fn consume_default(&mut self) -> impl Iterator<Item = String> {
        self.consume("").unwrap()
    }

    pub fn new(args: impl Iterator<Item = String>) -> StructuredArguments {
        let mut result = HashMap::<_, Vec<_>>::default();
        result.insert(Default::default(), Default::default());
        let mut name = String::new();
        for arg in args {
            if let Some(arg) = arg.strip_prefix("--") {
                name = arg.to_owned();
            } else if arg.starts_with('-') && arg.len() == 2 {
                name = arg[1..].to_owned()
            } else {
                result.entry(name.to_owned()).or_default().push(arg);
            }
        }
        StructuredArguments(result)
    }
}

pub struct AssetPipelinePlugin;

impl AssetPipelinePlugin {
    pub fn run<T, E>(
        f: impl FnOnce(AssetPipelineInput<T>) -> Result<Vec<String>, E>,
    ) -> Result<(), E>
    where
        T: for<'de> Deserialize<'de> + ParamsFromArgs,
    {
        let output = f(AssetPipelineInput::<T>::consume())?;
        serde_json::to_writer(std::io::stdout(), &output)
            .expect("Could not serialize output content");
        let _ = std::io::stdout().flush();
        Ok(())
    }
}

pub struct AssetPipelineInput<T> {
    pub source: Vec<PathBuf>,
    pub target: PathBuf,
    pub assets: String,
    pub params: T,
}

impl<T> AssetPipelineInput<T> {
    fn consume() -> Self
    where
        T: for<'de> Deserialize<'de> + ParamsFromArgs,
    {
        let mut args = std::env::args();
        args.next();
        let mut source = vec![];
        let mut target = Default::default();
        let mut assets = Default::default();
        for arg in args.by_ref() {
            if arg == "--" {
                break;
            } else {
                source.push(arg.into());
            }
        }
        for arg in args.by_ref() {
            if arg == "--" {
                break;
            } else {
                target = arg.into();
            }
        }
        for arg in args.by_ref() {
            if arg == "--" {
                break;
            } else {
                assets = arg;
            }
        }
        let params = if let Some(arg) = args.next() {
            if arg == "--" {
                T::params_from_args(args).expect("Could not read args input content")
            } else if let Some((t, i)) = arg.find('=').map(|i| (&arg[0..i], i + 1)) {
                let content = match t {
                    "file" => {
                        let path = &arg[i..];
                        read_to_string(path)
                            .unwrap_or_else(|_| panic!("Could not read file: {}", path))
                    }
                    "data" => arg[i..].to_owned(),
                    name => panic!("Unexpected type: {}", name),
                };
                serde_json::from_str::<T>(&content).expect("Could not deserialize input content")
            } else {
                panic!("Wrong input: {:?}", arg)
            }
        } else {
            serde_json::from_reader::<_, T>(std::io::stdin())
                .expect("Could not deserialize input stream")
        };
        Self {
            source,
            target,
            assets,
            params,
        }
    }
}
