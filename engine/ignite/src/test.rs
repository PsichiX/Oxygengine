use std::{io::Error, process::Command, str::from_utf8};

#[derive(Debug, Copy, Clone)]
pub enum TestProfile {
    Debug,
    Release,
}

impl Default for TestProfile {
    fn default() -> Self {
        Self::Debug
    }
}

pub fn test_project(
    profile: TestProfile,
    crate_dir: Option<String>,
    extra_options: Vec<String>,
) -> Result<(), Error> {
    let command = Command::new("rustc").arg("-Vv").output()?.stdout;
    let target = from_utf8(&command)
        .expect("Could not parse rustc output")
        .lines()
        .find_map(|line| line.strip_prefix("host:"))
        .map(|target| target.trim())
        .expect("Could not get host target triple");
    let mut command = Command::new("cargo");
    command.arg("test");
    if matches!(profile, TestProfile::Release) {
        command.arg("--release");
    }
    command.arg("--target");
    command.arg(target);
    command.arg("--");
    command.args(extra_options);
    if let Some(current_dir) = crate_dir {
        command.current_dir(current_dir);
    }
    command.status()?;
    Ok(())
}
