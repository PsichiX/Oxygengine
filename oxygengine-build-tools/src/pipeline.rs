use crate::{
    atlas::pack_sprites_and_write_to_files, pack::pack_assets_and_write_to_file,
    tiled::build_map_and_write_to_file,
};
use serde::{Deserialize, Serialize};
use std::{
    env::var,
    fs::read_to_string,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

fn pathbuf_is_empty(buf: &PathBuf) -> bool {
    buf.components().count() == 0
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn bool_is_false(value: &bool) -> bool {
    !value
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CopyPhase {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    from: Vec<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    to: PathBuf,
}

impl CopyPhase {
    #[allow(clippy::wrong_self_convention)]
    pub fn from_multi<P: AsRef<Path>>(mut self, from: &[P]) -> Self {
        self.from = from
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect::<Vec<_>>();
        self
    }

    pub fn from<P: AsRef<Path>>(mut self, from: P) -> Self {
        self.from.push(from.as_ref().to_path_buf());
        self
    }

    pub fn to<P: AsRef<Path>>(mut self, to: P) -> Self {
        self.to = to.as_ref().to_path_buf();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasPhase {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    paths: Vec<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    output_image: PathBuf,
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    output_atlas: PathBuf,
    #[serde(default = "AtlasPhase::default_max_width")]
    #[serde(skip_serializing_if = "AtlasPhase::is_default_max_width")]
    max_width: usize,
    #[serde(default = "AtlasPhase::default_max_height")]
    #[serde(skip_serializing_if = "AtlasPhase::is_default_max_height")]
    max_height: usize,
    #[serde(default = "AtlasPhase::default_padding")]
    #[serde(skip_serializing_if = "AtlasPhase::is_default_padding")]
    padding: usize,
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    pretty: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    full_names: bool,
}

impl Default for AtlasPhase {
    fn default() -> Self {
        Self {
            paths: Default::default(),
            output_image: Default::default(),
            output_atlas: Default::default(),
            max_width: 2048,
            max_height: 2048,
            padding: 2,
            pretty: false,
            full_names: false,
        }
    }
}

impl AtlasPhase {
    fn default_max_width() -> usize {
        2048
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default_max_width(value: &usize) -> bool {
        *value == Self::default_max_width()
    }

    fn default_max_height() -> usize {
        2048
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default_max_height(value: &usize) -> bool {
        *value == Self::default_max_height()
    }

    fn default_padding() -> usize {
        2
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_default_padding(value: &usize) -> bool {
        *value == Self::default_padding()
    }

    pub fn paths<P: AsRef<Path>>(mut self, paths: &[P]) -> Self {
        self.paths = paths
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect::<Vec<_>>();
        self
    }

    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.paths.push(path.as_ref().to_path_buf());
        self
    }

    pub fn output_image<P: AsRef<Path>>(mut self, output_image: P) -> Self {
        self.output_image = output_image.as_ref().to_path_buf();
        self
    }

    pub fn output_atlas<P: AsRef<Path>>(mut self, output_atlas: P) -> Self {
        self.output_atlas = output_atlas.as_ref().to_path_buf();
        self
    }

    pub fn max_width(mut self, max_width: usize) -> Self {
        self.max_width = max_width;
        self
    }

    pub fn max_height(mut self, max_height: usize) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }

    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }

    pub fn full_names(mut self, full_names: bool) -> Self {
        self.full_names = full_names;
        self
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TiledPhase {
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    input: PathBuf,
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    output: PathBuf,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    spritesheets: Vec<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    full_names: bool,
}

impl TiledPhase {
    pub fn input<P: AsRef<Path>>(mut self, input: P) -> Self {
        self.input = input.as_ref().to_path_buf();
        self
    }

    pub fn output<P: AsRef<Path>>(mut self, output: P) -> Self {
        self.output = output.as_ref().to_path_buf();
        self
    }

    pub fn spritesheets<P: AsRef<Path>>(mut self, spritesheets: &[P]) -> Self {
        self.spritesheets = spritesheets
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect::<Vec<_>>();
        self
    }

    pub fn spritesheet<P: AsRef<Path>>(mut self, spritesheet: P) -> Self {
        self.spritesheets.push(spritesheet.as_ref().to_path_buf());
        self
    }

    pub fn full_names(mut self, full_names: bool) -> Self {
        self.full_names = full_names;
        self
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PackPhase {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    paths: Vec<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    output: PathBuf,
}

impl PackPhase {
    pub fn paths<P: AsRef<Path>>(mut self, paths: &[P]) -> Self {
        self.paths = paths
            .iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect::<Vec<_>>();
        self
    }

    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.paths.push(path.as_ref().to_path_buf());
        self
    }

    pub fn output<P: AsRef<Path>>(mut self, output: P) -> Self {
        self.output = output.as_ref().to_path_buf();
        self
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    source: PathBuf,
    #[serde(default)]
    #[serde(skip_serializing_if = "pathbuf_is_empty")]
    destination: PathBuf,
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    clear_destination: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    quiet: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    commands: Vec<Command>,
}

impl Pipeline {
    pub fn from_file<P: AsRef<Path>>(config: P, project_is_root: bool) -> Result<Self, Error> {
        let config = if project_is_root {
            let root = var("CARGO_MANIFEST_DIR").unwrap();
            Path::new(&root).join(config.as_ref()).to_path_buf()
        } else {
            config.as_ref().to_path_buf()
        };
        let contents = read_to_string(&config)?;
        match serde_json::from_str::<Pipeline>(&contents) {
            Ok(pipeline) => Ok(pipeline),
            Err(error) => Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Could not parse pipeline JSON config: {:?}. Error: {:?}",
                    config, error
                ),
            )),
        }
    }

    pub fn source<P: AsRef<Path>>(mut self, source: P) -> Self {
        self.source = source.as_ref().to_path_buf();
        self
    }

    pub fn project_source<P: AsRef<Path>>(mut self, source: P) -> Self {
        let root = var("CARGO_MANIFEST_DIR").unwrap();
        self.source = Path::new(&root).join(source.as_ref()).to_path_buf();
        self
    }

    pub fn destination<P: AsRef<Path>>(mut self, destination: P) -> Self {
        self.destination = destination.as_ref().to_path_buf();
        self
    }

    pub fn project_destination<P: AsRef<Path>>(mut self, destination: P) -> Self {
        let root = var("CARGO_MANIFEST_DIR").unwrap();
        self.destination = Path::new(&root).join(destination.as_ref()).to_path_buf();
        self
    }

    pub fn clear_destination(mut self, clear_destination: bool) -> Self {
        self.clear_destination = clear_destination;
        self
    }

    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn copy(mut self, phase: CopyPhase) -> Self {
        self.commands.push(Command::Copy(phase));
        self
    }

    pub fn atlas(mut self, phase: AtlasPhase) -> Self {
        self.commands.push(Command::Atlas(phase));
        self
    }

    pub fn tiled(mut self, phase: TiledPhase) -> Self {
        self.commands.push(Command::Tiled(phase));
        self
    }

    pub fn pack(mut self, phase: PackPhase) -> Self {
        self.commands.push(Command::Pack(phase));
        self
    }

    pub fn pipeline(mut self, pipeline: Pipeline) -> Self {
        self.commands.push(Command::Pipeline(pipeline));
        self
    }

    pub fn execute(self) -> Result<(), Error> {
        if self.clear_destination {
            drop(fs_extra::dir::remove(&self.destination));
        }
        drop(fs_extra::dir::create_all(&self.destination, false));
        for command in self.commands.iter().cloned() {
            match command {
                Command::Copy(CopyPhase { from, to }) => {
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
                Command::Atlas(AtlasPhase {
                    paths,
                    output_image,
                    output_atlas,
                    max_width,
                    max_height,
                    padding,
                    pretty,
                    full_names,
                }) => {
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
                    let output_image = if output_image.is_absolute() {
                        output_image
                    } else {
                        self.destination.join(output_image)
                    };
                    let output_atlas = if output_atlas.is_absolute() {
                        output_atlas
                    } else {
                        self.destination.join(output_atlas)
                    };
                    pack_sprites_and_write_to_files(
                        &paths,
                        output_image,
                        output_atlas,
                        max_width,
                        max_height,
                        padding,
                        pretty,
                        full_names,
                        self.quiet,
                    )?;
                }
                Command::Tiled(TiledPhase {
                    input,
                    output,
                    spritesheets,
                    full_names,
                }) => {
                    let input = if input.is_absolute() {
                        input
                    } else {
                        self.source.join(input)
                    };
                    let output = if output.is_absolute() {
                        output
                    } else {
                        self.destination.join(output)
                    };
                    let spritesheets = spritesheets
                        .into_iter()
                        .map(|path| {
                            if path.is_absolute() {
                                path
                            } else {
                                self.source.join(path)
                            }
                        })
                        .collect::<Vec<_>>();
                    build_map_and_write_to_file(
                        input,
                        output,
                        &spritesheets,
                        full_names,
                        self.quiet,
                    )?;
                }
                Command::Pack(PackPhase { paths, output }) => {
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
                    pack_assets_and_write_to_file(&paths, output, self.quiet)?;
                }
                Command::Pipeline(mut pipeline) => {
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

    pub fn dry_run(&self) {
        #[derive(Debug)]
        struct Meta {
            source: PathBuf,
            destination: PathBuf,
            clear_destination: bool,
            quiet: bool,
        }
        println!(
            "* Pipeline: {:#?}",
            Meta {
                source: self.source.clone(),
                destination: self.destination.clone(),
                clear_destination: self.clear_destination,
                quiet: self.quiet
            }
        );
        for command in self.commands.iter().cloned() {
            match command {
                Command::Copy(CopyPhase { from, to }) => {
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
                    println!("* Copy: {:#?}", CopyPhase { from, to });
                }
                Command::Atlas(AtlasPhase {
                    paths,
                    output_image,
                    output_atlas,
                    max_width,
                    max_height,
                    padding,
                    pretty,
                    full_names,
                }) => {
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
                    let output_image = if output_image.is_absolute() {
                        output_image
                    } else {
                        self.destination.join(output_image)
                    };
                    let output_atlas = if output_atlas.is_absolute() {
                        output_atlas
                    } else {
                        self.destination.join(output_atlas)
                    };
                    println!(
                        "* Atlas: {:#?}",
                        AtlasPhase {
                            paths,
                            output_image,
                            output_atlas,
                            max_width,
                            max_height,
                            padding,
                            pretty,
                            full_names,
                        }
                    );
                }
                Command::Tiled(TiledPhase {
                    input,
                    output,
                    spritesheets,
                    full_names,
                }) => {
                    let input = if input.is_absolute() {
                        input
                    } else {
                        self.source.join(input)
                    };
                    let output = if output.is_absolute() {
                        output
                    } else {
                        self.destination.join(output)
                    };
                    let spritesheets = spritesheets
                        .into_iter()
                        .map(|path| {
                            if path.is_absolute() {
                                path
                            } else {
                                self.source.join(path)
                            }
                        })
                        .collect::<Vec<_>>();
                    println!(
                        "* Tiled: {:#?}",
                        TiledPhase {
                            input,
                            output,
                            spritesheets,
                            full_names,
                        }
                    );
                }
                Command::Pack(PackPhase { paths, output }) => {
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
                    println!("* Pack: {:#?}", PackPhase { paths, output });
                }
                Command::Pipeline(mut pipeline) => {
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
                    pipeline.dry_run();
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Command {
    Copy(CopyPhase),
    Atlas(AtlasPhase),
    Tiled(TiledPhase),
    Pack(PackPhase),
    Pipeline(Pipeline),
}
