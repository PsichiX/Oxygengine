use crate::{binary_artifacts_paths, Profile};
use std::{
    fs::{copy, create_dir_all},
    io::{Error, ErrorKind},
    path::PathBuf,
    process::Command as StdCommand,
};
use wasm_pack::{
    command::{
        build::{BuildOptions, Target},
        run_wasm_pack, Command,
    },
    install::InstallMode,
};

pub fn build_project(
    profile: Profile,
    crate_dir: PathBuf,
    out_dir: PathBuf,
    platform: String,
    wasm: bool,
    extra_options: Vec<String>,
) -> Result<(), Error> {
    if wasm {
        let options = BuildOptions {
            path: Some(crate_dir.join("platforms").join(platform)),
            scope: None,
            mode: InstallMode::Normal,
            disable_dts: true,
            target: Target::Web,
            debug: false,
            dev: matches!(profile, Profile::Debug),
            release: matches!(profile, Profile::Release),
            profiling: false,
            out_dir: PathBuf::from("../..")
                .join(out_dir)
                .to_string_lossy()
                .to_string(),
            out_name: Some("index".to_owned()),
            extra_options,
            no_pack: true,
            ..Default::default()
        };
        match run_wasm_pack(Command::Build(options)) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::new(
                ErrorKind::NotFound,
                format!("Build error: {:?}", error),
            )),
        }
    } else {
        let config = format!("platforms/{}/Cargo.toml", platform);
        let mut command = StdCommand::new("cargo");
        command.arg("build");
        if matches!(profile, Profile::Release) {
            command.arg("--release");
        }
        command.arg("--manifest-path");
        command.arg(&config);
        command.arg("--");
        command.args(extra_options);
        command.current_dir(crate_dir);
        command.status()?;
        if let Ok(paths) = binary_artifacts_paths(profile.as_str(), &config) {
            create_dir_all(&out_dir)?;
            for from in paths {
                let to = out_dir.join(
                    from.file_name()
                        .unwrap_or_else(|| panic!("Path does not have file name: {:?}", from)),
                );
                if let Err(error) = copy(&from, &to) {
                    println!(
                        "Could not copy file from: {:?} to: {:?}. Error: {}",
                        from, to, error
                    );
                }
            }
        }
        Ok(())
    }
}
