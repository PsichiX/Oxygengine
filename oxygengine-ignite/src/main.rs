use cargo_metadata::MetadataCommand;
use clap::{App, Arg, SubCommand};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::{current_dir, current_exe, vars},
    fs::{copy, create_dir_all, read_dir, read_to_string, write},
    io::{Error, ErrorKind, Result},
    path::Path,
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
    let presets_path = root_path.join("presets");
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
