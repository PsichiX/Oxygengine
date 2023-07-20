mod build;
mod pack;
mod pipeline;

use crate::{
    build::build_project,
    pack::pack_assets_and_write_to_file,
    pipeline::{Pipeline, PipelineCommand},
};
use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand};
use dirs::home_dir;
use hotwatch::{Event, Hotwatch};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env::{current_dir, current_exe, set_current_dir},
    fs::{copy, create_dir_all, read_dir, read_to_string, remove_dir_all, remove_file, write},
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Condvar, Mutex},
    thread::spawn,
    time::Instant,
};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create new project.
    New {
        /// Project ID.
        #[arg(value_name = "ID")]
        id: String,
        /// Project destination path.
        #[arg(short, long, value_name = "PATH", default_value = "./")]
        destination: PathBuf,
        /// Project preset.
        #[arg(short, long, value_name = "NAME", default_value = "base")]
        preset: String,
    },
    /// Pack assets.
    Pack {
        /// Assets root folder.
        #[arg(short, long, value_name = "PATH")]
        input: Vec<PathBuf>,
        /// Assets pack output file.
        #[arg(short, long, value_name = "PATH")]
        output: PathBuf,
    },
    /// Execute project pipeline.
    Pipeline {
        /// Pipeline JSON descriptor file.
        #[arg(value_name = "PATH", default_value = "./pipeline.json")]
        config: PathBuf,
        /// Create and save pipeline template.
        #[arg(short, long)]
        template: bool,
    },
    /// Build project.
    Build {
        /// Project build profile. Possible values: debug, release.
        #[arg(short, long, value_name = "NAME", default_value = "debug")]
        profile: String,
        /// Project crate directory.
        #[arg(short, long, value_name = "PATH", default_value = "./")]
        crate_dir: PathBuf,
        /// Binaries output directory relative to crate directory.
        #[arg(short, long, value_name = "PATH", default_value = "./bin/")]
        out_dir: PathBuf,
        /// Platform name to pack assets for.
        #[arg(value_name = "NAME")]
        platform: String,
        /// Run WASM build instead of cargo build.
        #[arg(short, long)]
        wasm: bool,
        /// Extra arguments passed to build executable.
        #[arg(last = true)]
        extras: Vec<String>,
    },
    /// Serve project binary and baked asset files to browsers.
    Serve {
        /// HTTP server port number.
        #[arg(short, long, value_name = "NUMBER", default_value = "8080")]
        port: u16,
        /// Path to binaries folder.
        #[arg(short, long, value_name = "PATH", default_value = "./bin/")]
        binaries: PathBuf,
        /// Path to baked assets folder.
        #[arg(short, long, value_name = "PATH", default_value = "./assets-baked/")]
        assets: PathBuf,
        /// Open URL in the browser.
        #[arg(short, long)]
        open: bool,
    },
    /// Serve directory files to browsers.
    ServeDir {
        /// HTTP server port number.
        #[arg(short, long, value_name = "NUMBER", default_value = "8080")]
        port: u16,
        /// Path to root folder.
        #[arg(short, long, value_name = "PATH", default_value = "./")]
        root: PathBuf,
        /// Open URL in the browser.
        #[arg(short, long)]
        open: bool,
    },
    /// Listen for changes in binary and asset sources and rebuild them when they change.
    Live {
        /// Project build profile. Possible values: debug, release.
        #[arg(short, long, value_name = "NAME", default_value = "debug")]
        profile: String,
        /// Path to source code folder.
        #[arg(short, long, value_name = "PATH", default_value = "./src/")]
        sources: Vec<PathBuf>,
        /// Path to source assets folder.
        #[arg(short, long, value_name = "PATH", default_value = "./assets/")]
        assets: Vec<PathBuf>,
        /// Project crate directory.
        #[arg(short, long, value_name = "PATH", default_value = "./")]
        crate_dir: PathBuf,
        /// Platform name to pack assets for.
        #[arg(value_name = "NAME")]
        platform: String,
        /// Run WASM build instead of cargo build.
        #[arg(short, long)]
        wasm: bool,
        /// Optional additional command to run.
        command: Option<String>,
        #[arg(allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Make distribution package of the project.
    Package {
        /// Project build profile. Possible values: debug, release.
        #[arg(short, long, value_name = "NAME", default_value = "release")]
        profile: String,
        /// Path to baked assets folder.
        #[arg(short, long, value_name = "PATH", default_value = "./assets-baked/")]
        assets: PathBuf,
        /// Project crate directory.
        #[arg(short, long, value_name = "PATH", default_value = "./")]
        crate_dir: PathBuf,
        /// Artifacts output directory relative to crate directory.
        #[arg(short, long, value_name = "PATH", default_value = "./dist/")]
        out_dir: PathBuf,
        /// Platform name to pack assets for.
        #[arg(value_name = "NAME")]
        platform: String,
        /// Run WASM build instead of cargo build.
        #[arg(short, long)]
        wasm: bool,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Profile {
    Debug,
    Release,
}

impl Default for Profile {
    fn default() -> Self {
        Self::Debug
    }
}

impl Profile {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "debug" => Some(Self::Debug),
            "release" => Some(Self::Release),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

#[derive(Default, Deserialize)]
struct ProjectMeta {
    #[serde(default)]
    pub sccache_bin: Option<PathBuf>,
    #[serde(default)]
    pub sccache_dir: Option<PathBuf>,
}

#[allow(clippy::cognitive_complexity)]
#[tokio::main]
async fn main() -> Result<()> {
    let meta = MetadataCommand::new().exec();
    let (mut root_path, project_meta) = if let Ok(meta) = meta {
        meta.packages
            .iter()
            .find_map(|p| {
                if p.name == env!("CARGO_PKG_NAME") && !p.metadata.is_null() {
                    let root_path = p.manifest_path.clone();
                    let project_meta = if let Ok(project_meta) =
                        serde_json::from_value::<ProjectMeta>(p.metadata.clone())
                    {
                        project_meta
                    } else {
                        ProjectMeta::default()
                    };
                    Some((root_path.into(), project_meta))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| (current_exe().unwrap(), ProjectMeta::default()))
    } else {
        (current_exe()?, ProjectMeta::default())
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
    let has_presets = std::env::var("OXY_DONT_AUTO_UPDATE").is_ok()
        || read_dir(&presets_path)
            .map(|iter| iter.count() > 0)
            .unwrap_or_default();
    let update_presets = std::env::var("OXY_UPDATE_PRESETS").is_ok();
    let update_pack_file = std::env::var("OXY_UPDATE_FILE");
    if !has_presets || update_presets {
        if update_presets {
            let _ = remove_dir_all(&presets_path);
        }
        let bytes = if let Ok(ref update_pack_file) = update_pack_file {
            std::fs::read(update_pack_file)
                .unwrap_or_else(|_| panic!("Could not get bytes from {:?} file", update_pack_file))
        } else {
            let url = format!(
                "https://oxygengine.io/ignite-presets/oxygengine-presets-{}.pack",
                env!("CARGO_PKG_VERSION")
            );
            println!(
                "There are no presets installed in {:?} - trying to download them now from: {:?}",
                presets_path, url
            );
            let bytes = reqwest::blocking::get(&url)
                .unwrap_or_else(|_| panic!("Request for {:?} failed", url))
                .bytes()
                .unwrap_or_else(|_| panic!("Could not get bytes from {:?} response", url));
            bytes.as_ref().to_owned()
        };
        let files = bincode::deserialize::<HashMap<String, Vec<u8>>>(&bytes)
            .unwrap_or_else(|_| panic!("Could not unpack files from presets pack"));
        let _ = create_dir_all(&presets_path);
        for (fname, bytes) in files {
            let path = presets_path.join(fname);
            let mut dir_path = path.clone();
            dir_path.pop();
            let _ = create_dir_all(&dir_path);
            println!("Store file: {:?}", path);
            write(path, bytes)?;
        }
    }
    let presets_list = if presets_path.is_dir() {
        read_dir(&presets_path)?
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    Some(path.file_name().unwrap().to_str().unwrap().to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
    if presets_list.is_empty() && std::env::var("OXY_DONT_AUTO_UPDATE").is_err() {
        return Err(Error::new(
                ErrorKind::NotFound,
                "There are no presets installed - consider reinstalling oxygengine-ignite, it might be corrupted",
            ));
    }

    if let Some(path) = &project_meta.sccache_bin {
        std::env::set_var("RUSTC_WRAPPER", path.to_str().unwrap());
    }
    if let Some(path) = &project_meta.sccache_dir {
        std::env::set_var("SCCACHE_DIR", path.to_str().unwrap());
    }

    let cli = Cli::parse();
    match cli.command {
        Commands::New {
            id,
            mut destination,
            preset,
        } => {
            let preset_path = presets_path.join(&preset);
            if !preset_path.exists() {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("Preset not found: {} (in path: {:?})", preset, preset_path),
                ));
            }
            destination.push(&id);
            if let Err(err) = create_dir_all(&destination) {
                if err.kind() != ErrorKind::AlreadyExists {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Could not create directory: {:?}", destination),
                    ));
                }
            }

            println!("Make project: {:?}", &destination);
            println!("* Prepare project structure...");
            copy_dir(&preset_path, &destination, &id)?;
            println!("Done!");
        }
        Commands::Pack { input, output } => {
            pack_assets_and_write_to_file(&input, output)?;
        }
        Commands::Pipeline { config, template } => {
            if template {
                let pipeline = Pipeline {
                    source: "static".into(),
                    destination: "static".into(),
                    commands: vec![
                        PipelineCommand::Pipeline(Pipeline {
                            disabled: false,
                            destination: "assets-generated".into(),
                            clear_destination: true,
                            ..Default::default()
                        }),
                        PipelineCommand::Pipeline(Pipeline {
                            disabled: false,
                            source: "assets-source".into(),
                            destination: "assets-generated".into(),
                            commands: vec![PipelineCommand::Copy {
                                disabled: false,
                                from: vec!["assets.txt".into()],
                                to: "".into(),
                            }],
                            ..Default::default()
                        }),
                        PipelineCommand::Pack {
                            disabled: false,
                            paths: vec!["assets-generated".into()],
                            output: "assets.pack".into(),
                        },
                    ],
                    ..Default::default()
                };
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
                write(config, &contents)?;
            } else if config.exists() {
                Pipeline::load_and_execute(config)?;
            } else {
                println!("Could not find pipeline config file: {:?}", config);
            }
        }
        Commands::Build {
            profile,
            crate_dir,
            out_dir,
            platform,
            wasm,
            extras,
        } => {
            let profile = match Profile::from_str(profile.as_str()) {
                Some(profile) => profile,
                None => {
                    return Err(Error::new(
                        ErrorKind::NotFound,
                        format!("Unknown build profile: {}", profile),
                    ))
                }
            };
            build_project(profile, crate_dir, out_dir, platform, wasm, extras)?;
        }
        Commands::Serve {
            port,
            binaries,
            assets,
            open,
        } => {
            if open {
                open::that(format!("http://localhost:{}", port))
                    .expect("Could not open URL in the browser");
            }
            serve_files(port, binaries, assets).await;
        }
        Commands::ServeDir { port, root, open } => {
            if open {
                open::that(format!("http://localhost:{}", port))
                    .expect("Could not open URL in the browser");
            }
            serve_dir(port, root).await;
        }
        Commands::Live {
            profile,
            sources,
            assets,
            crate_dir,
            platform,
            wasm,
            command,
            args,
        } => {
            let mut watcher = Hotwatch::new().expect("Could not start files watcher");
            let build_notifier = Arc::new((Mutex::new(()), Condvar::new()));
            let pipeline_notifier = Arc::new((Mutex::new(()), Condvar::new()));
            for path in &sources {
                let build_notifier = build_notifier.clone();
                watcher
                    .watch(&path, move |event| match event {
                        Event::Create(_) | Event::Write(_) | Event::Remove(_) => {
                            build_notifier.1.notify_all();
                        }
                        _ => {}
                    })
                    .unwrap_or_else(|_| panic!("Could not watch for sources: {:?}", path));
                println!("* Watching sources: {:?}", path);
            }
            for path in &assets {
                let pipeline_notifier = pipeline_notifier.clone();
                watcher
                    .watch(&path, move |event| match event {
                        Event::Create(_) | Event::Write(_) | Event::Remove(_) => {
                            pipeline_notifier.1.notify_all();
                        }
                        _ => {}
                    })
                    .unwrap_or_else(|_| panic!("Could not watch for assets: {:?}", path));
                println!("* Watching assets: {:?}", path);
            }
            let platform2 = platform.to_owned();
            let builds = spawn(move || {
                let exe = current_exe().expect("Could not get path to the running executable");
                loop {
                    let mut command = Command::new(&exe);
                    command.arg("build");
                    command.arg("--profile");
                    command.arg(&profile);
                    command.arg("--crate-dir");
                    command.arg(&crate_dir);
                    command.arg(&platform2);
                    if wasm {
                        command.arg("--wasm");
                    }
                    if let Err(error) = command.status() {
                        println!("Error during build process execution: {:#?}", error);
                    } else {
                        println!("* Done building binaries!");
                    }
                    let guard = build_notifier
                        .0
                        .lock()
                        .expect("Could not lock build notifier");
                    drop(
                        build_notifier
                            .1
                            .wait(guard)
                            .expect("Could not wait for build notifier"),
                    );
                }
            });
            let pipelines = spawn(move || {
                let pipeline = format!("platforms/{}/pipeline.json", platform);
                let exe = current_exe().expect("Could not get path to the running executable");
                loop {
                    if let Err(error) = Command::new(&exe).arg("pipeline").arg(&pipeline).status() {
                        println!("* Error during pipeline process execution: {:#?}", error);
                    } else {
                        println!("* Done baking assets!");
                    }
                    let guard = pipeline_notifier
                        .0
                        .lock()
                        .expect("Could not lock pipeline notifier");
                    drop(
                        pipeline_notifier
                            .1
                            .wait(guard)
                            .expect("Could not wait for pipeline notifier"),
                    );
                }
            });
            if let Some(command) = command {
                let exe = current_exe().expect("Could not get path to the running executable");
                Command::new(exe)
                    .arg(command)
                    .args(args)
                    .status()
                    .expect("Could not serve binary and asset files");
            }
            let _ = builds.join();
            let _ = pipelines.join();
        }
        Commands::Package {
            profile,
            assets,
            crate_dir,
            out_dir,
            platform,
            wasm,
        } => {
            let current_path = current_dir()?;
            let exe = current_exe().expect("Could not get path to the running executable");
            let timer = Instant::now();
            create_dir_all(&out_dir)?;
            println!("* Packaging application in {} mode", profile);
            let mut command = Command::new(&exe);
            command.arg("build");
            command.arg("--profile");
            command.arg(&profile);
            command.arg("--crate-dir");
            command.arg(&crate_dir);
            command.arg("--out-dir");
            command.arg(&out_dir);
            command.arg(&platform);
            if wasm {
                command.arg("--wasm");
            }
            command
                .status()
                .unwrap_or_else(|_| panic!("Could not run build in {} mode", profile));
            set_current_dir(current_path.join(&crate_dir))?;
            let out_dir = Path::new(&crate_dir).join(&out_dir);
            if wasm {
                if remove_file(out_dir.join(".gitignore")).is_err() {
                    println!("Could not remove .gitignore file");
                }
                if remove_file(out_dir.join("package.json")).is_err() {
                    println!("Could not remove package.json file");
                }
            }
            println!("* Executing assets pipeline");
            let pipeline = format!("platforms/{}/pipeline.json", platform);
            Command::new(exe)
                .arg("pipeline")
                .arg(pipeline)
                .status()
                .expect("Could not run assets pipeline");
            set_current_dir(current_path.join(crate_dir))?;
            println!("* Copying assets to output directory");
            copy_dir(&assets, &out_dir, "")
                .expect("Could not copy baked assets to output directory");
            println!("* Done in: {:?}", timer.elapsed());
        }
    }

    Ok(())
}

async fn serve_files(port: u16, binaries: PathBuf, assets: PathBuf) {
    println!("* Serve project files at: localhost:{}", port);
    println!("- binaries:\t{:?}", binaries);
    println!("- assets:\t{:?}", assets);
    use warp::Filter;
    let binaries = warp::fs::dir(binaries);
    let assets = warp::fs::dir(assets);
    let routes = warp::get().and(binaries.or(assets));
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn serve_dir(port: u16, root: PathBuf) {
    println!("* Serve directory files at: localhost:{}", port);
    println!("- root:\t{:?}", root);
    use warp::Filter;
    let root = warp::fs::dir(root);
    let routes = warp::get().and(root);
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
                        continue;
                    }
                }
                let to = to.join(path.file_name().unwrap());
                copy(&path, to)?;
            }
        }
    }
    Ok(())
}

pub fn binary_artifacts_paths(profile: &str, config: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let meta = match MetadataCommand::new().manifest_path(config.as_ref()).exec() {
        Ok(meta) => meta,
        Err(error) => return Err(Error::new(ErrorKind::Other, error)),
    };
    let package = match meta.root_package() {
        Some(package) => package,
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not get project metadata root package",
            ))
        }
    };
    let target = match package.targets.first() {
        Some(target) => target,
        None => {
            return Err(Error::new(
                ErrorKind::Other,
                "Could not get project metadata root package target",
            ))
        }
    };
    if cfg!(target_os = "windows") {
        let binary = format!("{}.exe", target.name);
        let symbols = format!("{}.pdb", target.name.replace('-', "_"));
        Ok(vec![
            meta.target_directory.join(profile).join(binary).into(),
            meta.target_directory.join(profile).join(symbols).into(),
        ])
    } else if cfg!(target_os = "linux") {
        Ok(vec![meta
            .target_directory
            .join(profile)
            .join(&target.name)
            .into()])
    } else {
        Ok(vec![])
    }
}
