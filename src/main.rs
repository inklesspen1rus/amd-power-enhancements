use clap::Parser;
use rustix::{fd::AsFd, fs::inotify};
use serde::Deserialize;
use hashbrown::HashMap;
use std::{
    error, fs,
    io::Read,
    mem::swap,
    path::{Path, PathBuf},
};

#[derive(Parser)]
struct Cli {
    /// Path to config file
    #[arg(short, long)]
    config: String,
}

#[derive(Deserialize, Debug)]
struct Options {
    #[serde(default)]
    power_profiles: HashMap<String, PowerProfileOptions>,

    backend: BackendOptions
}

#[derive(Deserialize, Debug, Default)]
struct PowerProfileOptions {
    #[serde(default)]
    commands: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct BackendOptions {
    backend: String,
    file_watch: FileWatchBackendOptions,
}

#[derive(Deserialize, Debug)]
struct FileWatchBackendOptions {
    file: PathBuf,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let cli = Cli::parse();
    let options = load_options(cli.config)?;

    assert_eq!(options.backend.backend, "file_watch", "Only file_watch backend supported!");

    let ntfy = inotify::inotify_init(inotify::CreateFlags::CLOEXEC)
        .expect("Unable to init inotify descriptor");

    let watch_file = &options
        .backend
        .file_watch
        .file;

    let mut applied_profile = std::fs::read_to_string(watch_file)?;
    let mut new_profile = String::new();
    
    apply_profile(applied_profile.trim_end(), &options)?;

    inotify::inotify_add_watch(ntfy.as_fd(), watch_file, inotify::WatchFlags::CLOSE_WRITE)?;

    let mut buffer = [0u8; 1024];
    loop {
        rustix::io::read(ntfy.as_fd(), &mut buffer)?;

        fs::File::open(&watch_file)?.read_to_string(&mut new_profile)?;

        if new_profile.trim() != applied_profile.trim() {
            swap(&mut new_profile, &mut applied_profile);
            apply_profile(applied_profile.trim_end(), &options)?;
        }

        new_profile.clear();
    }
}

fn load_options(path: impl AsRef<Path>) -> Result<Options, Box<dyn error::Error>> {
    let text = std::fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&text)?)
}

fn apply_profile(profile: &str, options: &Options) -> Result<(), Box<dyn error::Error>> {
    let profile_options = options
        .power_profiles
        .get(profile)
        .or_else(|| options.power_profiles.get("default"));
    
    if let Some(profile_options) = profile_options {
        for cmd in profile_options.commands.iter() {
            let ret = unsafe { libc::system(cmd.as_ptr().cast()) };
            if ret != 0 {
                eprintln!("Command {cmd} failed with code: {ret}");
            }
        }
    } else {
        eprintln!(r#"Not found power profile configuration for "{profile}""#);
    }

    Ok(())
}
