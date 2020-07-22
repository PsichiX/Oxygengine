use std::io::{Error, ErrorKind};
use wasm_pack::{
    command::{
        build::{BuildOptions, Target},
        run_wasm_pack, Command,
    },
    install::InstallMode,
};

#[derive(Debug, Copy, Clone)]
pub enum BuildProfile {
    Debug,
    Release,
}

impl Default for BuildProfile {
    fn default() -> Self {
        Self::Debug
    }
}

pub fn build_project(
    profile: BuildProfile,
    crate_dir: Option<String>,
    out_dir: Option<String>,
    extra_options: Vec<String>,
) -> Result<(), Error> {
    let options = BuildOptions {
        path: crate_dir.map(|p| p.into()),
        scope: None,
        mode: InstallMode::Noinstall,
        disable_dts: true,
        target: Target::Web,
        debug: false,
        dev: match profile {
            BuildProfile::Debug => true,
            _ => false,
        },
        release: match profile {
            BuildProfile::Release => true,
            _ => false,
        },
        profiling: false,
        out_dir: out_dir.unwrap_or_else(|| "bin".to_owned()),
        out_name: Some("index".to_owned()),
        extra_options,
    };
    match run_wasm_pack(Command::Build(options)) {
        Ok(_) => Ok(()),
        Err(error) => Err(Error::new(
            ErrorKind::NotFound,
            format!("Build error: {:?}", error),
        )),
    }
}
