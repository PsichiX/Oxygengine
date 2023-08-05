use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::{create_dir_all, read_dir, read_to_string, write},
    io::{Error, ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[allow(clippy::trivially_copy_pass_by_ref)]
fn bool_is_false(value: &bool) -> bool {
    !value
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetPhase {
    #[default]
    SourceToIntermediate,
    SourceToBaked,
    IntermediateToBaked,
}

impl AssetPhase {
    fn is_default(&self) -> bool {
        self == &Self::SourceToIntermediate
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetInput {
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    pub ignore: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "AssetPhase::is_default")]
    pub phase: AssetPhase,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source: Vec<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub target: Vec<PathBuf>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Pipeline::is_default")]
    pub pipeline: Pipeline,
}

impl AssetInput {
    pub fn bake_assets_list(path: impl AsRef<Path>, assets: &str) -> std::io::Result<()> {
        let path = path.as_ref();
        if !path.is_dir() {
            return Ok(());
        }
        for entry in read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let assets = if assets.is_empty() {
                    entry.file_name().to_string_lossy().as_ref().to_owned()
                } else {
                    format!(
                        "{}/{}",
                        assets,
                        entry.file_name().to_string_lossy().as_ref()
                    )
                };
                Self::bake_assets_list(&path, &assets)?;
            }
        }
        let mut result = Vec::default();
        for entry in read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let asset = if assets.is_empty() {
                format!("meta://{}", entry.file_name().to_string_lossy().as_ref())
            } else {
                format!(
                    "meta://{}/{}",
                    assets,
                    entry.file_name().to_string_lossy().as_ref()
                )
            };
            if path.is_file()
                && path
                    .extension()
                    .map(|extension| extension == "asset")
                    .unwrap_or_default()
            {
                result.push(asset);
            } else if path.is_dir() {
                result.push(format!("{}/assets.asset", asset));
            }
        }
        let path = path.join("assets.asset");
        match serde_json::to_string_pretty(&AssetOutput { target: result }) {
            Ok(contents) => {
                write(path, contents)?;
            }
            Err(error) => {
                println!(
                    "Could not serialize meta asset JSON config: {:?}. Error: {:?}",
                    path, error
                );
            }
        }
        Ok(())
    }

    pub fn load_and_execute(
        phase: AssetPhase,
        source: impl AsRef<Path>,
        target: impl AsRef<Path>,
        assets: &str,
        tags: &[String],
    ) -> std::io::Result<()> {
        let mut source = source.as_ref().to_owned();
        let target = target.as_ref().to_owned();
        if source.is_file()
            && source
                .extension()
                .map(|extension| extension == "asset")
                .unwrap_or_default()
        {
            let contents = read_to_string(&source)?;
            match serde_json::from_str::<AssetInput>(&contents) {
                Ok(asset) => {
                    if !asset.target.is_empty() || asset.ignore || asset.phase != phase {
                        return Ok(());
                    }
                    for tag in asset.tags {
                        if let Some(tag) = tag.strip_prefix('!') {
                            if tags.iter().any(|t| t == tag) {
                                return Ok(());
                            }
                        } else if !tags.iter().any(|t| t == &tag) {
                            return Ok(());
                        }
                    }
                    asset.pipeline.verify_used_plugins();
                    if source.is_file() {
                        source.pop();
                    }
                    let source = asset
                        .source
                        .iter()
                        .map(|path| source.join(path))
                        .collect::<Vec<_>>();
                    let directory = target.with_extension("");
                    let assets = assets.strip_suffix(".asset").unwrap_or(assets);
                    let result = asset.pipeline.execute(&source, directory, assets)?;
                    if !result.is_empty() {
                        match serde_json::to_string_pretty(&AssetOutput { target: result }) {
                            Ok(contents) => {
                                write(target, contents)?;
                            }
                            Err(error) => {
                                println!(
                                    "Could not serialize meta asset JSON config: {:?}. Error: {:?}",
                                    source, error
                                );
                            }
                        }
                    }
                }
                Err(error) => println!(
                    "Could not parse pipeline asset JSON config: {:?}. Error: {:?}",
                    source, error
                ),
            }
        } else if source.is_dir() {
            for entry in read_dir(source)? {
                let entry = entry?;
                let source = entry.path();
                let target = target.join(entry.file_name());
                let assets = if assets.is_empty() {
                    entry.file_name().to_string_lossy().as_ref().to_owned()
                } else {
                    format!(
                        "{}/{}",
                        assets,
                        entry.file_name().to_string_lossy().as_ref()
                    )
                };
                Self::load_and_execute(phase, source, target, &assets, tags)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AssetOutput {
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub target: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pipeline {
    #[default]
    Copy,
    Generate(Box<AssetInput>),
    Pack {
        #[serde(default = "Pipeline::default_pack_name")]
        name: String,
    },
    Plugin {
        #[serde(default)]
        #[serde(skip_serializing_if = "bool_is_false")]
        do_not_verify: bool,
        name: String,
        params: Value,
    },
    Shell {
        command: String,
    },
}

impl Pipeline {
    fn is_default(&self) -> bool {
        self == &Self::Copy
    }

    fn default_pack_name() -> String {
        "assets".to_owned()
    }

    fn verify_used_plugins(&self) {
        if let Self::Plugin {
            name,
            do_not_verify: false,
            ..
        } = self
        {
            if which::which(name).is_err() {
                let plugin = name.rfind('-').map(|index| &name[0..index]).unwrap_or(name);
                let plugin = format!("{}-tools", plugin);
                println!(
                    "Plugin not found: {}. Trying to install it: {}",
                    name, plugin
                );
                let _ = Command::new("cargo").arg("install").arg(plugin).status();
            }
        }
    }

    fn execute(
        &self,
        source: &[impl AsRef<Path>],
        target: impl AsRef<Path>,
        assets: &str,
    ) -> Result<Vec<String>, Error> {
        match self {
            Self::Copy => {
                let mut target = target.as_ref().to_owned();
                target.pop();
                create_dir_all(&target)?;
                let mut options = fs_extra::dir::CopyOptions::new();
                options.overwrite = true;
                options.copy_inside = true;
                if let Err(error) = fs_extra::copy_items(source, target, &options) {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Could not copy files: {:?}", error),
                    ));
                }
                Ok(vec![])
            }
            Self::Generate(value) => {
                let mut directory = target.as_ref().to_owned();
                directory.pop();
                create_dir_all(directory)?;
                let target = target.as_ref().with_extension("asset");
                match serde_json::to_string_pretty(value) {
                    Ok(contents) => {
                        write(target, contents)?;
                    }
                    Err(error) => {
                        println!(
                            "Could not serialize meta asset JSON config: {:?}. Error: {:?}",
                            value, error
                        );
                    }
                }
                Ok(vec![])
            }
            Self::Pack { name } => {
                let mut target = target.as_ref().to_owned();
                target.pop();
                create_dir_all(&target)?;
                crate::pack::pack_assets_and_write_to_file(
                    source,
                    target.join(name).with_extension("pack"),
                )?;
                Ok(vec![])
            }
            Self::Plugin { name, params, .. } => {
                create_dir_all(target.as_ref())?;
                let mut child = Command::new(name)
                    .args(source.iter().map(|source| source.as_ref().as_os_str()))
                    .arg("--")
                    .arg(target.as_ref().as_os_str())
                    .arg("--")
                    .arg(assets)
                    .arg("--")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .unwrap_or_else(|_| {
                        panic!(
                            "{1}: {0}. {2}: `cargo install {0}`",
                            name,
                            "Could not run plugin",
                            "Make sure it is installed or install it with",
                        )
                    });
                let mut stdin = child.stdin.take().unwrap_or_else(|| {
                    panic!("Could not take input stream of plugin process: {}", name)
                });
                let params = serde_json::to_string(&params)
                    .unwrap_or_else(|_| panic!("Could not serialize params for plugin: {}", name));
                stdin.write_all(params.as_bytes()).unwrap_or_else(|_| {
                    panic!("Could not write serialized parameters for plugin: {}", name)
                });
                stdin.flush().unwrap_or_else(|_| {
                    panic!("Could not complete sending data to plugin: {}", name)
                });
                drop(stdin);
                let output = child
                    .wait_with_output()
                    .unwrap_or_else(|_| panic!("Could not wait for plugin: {}", name));
                if !output.status.success() {
                    panic!(
                        "Plugin run failed: {}. Output: {}. Error: {}",
                        name,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Ok(
                    serde_json::from_slice::<Vec<String>>(&output.stdout).unwrap_or_else(|_| {
                        panic!("Could not deserialize assets list from plugin: {}", name)
                    }),
                )
            }
            Self::Shell { command } => {
                create_dir_all(target.as_ref())?;
                for source in source {
                    let command = command
                        .replace("<source>", source.as_ref().to_string_lossy().as_ref())
                        .replace("<target>", target.as_ref().to_string_lossy().as_ref())
                        .replace("<assets>", assets);
                    let parts = command.split(char::is_whitespace).collect::<Vec<_>>();
                    let mut command = Command::new(parts[0]);
                    for part in &parts[1..] {
                        command.arg(part);
                    }
                    command
                        .status()
                        .unwrap_or_else(|_| panic!("Shell command failed to run: {:?}", command));
                }
                Ok(vec![])
            }
        }
    }
}
