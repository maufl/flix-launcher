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
use slint::{Color, Image, VecModel, Weak};

slint::include_modules!();

/// A simple media center launcher
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// The location of the configuration file
    #[arg(short, long)]
    config: Option<String>,
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
    text_color: Option<String>,
    shutdown_command: String,
    apps: Vec<AppConfig>,
}

fn run_command(command: &str) {
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

fn read_config(path: Option<String>) -> Option<(PathBuf, Config)> {
    let paths = if let Some(p) = path {
        vec![p]
    } else {
        vec!["./flix.toml".to_owned()]
    };
    for path in paths {
        let Ok(mut path) = Path::new(&path).canonicalize() else {
            continue;
        };
        let Ok(config) = std::fs::read_to_string(&path) else {
            continue;
        };
        match toml::from_str(&config) {
            Ok(config) => {
                path.pop();
                return Some((path, config));
            }
            Err(err) => {
                println!("Failed to load configuration from {:?}: {}", path, err);
                return None;
            }
        };
    }
    println!("Unable to find a valid configuration");
    None
}

fn main() {
    let args = Args::parse();
    let Some((conf_dir, config)) = read_config(args.config) else {
        return;
    };

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
    if let Some(color_string) = config.text_color {
        if let Ok(color) = parse_color(&color_string) {
            let c = color.to_alpha_color::<Srgb>().to_rgba8();
            launcher
                .global::<Theme>()
                .set_text_color(Color::from_rgb_u8(c.r, c.g, c.b));
        } else {
            println!("Text color is invalid: {color_string}");
        };
    };
    launcher.set_wallpaper(
        Image::load_from_path(&conf_dir.join(&config.wallpaper)).expect("To load wallpaper"),
    );
    launcher.set_apps(VecModel::from_slice(apps.as_slice()));
    launcher.on_launch_command(move |command| {
        run_command(&command);
    });
    launcher.on_exit(move || {
        run_command(&config.shutdown_command);
    });
    let launcher_handle = launcher.as_weak();
    thread::spawn(move || update_time(launcher_handle));
    launcher.run().unwrap();
}
