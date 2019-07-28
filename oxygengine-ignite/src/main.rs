use cargo_metadata::MetadataCommand;
use clap::{App, Arg, SubCommand};
use std::{
    env::current_dir,
    fs::{copy, create_dir_all, read_dir, read_to_string, write},
    io::{Error, ErrorKind, Result},
    path::Path,
    process::Command,
};

fn main() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            SubCommand::with_name("new")
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
                        .help("Project preset (web-composite-game)")
                        .takes_value(false)
                        .required(false)
                        .default_value("web-composite-game"),
                )
                .arg(
                    Arg::with_name("dont-build")
                        .long("dont-build")
                        .help("Prepare project and exit without building it")
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
        let meta = MetadataCommand::new().exec().unwrap();
        let mut root_path = meta
            .packages
            .iter()
            .find_map(|p| {
                if p.name == env!("CARGO_PKG_NAME") {
                    Some(p.manifest_path.clone())
                } else {
                    None
                }
            })
            .unwrap();
        root_path.pop();
        let preset_path = root_path.join("presets").join(preset);
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
        println!("Make project: {:?}", &destination_path);
        println!("* Prepare project structure...");
        copy_dir(&preset_path, &destination_path, &id)?;
        println!("  Done!");
        if !dont_build {
            println!("* Build rust project...");
            Command::new("cargo")
                .arg("build")
                .current_dir(&destination_path)
                .output()?;
            println!("  Done!");
            println!(
                "NOTE: Go to {:?} and run `npm install` before working with project!",
                destination_path
            );
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
