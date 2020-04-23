use cargo_metadata::MetadataCommand;
use clap::{App, Arg, SubCommand};
use dirs::home_dir;
use oxygengine_build_tools::{
    atlas::pack_sprites_and_write_to_files,
    pack::pack_assets_and_write_to_file,
    pipeline::{AtlasPhase, CopyPhase, PackPhase, Pipeline, TiledPhase},
    tiled::build_map_and_write_to_file,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::{current_dir, current_exe, vars},
    fs::{copy, create_dir_all, read_dir, read_to_string, remove_dir_all, write},
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    process::{Command, Stdio},
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
fn main() -> Result<()> {
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
            "https://github.com/PsichiX/Oxygengine/releases/download/{}/oxygengine-presets.pack",
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
                .about("Create new Oxygen Engine project")
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
                .about("Pack assets for Oxygen Engine")
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
                .about("Pack images into sprite sheet (texture atlas) for Oxygen Engine")
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
                .about("Build map for Oxygen Engine from Tiled JSON format")
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
                .about("Execute pipeline for Oxygen Engine")
                .arg(
                    Arg::with_name("config")
                        .short("c")
                        .long("config")
                        .value_name("PATH")
                        .help("Pipeline JSON descriptor file")
                        .takes_value(true)
                        .required(false),
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
        let config = matches
            .value_of("config")
            .unwrap_or_else(|| "pipeline.json");
        let dry_run = matches.is_present("dry-run");
        let template = matches.is_present("template");
        let config = Path::new(&config);
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
            let contents = read_to_string(config)?;
            match serde_json::from_str::<Pipeline>(&contents) {
                Ok(pipeline) => {
                    if dry_run {
                        pipeline.dry_run();
                    } else {
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
    }
    Ok(())
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
