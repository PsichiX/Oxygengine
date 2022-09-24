use crate::pack::pack_assets_and_write_to_file;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    io::{Error, ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn pathbuf_is_empty(buf: &Path) -> bool {
    buf.components().count() == 0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn bool_is_false(value: &bool) -> bool {
    !value
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    pub disabled: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    pub source: PathBuf,
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    pub destination: PathBuf,
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    pub clear_destination: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<PipelineCommand>,
}

impl Pipeline {
    pub fn execute(self) -> Result<(), Error> {
        if self.disabled {
            return Ok(());
        }
        if self.clear_destination {
            drop(fs_extra::dir::remove(&self.destination));
        }
        drop(fs_extra::dir::create_all(&self.destination, false));
        for command in self.commands.iter().cloned() {
            match command {
                PipelineCommand::Copy { disabled, from, to } => {
                    if disabled {
                        continue;
                    }
                    if from.is_empty() && pathbuf_is_empty(&to) {
                        let mut options = fs_extra::dir::CopyOptions::new();
                        options.overwrite = true;
                        options.copy_inside = true;
                        options.content_only = true;
                        if let Err(error) =
                            fs_extra::dir::copy(&self.source, &self.destination, &options)
                        {
                            return Err(Error::new(
                                ErrorKind::Other,
                                format!("Could not copy directory content: {:?}", error),
                            ));
                        }
                        continue;
                    }
                    let from = from
                        .into_iter()
                        .map(|path| {
                            if path.is_absolute() {
                                path
                            } else {
                                self.source.join(path)
                            }
                        })
                        .collect::<Vec<_>>();
                    let to = if to.is_absolute() {
                        to
                    } else {
                        self.destination.join(to)
                    };
                    let mut options = fs_extra::dir::CopyOptions::new();
                    options.overwrite = true;
                    options.copy_inside = true;
                    if let Err(error) = fs_extra::copy_items(&from, to, &options) {
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("Could not copy files: {:?}", error),
                        ));
                    }
                }
                PipelineCommand::Pack {
                    disabled,
                    paths,
                    output,
                } => {
                    if disabled {
                        continue;
                    }
                    let paths = paths
                        .into_iter()
                        .map(|path| {
                            if path.is_absolute() {
                                path
                            } else {
                                self.source.join(path)
                            }
                        })
                        .collect::<Vec<_>>();
                    let output = if output.is_absolute() {
                        output
                    } else {
                        self.destination.join(output)
                    };
                    pack_assets_and_write_to_file(&paths, output)?;
                }
                PipelineCommand::Plugin {
                    disabled,
                    name,
                    params,
                    ..
                } => {
                    if disabled {
                        continue;
                    }
                    let mut child = Command::new(&name)
                        .arg(&self.source)
                        .arg(&self.destination)
                        .stdin(Stdio::piped())
                        .spawn()
                        .unwrap_or_else(|_| panic!("Could not run plugin: {0}. Make sure it is installed or install it with: `cargo install {0}`", name));
                    let mut stdin = child.stdin.take().unwrap_or_else(|| {
                        panic!("Could not take input stream of plugin process: {}", name)
                    });
                    let params = serde_json::to_string(&params).unwrap_or_else(|_| {
                        panic!("Could not serialize params for plugin: {}", name)
                    });
                    stdin.write_all(params.as_bytes()).unwrap_or_else(|_| {
                        panic!("Could not write serialized parameters for plugin: {}", name)
                    });
                    stdin.flush().unwrap_or_else(|_| {
                        panic!("Could not complete sending data to plugin: {}", name)
                    });
                    drop(stdin);
                    if !child
                        .wait()
                        .unwrap_or_else(|_| panic!("Could not wait for plugin: {}", name))
                        .success()
                    {
                        panic!("Plugin run failed: {}", name);
                    }
                }
                PipelineCommand::Shell { disabled, command } => {
                    if disabled {
                        continue;
                    }
                    let command = command
                        .replace("<source>", self.source.to_str().unwrap())
                        .replace("<destination>", self.destination.to_str().unwrap());
                    let parts = command.split(char::is_whitespace).collect::<Vec<_>>();
                    let mut command = Command::new(&parts[0]);
                    for part in &parts[1..] {
                        command.arg(part);
                    }
                    command
                        .status()
                        .unwrap_or_else(|_| panic!("Shell command failed to run: {:?}", command));
                }
                PipelineCommand::Pipeline(mut pipeline) => {
                    pipeline.source = if pipeline.source.is_absolute() {
                        pipeline.source
                    } else {
                        self.source.join(pipeline.source)
                    };
                    pipeline.destination = if pipeline.destination.is_absolute() {
                        pipeline.destination
                    } else {
                        self.destination.join(pipeline.destination)
                    };
                    pipeline.execute()?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipelineCommand {
    Copy {
        #[serde(default)]
        #[serde(skip_serializing_if = "bool_is_false")]
        disabled: bool,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        from: Vec<PathBuf>,
        #[serde(default)]
        #[serde(skip_serializing_if = "pathbuf_is_empty")]
        to: PathBuf,
    },
    Pack {
        #[serde(default)]
        #[serde(skip_serializing_if = "bool_is_false")]
        disabled: bool,
        #[serde(default)]
        #[serde(skip_serializing_if = "Vec::is_empty")]
        paths: Vec<PathBuf>,
        #[serde(default)]
        #[serde(skip_serializing_if = "pathbuf_is_empty")]
        output: PathBuf,
    },
    Plugin {
        #[serde(default)]
        #[serde(skip_serializing_if = "bool_is_false")]
        disabled: bool,
        #[serde(default)]
        #[serde(skip_serializing_if = "bool_is_false")]
        do_not_verify: bool,
        name: String,
        #[serde(default)]
        #[serde(skip_serializing_if = "Value::is_null")]
        params: Value,
    },
    Shell {
        #[serde(default)]
        #[serde(skip_serializing_if = "bool_is_false")]
        disabled: bool,
        command: String,
    },
    Pipeline(Pipeline),
}
