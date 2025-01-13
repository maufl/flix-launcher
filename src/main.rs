use std::{
    path::{Path, PathBuf},
    process::Command,
    thread::{self, sleep},
    time::Duration,
};

use chrono::Local;
use clap::{command, Parser};
use color::{parse_color, Srgb};
use serde::Deserialize;
use slint::{Color, Image, SharedString, VecModel, Weak};

slint::include_modules!();

/// A simple media center launcher
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// The location of the configuration file
    #[arg(short, long)]
    config: String,
}

#[derive(Deserialize, Default)]
struct AppConfig {
    icon: String,
    preferred_color: String,
    command: String,
}

#[derive(Deserialize, Default)]
struct Config {
    wallpaper: String,
    apps: Vec<AppConfig>,
}

fn run_command(command: &SharedString) {
    let parts: Vec<_> = command.split(" ").collect();
    let Some((command, args)) = parts.split_first() else {
        return println!("Can't parse command: {}", command);
    };
    println!("Starting {} with args {:?}", command, args);
    if let Err(e) = Command::new(command).args(args).spawn() {
        println!("Unable to execute comman: {}", e);
    }
}

fn update_time(launcher_handle: Weak<Launcher>) -> ! {
    loop {
        let now = Local::now();
        let time_string = now.format("%H:%M %P").to_string();
        let date_string = now.format("%B %d, %Y").to_string();
        let launcher_handle = launcher_handle.clone();
        let res = slint::invoke_from_event_loop(move || {
            let launcher_handle = launcher_handle.unwrap();
            launcher_handle.set_time_string(time_string.into());
            launcher_handle.set_date_string(date_string.into());
        });
        if let Err(e) = res {
            println!("Error updating date and time: {}", e);
        }
        sleep(Duration::from_secs(1));
    }
}

fn read_config(path: &str) -> (PathBuf, Config) {
    let paths = [ path, "./flix.toml" ];
    for path in paths {
        let Ok(mut path) = Path::new(path).canonicalize() else {
            continue;
        };
        let Ok(config) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = toml::from_str(&config) else {
            continue;
        };
        path.pop();
        return (path, config);
    }
    println!("Unable to read configuration");
    (PathBuf::new(), Config::default())
}

fn main() {
    let args = Args::parse();
    let (conf_dir, config) = read_config(&args.config);

    let apps: Vec<App> = config
        .apps
        .iter()
        .map(|a| {
            let c = parse_color(&a.preferred_color)
                .expect("A valid color")
                .to_alpha_color::<Srgb>()
                .to_rgba8();
            App {
                icon: Image::load_from_path(&conf_dir.join(&a.icon)).expect("To load icon"),
                preferred_color: Color::from_rgb_u8(c.r, c.g, c.b),
                command: a.command.clone().into(),
            }
        })
        .collect();
    let launcher = Launcher::new().unwrap();
    launcher.set_wallpaper(
        Image::load_from_path(&conf_dir.join(&config.wallpaper)).expect("To load wallpaper"),
    );
    launcher.set_apps(VecModel::from_slice(apps.as_slice()));
    launcher.on_launch_command(move |command| {
        run_command(&command);
    });
    let launcher_handle = launcher.as_weak();
    thread::spawn(move || update_time(launcher_handle));
    launcher.run().unwrap();
}
