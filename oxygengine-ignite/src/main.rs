use cargo_metadata::MetadataCommand;
use clap::{App, Arg, SubCommand};
use dirs::home_dir;
use hotwatch::{Event, Hotwatch};
use oxygengine_build_tools::{
    atlas::pack_sprites_and_write_to_files,
    build::{build_project, BuildProfile},
    pack::pack_assets_and_write_to_file,
    pipeline::{AtlasPhase, CopyPhase, PackPhase, Pipeline, TiledPhase},
    tiled::build_map_and_write_to_file,
};
use oxygengine_ignite_types::IgniteTypeDefinition;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    env::{current_dir, current_exe, set_current_dir, vars},
    fs::{
        copy, create_dir_all, read, read_dir, read_to_string, remove_dir_all, remove_file, write,
    },
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::channel,
    thread::spawn,
    time::Instant,
};

enum ActionType {
    PreCreate,
    PostCreate,
    PreBuild,
    PostBuild,
}

#[derive(Default, Deserialize)]
struct PresetManifest {
    #[serde(default)]
    pub notes: Actions,
    #[serde(default)]
    pub scripts: Actions,
}

impl PresetManifest {
    pub fn print_note(&self, action: ActionType) {
        if let Some(text) = self.notes.format(action) {
            println!("{}", text);
        }
    }

    pub fn execute_script(&self, action: ActionType, wkdir: &Path) -> Result<()> {
        if let Some(mut text) = self.scripts.format(action) {
            for (key, value) in vars() {
                text = text.replace(&format!("~${}$~", key), &value);
            }
            let parts = text
                .split("<|>")
                .map(|part| part.trim())
                .collect::<Vec<_>>();
            let output = Command::new(parts[0])
                .args(&parts[1..])
                .envs(vars().map(|(k, v)| (k, v)))
                .current_dir(wkdir)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output();
            println!("{}", parts.join(" "));
            output?;
        }
        Ok(())
    }
}

#[derive(Default, Deserialize)]
struct Actions {
    pub precreate: Option<String>,
    pub postcreate: Option<String>,
    pub prebuild: Option<String>,
    pub postbuild: Option<String>,
    #[serde(skip)]
    pub dictionary: HashMap<String, String>,
}

impl Actions {
    pub fn format(&self, action: ActionType) -> Option<String> {
        let text = match action {
            ActionType::PreCreate => &self.precreate,
            ActionType::PostCreate => &self.postcreate,
            ActionType::PreBuild => &self.prebuild,
            ActionType::PostBuild => &self.postbuild,
        };
        let mut text = if let Some(text) = text {
            text.clone()
        } else {
            return None;
        };
        for (key, value) in &self.dictionary {
            text = text.replace(&format!("~%{}%~", key), value);
        }
        Some(text)
    }
}

#[allow(clippy::cognitive_complexity)]
#[tokio::main]
async fn main() -> Result<()> {
    let meta = MetadataCommand::new().exec();
    let mut root_path = if let Ok(meta) = meta {
        meta.packages
            .iter()
            .find_map(|p| {
                if p.name == env!("CARGO_PKG_NAME") {
                    Some(p.manifest_path.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| current_exe().unwrap())
    } else {
        current_exe()?
    };
    root_path.pop();
    let presets_path = if let Ok(path) = std::env::var("OXY_PRESETS_DIR") {
        PathBuf::from(path)
    } else {
        let mut presets_path = root_path.join("presets");
        if !presets_path.exists() {
            presets_path = if let Some(path) = home_dir() {
                path
            } else {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "There is no HOME directory on this machine",
                ));
            }
            .join(".ignite")
            .join("presets");
        }
        presets_path
    };
    let has_presets = if let Ok(iter) = read_dir(&presets_path) {
        iter.count() > 0
    } else {
        false
    };
    let update_presets = std::env::var("OXY_UPDATE_PRESETS").is_ok();
    if !has_presets || update_presets {
        if update_presets {
            drop(remove_dir_all(&presets_path));
        }
        let url = format!(
            "https://oxygengine.io/ignite-presets/oxygengine-presets-{}.pack",
            env!("CARGO_PKG_VERSION")
        );
        println!(
            "There are no presets installed in {:?} - trying to download them now from: {:?}",
            presets_path, url
        );
        let response =
            reqwest::blocking::get(&url).unwrap_or_else(|_| panic!("Request for {:?} failed", url));
        let bytes = response
            .bytes()
            .unwrap_or_else(|_| panic!("Could not get bytes from {:?} response", url));
        let files = bincode::deserialize::<HashMap<String, Vec<u8>>>(bytes.as_ref())
            .unwrap_or_else(|_| panic!("Could not unpack files from {:?} response", url));
        drop(create_dir_all(&presets_path));
        for (fname, bytes) in files {
            let path = presets_path.join(fname);
            let mut dir_path = path.clone();
            dir_path.pop();
            drop(create_dir_all(&dir_path));
            println!("Store file: {:?}", path);
            write(path, bytes)?;
        }
    }
    let presets_list = read_dir(&presets_path)?
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_dir() {
                Some(path.file_name().unwrap().to_str().unwrap().to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    if presets_list.is_empty() {
        return Err(Error::new(
            ErrorKind::NotFound,
            "There are no presets installed - consider reinstalling oxygengine-ignite, it might be corrupted",
        ));
    }

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("new")
                .about("Create new project")
                .arg(
                    Arg::with_name("id")
                        .value_name("ID")
                        .help("Project ID")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("destination")
                        .short("d")
                        .long("destination")
                        .value_name("PATH")
                        .help("Project destination path")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("preset")
                        .short("p")
                        .long("preset")
                        .help(&format!("Project preset ({})", presets_list.join(", ")))
                        .takes_value(false)
                        .required(false)
                        .default_value(presets_list.last().unwrap()),
                )
                .arg(
                    Arg::with_name("dont-build")
                        .long("dont-build")
                        .help("Prepare project and exit without building it")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("quiet")
                        .short("q")
                        .long("quiet")
                        .help("Don't show progress information")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("pack")
                .about("Pack assets")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .value_name("PATH")
                        .help("Assets root folder")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("PATH")
                        .help("Asset pack output file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("quiet")
                        .short("q")
                        .long("quiet")
                        .help("Don't show progress information")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("atlas")
                .about("Pack images into sprite sheet (texture atlas)")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .value_name("PATH")
                        .help("Image file or folder containing images")
                        .takes_value(true)
                        .required(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("output-image")
                        .short("o")
                        .long("output-image")
                        .value_name("PATH")
                        .help("Image output file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output-atlas")
                        .short("a")
                        .long("output-atlas")
                        .value_name("PATH")
                        .help("Spritesheet output file")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("max-width")
                        .short("w")
                        .long("max-width")
                        .value_name("NUMBER")
                        .help("Maximum atlas image width")
                        .takes_value(true)
                        .default_value("2048")
                        .required(false),
                )
                .arg(
                    Arg::with_name("max-height")
                        .short("h")
                        .long("max-height")
                        .value_name("NUMBER")
                        .help("Maximum atlas image height")
                        .takes_value(true)
                        .default_value("2048")
                        .required(false),
                )
                .arg(
                    Arg::with_name("padding")
                        .short("p")
                        .long("padding")
                        .value_name("NUMBER")
                        .help("Padding between atlas images")
                        .takes_value(true)
                        .default_value("2")
                        .required(false),
                )
                .arg(
                    Arg::with_name("pretty")
                        .short("r")
                        .long("pretty")
                        .help("Produce pretty human-readable JSON")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("full-names")
                        .short("f")
                        .long("full-names")
                        .help("Give full name (with parent folders) to frame ID")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("quiet")
                        .short("q")
                        .long("quiet")
                        .help("Don't show progress information")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("tiled")
                .about("Build Tiled JSON file into set of engine map related files")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .value_name("PATH")
                        .help("Tiled JSON file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .short("o")
                        .long("output")
                        .value_name("PATH")
                        .help("Map binary file")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("spritesheet")
                        .short("s")
                        .long("spritesheet")
                        .value_name("PATH")
                        .help("Sprite sheet (texture atlas) JSON file")
                        .takes_value(true)
                        .required(true)
                        .multiple(true),
                )
                .arg(
                    Arg::with_name("full-names")
                        .short("f")
                        .long("full-names")
                        .help("Give full name (with parent folders) to sprites used")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("quiet")
                        .short("q")
                        .long("quiet")
                        .help("Don't show progress information")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("pipeline")
                .about("Execute project pipeline")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("PATH")
                        .help("Pipeline JSON descriptor file")
                        .takes_value(true)
                        .required(false)
                        .default_value("./pipeline.json")
                )
                .arg(
                    Arg::with_name("template")
                        .short("t")
                        .long("template")
                        .help("Create and save pipeline template")
                        .takes_value(false)
                        .required(false),
                )
                .arg(
                    Arg::with_name("dry-run")
                        .short("d")
                        .long("dry-run")
                        .help("Perform dry run (don't execute pipeline, just show its flow)")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("build")
                .about("Build project")
                .arg(
                    Arg::with_name("profile")
                        .short("p")
                        .long("profile")
                        .value_name("PROFILE")
                        .help("Project build profile. Possible values: debug, release")
                        .takes_value(true)
                        .required(false)
                        .default_value("debug"),
                )
                .arg(
                    Arg::with_name("crate_dir")
                        .short("c")
                        .long("crate_dir")
                        .value_name("PATH")
                        .help("Project crate directory")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("out_dir")
                        .short("o")
                        .long("out_dir")
                        .value_name("PATH")
                        .help("Binaries output directory relative to crate directory")
                        .takes_value(true)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("serve")
                .about("Serve project binary and baked asset files to browsers")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .value_name("NUMBER")
                        .help("HTTP server port")
                        .takes_value(true)
                        .required(false)
                        .default_value("8080")
                )
                .arg(
                    Arg::with_name("binaries")
                        .short("b")
                        .long("binaries")
                        .value_name("PATH")
                        .help("Path to the binaries folder")
                        .takes_value(true)
                        .required(false)
                        .default_value("./bin/")
                )
                .arg(
                    Arg::with_name("assets")
                        .short("a")
                        .long("assets")
                        .value_name("PATH")
                        .help("Path to the baked assets folder")
                        .takes_value(true)
                        .required(false)
                        .default_value("./assets-baked/")
                )
                .arg(
                    Arg::with_name("open")
                        .short("o")
                        .long("open")
                        .help("Open URL in the browser")
                        .required(false)
                )
        )
        .subcommand(
            SubCommand::with_name("live")
                .about("Listen for changes in binary and asset sources and rebuild them when they change")
                .arg(
                    Arg::with_name("binaries")
                        .short("b")
                        .long("binaries")
                        .value_name("PATH")
                        .help("Path to the binaries source folders (Rust or JS code)")
                        .takes_value(true)
                        .required(false)
                        .multiple(true)
                )
                .arg(
                    Arg::with_name("assets")
                        .short("a")
                        .long("assets")
                        .value_name("PATH")
                        .help("Path to the assets sources folders")
                        .takes_value(true)
                        .required(false)
                        .multiple(true)
                )
                .arg(
                    Arg::with_name("crate_dir")
                        .short("c")
                        .long("crate_dir")
                        .value_name("PATH")
                        .help("Project crate directory")
                        .takes_value(true)
                        .required(false)
                        .default_value("./")
                )
                .arg(
                    Arg::with_name("pipeline")
                        .short("p")
                        .long("pipeline")
                        .value_name("PATH")
                        .help("Pipeline JSON descriptor file")
                        .takes_value(true)
                        .required(false)
                        .default_value("./pipeline.json")
                )
                .arg(
                    Arg::with_name("serve")
                        .help("Arguments passed to serve subcommand")
                        .multiple(true)
                )
        )
        .subcommand(
            SubCommand::with_name("package")
                .about("Make distribution package of the project")
                .arg(
                    Arg::with_name("crate_dir")
                        .short("c")
                        .long("crate_dir")
                        .value_name("PATH")
                        .help("Crate directory")
                        .takes_value(true)
                        .required(false)
                        .default_value("./")
                )
                .arg(
                    Arg::with_name("pipeline")
                        .short("p")
                        .long("pipeline")
                        .value_name("PATH")
                        .help("Assets pipeline config file")
                        .takes_value(true)
                        .required(false)
                        .default_value("./pipeline.json")
                )
                .arg(
                    Arg::with_name("assets")
                        .short("a")
                        .long("assets")
                        .value_name("PATH")
                        .help("Baked assets directory")
                        .takes_value(true)
                        .required(false)
                        .default_value("./assets-baked/")
                )
                .arg(
                    Arg::with_name("out_dir")
                        .short("o")
                        .long("out_dir")
                        .value_name("PATH")
                        .help("Output directory relative to crate directory")
                        .takes_value(true)
                        .required(false)
                        .default_value("./dist/")
                )
        )
        .subcommand(
            SubCommand::with_name("types")
                .about("Performs operation on ignite type definitions")
                .subcommand(
                    SubCommand::with_name("validate")
                        .about("Validate types")
                        .arg(
                            Arg::with_name("path")
                                .short("p")
                                .long("path")
                                .value_name("PATH")
                                .help("Path to the crate root directory")
                                .takes_value(true)
                                .required(false)
                        )
                        .arg(
                            Arg::with_name("ignore")
                                .short("i")
                                .long("ignore")
                                .value_name("NAME")
                                .help("Names of types to ignore in report")
                                .takes_value(true)
                                .required(false)
                                .multiple(true)
                        )
                        .arg(
                            Arg::with_name("ignore-file")
                                .short("f")
                                .long("ignore-file")
                                .value_name("PATH")
                                .help("Path to list of type names to ignore in report")
                                .takes_value(true)
                                .required(false)
                        )
                )
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("new") {
        let id = matches.value_of("id").unwrap();
        let destination = matches.value_of("destination");
        let preset = matches.value_of("preset").unwrap();
        let dont_build = matches.is_present("dont-build");
        let quiet = matches.is_present("quiet");
        let preset_path = presets_path.join(preset);
        if !preset_path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Preset not found: {} (in path: {:?})", preset, preset_path),
            ));
        }
        let mut destination_path = if let Some(destination) = destination {
            destination.into()
        } else {
            current_dir()?
        };
        destination_path.push(id);
        if let Err(err) = create_dir_all(&destination_path) {
            if err.kind() != ErrorKind::AlreadyExists {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Could not create directory: {:?}", destination_path),
                ));
            }
        }

        let preset_manifest_path = presets_path.join(format!("{}.toml", preset));
        let preset_manifest = if preset_manifest_path.exists() {
            let contents = read_to_string(&preset_manifest_path)?;
            if let Ok(mut manifest) = toml::from_str::<PresetManifest>(&contents) {
                manifest.notes.dictionary.insert(
                    "IGNITE_DESTINATION_PATH".to_owned(),
                    destination_path.to_str().unwrap().to_owned(),
                );
                manifest.scripts.dictionary = manifest.notes.dictionary.clone();
                Some(manifest)
            } else {
                None
            }
        } else {
            None
        };

        if !quiet {
            println!("Make project: {:?}", &destination_path);
            if let Some(preset_manifest) = &preset_manifest {
                preset_manifest.print_note(ActionType::PreCreate);
            }
            println!("* Prepare project structure...");
        }
        if let Some(preset_manifest) = &preset_manifest {
            preset_manifest.execute_script(ActionType::PreCreate, &destination_path)?;
        }
        copy_dir(&preset_path, &destination_path, &id)?;
        if let Some(preset_manifest) = &preset_manifest {
            preset_manifest.execute_script(ActionType::PostCreate, &destination_path)?;
        }
        if !quiet {
            println!("  Done!");
            if let Some(preset_manifest) = &preset_manifest {
                preset_manifest.print_note(ActionType::PostCreate);
            }
        }
        if !dont_build {
            if !quiet {
                if let Some(preset_manifest) = &preset_manifest {
                    preset_manifest.print_note(ActionType::PreBuild);
                }
                println!("* Build rust project...");
            }
            if let Some(preset_manifest) = &preset_manifest {
                preset_manifest.execute_script(ActionType::PreBuild, &destination_path)?;
            }
            Command::new("cargo")
                .arg("build")
                .current_dir(&destination_path)
                .output()?;
            if let Some(preset_manifest) = &preset_manifest {
                preset_manifest.execute_script(ActionType::PostBuild, &destination_path)?;
            }
            if !quiet {
                println!("  Done!");
                if let Some(preset_manifest) = &preset_manifest {
                    preset_manifest.print_note(ActionType::PostBuild);
                }
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("pack") {
        let input = matches.values_of("input").unwrap().collect::<Vec<_>>();
        let output = matches.value_of("output").unwrap();
        let quiet = matches.is_present("quiet");
        pack_assets_and_write_to_file(&input, output, quiet)?;
    } else if let Some(matches) = matches.subcommand_matches("atlas") {
        let input = matches.values_of("input").unwrap().collect::<Vec<_>>();
        let output_image = matches.value_of("output-image").unwrap();
        let output_atlas = &if let Some(path) = matches.value_of("output-atlas") {
            path.to_owned()
        } else {
            Path::new(output_image)
                .with_extension("json")
                .to_str()
                .unwrap()
                .to_owned()
        };
        let max_width = matches.value_of("max-width").unwrap().parse().unwrap();
        let max_height = matches.value_of("max-height").unwrap().parse().unwrap();
        let padding = matches.value_of("padding").unwrap().parse().unwrap();
        let pretty = matches.is_present("pretty");
        let full_names = matches.is_present("full-names");
        let quiet = matches.is_present("quiet");
        pack_sprites_and_write_to_files(
            &input,
            output_image,
            output_atlas,
            max_width,
            max_height,
            padding,
            pretty,
            full_names,
            quiet,
        )?;
    } else if let Some(matches) = matches.subcommand_matches("tiled") {
        let input = matches.value_of("input").unwrap();
        let output = matches.value_of("output").unwrap();
        let spritesheets = matches
            .values_of("spritesheet")
            .unwrap()
            .collect::<Vec<_>>();
        let full_names = matches.is_present("full-names");
        let quiet = matches.is_present("quiet");
        build_map_and_write_to_file(input, output, &spritesheets, full_names, quiet)?;
    } else if let Some(matches) = matches.subcommand_matches("pipeline") {
        let config = matches.value_of("config").unwrap();
        let config = Path::new(&config).to_owned();
        let dry_run = matches.is_present("dry-run");
        let template = matches.is_present("template");
        if template {
            let pipeline = Pipeline::default()
                .source("static")
                .destination("static")
                .pipeline(
                    Pipeline::default()
                        .destination("assets-generated")
                        .clear_destination(true),
                )
                .pipeline(
                    Pipeline::default()
                        .source("assets-source")
                        .destination("assets-generated")
                        .copy(CopyPhase::default().from("assets.txt"))
                        .atlas(
                            AtlasPhase::default()
                                .path("images")
                                .output_image("sprites.png")
                                .output_atlas("sprites.json")
                                .pretty(true),
                        ),
                )
                .pipeline(
                    Pipeline::default().destination("assets-generated").tiled(
                        TiledPhase::default()
                            .input("assets-source/maps/map.json")
                            .spritesheet("assets-generated/sprites.0.json")
                            .output("map.map"),
                    ),
                )
                .pack(
                    PackPhase::default()
                        .path("assets-generated")
                        .output("assets.pack"),
                );
            let contents = match serde_json::to_string_pretty(&pipeline) {
                Ok(contents) => contents,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!(
                            "Could not stringify pipeline JSON config: {:?}. Error: {:?}",
                            config, error
                        ),
                    ))
                }
            };
            if dry_run {
                println!("Pipeline: {}", contents);
            } else {
                write(config, &contents)?;
            }
        } else if config.exists() {
            let contents = read_to_string(&config)?;
            match serde_json::from_str::<Pipeline>(&contents) {
                Ok(pipeline) => {
                    if dry_run {
                        pipeline.dry_run();
                    } else {
                        set_current_dir(if config.is_file() {
                            let mut path = config.clone();
                            path.pop();
                            path
                        } else {
                            config.clone()
                        })?;
                        pipeline.execute()?;
                    }
                }
                Err(error) => println!(
                    "Could not parse pipeline JSON config: {:?}. Error: {:?}",
                    config, error
                ),
            }
        } else {
            println!("Could not find pipeline config file: {:?}", config);
        }
    } else if let Some(matches) = matches.subcommand_matches("build") {
        let profile = match matches.value_of("profile").unwrap() {
            "debug" => BuildProfile::Debug,
            "release" => BuildProfile::Release,
            profile => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Unknown build profile: {}", profile),
                ))
            }
        };
        let crate_dir = matches.value_of("crate_dir").map(|v| v.to_owned());
        let out_dir = matches.value_of("out_dir").map(|v| v.to_owned());
        build_project(profile, crate_dir, out_dir, vec![])?;
    } else if let Some(matches) = matches.subcommand_matches("serve") {
        let port = matches
            .value_of("port")
            .unwrap()
            .parse::<u16>()
            .expect("Could not parse port number");
        let binaries = matches.value_of("binaries").unwrap().to_owned();
        let assets = matches.value_of("assets").unwrap().to_owned();
        if matches.is_present("open") {
            open::that(format!("http://localhost:{}", port))
                .expect("Could not open URL in the browser");
        }
        serve_files(port, binaries, assets).await;
    } else if let Some(matches) = matches.subcommand_matches("live") {
        let binaries = matches
            .values_of("binaries")
            .map(|v| v.collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["./src"]);
        let assets = matches
            .values_of("assets")
            .map(|v| v.collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["./assets"]);
        let crate_dir = matches.value_of("crate_dir").unwrap().to_owned();
        let pipeline = matches.value_of("pipeline").unwrap().to_owned();
        let serve = matches.values_of("serve");
        let mut watcher = Hotwatch::new().expect("Could not start files watcher");
        let (build_sender, build_receiver) = channel();
        let (pipeline_sender, pipeline_receiver) = channel();
        drop(build_sender.send(()));
        drop(pipeline_sender.send(()));
        for path in binaries {
            let build_sender = build_sender.clone();
            watcher
                .watch(&path, move |event| match event {
                    Event::Create(_) | Event::Write(_) | Event::Remove(_) => {
                        if build_sender.send(()).is_ok() {
                            println!("* Rebuild project binaries");
                        }
                    }
                    _ => {}
                })
                .unwrap_or_else(|_| panic!("Could not watch for binaries sources: {:?}", path));
            println!("* Watching binaries sources: {:?}", path);
        }
        for path in assets {
            let pipeline_sender = pipeline_sender.clone();
            watcher
                .watch(&path, move |event| match event {
                    Event::Create(_) | Event::Write(_) | Event::Remove(_) => {
                        if pipeline_sender.send(()).is_ok() {
                            println!("* Rebake project assets");
                        }
                    }
                    _ => {}
                })
                .unwrap_or_else(|_| panic!("Could not watch for assets sources: {:?}", path));
            println!("* Watching assets sources: {:?}", path);
        }
        let builds = spawn(move || {
            let exe = current_exe().expect("Could not get path to the running executable");
            while build_receiver.recv().is_ok() {
                if let Err(error) = Command::new(&exe)
                    .arg("build")
                    .arg("--crate_dir")
                    .arg(&crate_dir)
                    .status()
                {
                    println!("Error during build process execution: {:#?}", error);
                } else {
                    println!("* Done building binaries!");
                }
            }
        });
        let pipelines = spawn(move || {
            let exe = current_exe().expect("Could not get path to the running executable");
            while pipeline_receiver.recv().is_ok() {
                if let Err(error) = Command::new(&exe)
                    .arg("pipeline")
                    .arg("--config")
                    .arg(&pipeline)
                    .status()
                {
                    println!("* Error during pipeline process execution: {:#?}", error);
                } else {
                    println!("* Done baking assets!");
                }
            }
        });
        let exe = current_exe().expect("Could not get path to the running executable");
        if let Some(serve) = serve {
            Command::new(exe)
                .arg("serve")
                .args(serve)
                .status()
                .expect("Could not run HTTP server");
        }
        drop(builds.join());
        drop(pipelines.join());
    } else if let Some(matches) = matches.subcommand_matches("package") {
        let crate_dir = matches.value_of("crate_dir").unwrap();
        let pipeline = matches.value_of("pipeline").unwrap();
        let assets = Path::new(matches.value_of("assets").unwrap());
        let out_dir = matches.value_of("out_dir").unwrap();
        let exe = current_exe().expect("Could not get path to the running executable");
        let timer = Instant::now();
        println!("* Building application in release mode");
        Command::new(&exe)
            .arg("build")
            .arg("--profile")
            .arg("release")
            .arg("--crate_dir")
            .arg(crate_dir)
            .arg("--out_dir")
            .arg(out_dir)
            .status()
            .expect("Could not run build in release mode");
        let out_dir = Path::new(crate_dir).join(out_dir);
        remove_file(out_dir.join(".gitignore")).expect("Could not remove .gitignore file");
        remove_file(out_dir.join("package.json")).expect("Could not remove package.json file");
        println!("* Executing assets pipeline");
        Command::new(exe)
            .arg("pipeline")
            .arg("--config")
            .arg(pipeline)
            .status()
            .expect("Could not run assets pipeline");
        println!("* Copying assets to output directory");
        copy_dir(assets, &out_dir, "").expect("Could not copy baked assets to output directory");
        println!("* Done in: {:?}", timer.elapsed());
    } else if let Some(matches) = matches.subcommand_matches("types") {
        if let Some(matches) = matches.subcommand_matches("validate") {
            let path = if let Some(path) = matches.value_of("path") {
                Path::new(path).to_owned()
            } else {
                std::env::current_dir().unwrap()
            };
            let path = path.join("target").join("ignite").join("types");
            if !path.is_dir() {
                panic!("Ignite types directory does not exists: {:?}", path);
            }
            let mut ignore = if let Some(ignore) = matches.values_of("ignore") {
                ignore.map(|item| item.to_owned()).collect::<Vec<_>>()
            } else {
                vec![]
            };
            if let Some(path) = matches.value_of("ignore-file") {
                let contents = read_to_string(path).expect("Could not read ignored types file");
                for line in contents.lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        ignore.push(line.to_owned());
                    }
                }
            }
            let types = path
                .read_dir()
                .expect("Could not scan ignite types directory")
                .filter_map(|entry| {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            let extension = path
                                .extension()
                                .unwrap_or_else(|| panic!("Unknown file type: {:?}", path));
                            let extension = extension.to_str().unwrap_or_else(|| {
                                panic!("Could not parse file extension: {:?}", path)
                            });
                            let definition = {
                                match extension {
                                    "yaml" => {
                                        let contents = read_to_string(path.to_owned())
                                            .unwrap_or_else(|_| {
                                                panic!(
                                                    "Could not read ignite type file: {:?}",
                                                    path
                                                )
                                            });
                                        serde_yaml::from_str::<IgniteTypeDefinition>(&contents)
                                            .expect("Could not parse YAML type definition file")
                                    }
                                    "json" => {
                                        let contents = read_to_string(path.to_owned())
                                            .unwrap_or_else(|_| {
                                                panic!(
                                                    "Could not read ignite type file: {:?}",
                                                    path
                                                )
                                            });
                                        serde_json::from_str::<IgniteTypeDefinition>(&contents)
                                            .expect("Could not parse YAML type definition file")
                                    }
                                    "ron" => {
                                        let contents = read_to_string(path.to_owned())
                                            .unwrap_or_else(|_| {
                                                panic!(
                                                    "Could not read ignite type file: {:?}",
                                                    path
                                                )
                                            });
                                        ron::from_str::<IgniteTypeDefinition>(&contents)
                                            .expect("Could not parse YAML type definition file")
                                    }
                                    "bin" => {
                                        let contents = read(path.to_owned()).unwrap_or_else(|_| {
                                            panic!("Could not read ignite type file: {:?}", path)
                                        });
                                        bincode::deserialize::<IgniteTypeDefinition>(&contents)
                                            .expect("Could not parse YAML type definition file")
                                    }
                                    extension => panic!("Unsupported file type: {:?}", extension),
                                }
                            };
                            let key = definition.name();
                            let value = definition.referenced();
                            return Some((key, value));
                        }
                    }
                    None
                })
                .collect::<HashMap<_, _>>();
            let referenced = types.values().cloned().flatten().collect::<HashSet<_>>();
            let type_names = types.keys().cloned().collect::<HashSet<_>>();
            let mut diff = referenced.difference(&type_names).collect::<Vec<_>>();
            diff.sort();
            println!("* Referenced types without definition:");
            for name in diff {
                if !ignore.contains(name) {
                    println!("{}:", name);
                    for (n, item) in &types {
                        if item.contains(name) {
                            println!("- {}", n);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn serve_files(port: u16, binaries: String, assets: String) {
    println!("* Serve project files at: localhost:{}", port);
    println!("- binaries:\t{:?}", binaries);
    println!("- assets:\t{:?}", assets);
    use warp::Filter;
    let binaries = warp::fs::dir(binaries);
    let assets = warp::fs::dir(assets);
    let routes = warp::get().and(binaries.or(assets));
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

fn copy_dir(from: &Path, to: &Path, id: &str) -> Result<()> {
    if from.is_dir() {
        for entry in read_dir(from)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let dir = to.join(path.file_name().unwrap());
                create_dir_all(&dir)?;
                copy_dir(&path, &dir, id)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "chrobry" {
                        if let Ok(contents) = read_to_string(&path) {
                            let mut vars = HashMap::new();
                            vars.insert("IGNITE_ID".to_owned(), id.to_owned());
                            match chrobry_core::generate(&contents, "\n", vars, |_| Ok("".to_owned())) {
                                Ok(contents) => {
                                    let to = to.join(path.file_stem().unwrap());
                                    write(to, contents)?;
                                }
                                Err(error) => println!(
                                    "Could not generate file from Chrobry template: {:?}. Error: {:?}",
                                    path, error
                                ),
                            }
                        } else {
                            println!("Could not open Chrobry template file: {:?}", path);
                        }
                        return Ok(());
                    }
                }
                let to = to.join(path.file_name().unwrap());
                if let Ok(contents) = read_to_string(&path) {
                    let contents = contents.replace("~%IGNITE_ID%~", id);
                    write(to, contents)?;
                } else {
                    copy(&path, to)?;
                }
            }
        }
    }
    Ok(())
}
