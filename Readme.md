# Simple program launcher for media center PCs
This is a very basic program launcher I wrote for my media center PC. I more or less tried to steal the design from [here](https://invent.kde.org/plasma/plasma-bigscreen/-/issues/17). Inspired by [Flex Launcher](https://github.com/complexlogic/flex-launcher).

Written in a bit of Rust, using [Slint](https://slint.dev/) for the UI.
Contributions are welcome, but I will not want to grow the feature set too much. 

Example of what it could look like ![screenshot](screenshot.png)

The configuration is a toml file, any files referenced in it are resolved relative to it.
```toml
wallpaper = "wallpaper.jpeg"
text_color = "black"
shutdown_command = "echo 'going to sleep'"

[[apps]]
icon = "youtube.png"
preferred_color = "#ffffff"
command = "firefox --kiosk https://youtube.com"


[[apps]]
icon = "steam.svg"
preferred_color = "rgb(51, 60, 68)"
command = "steam -bigpicture"

[[apps]]
icon = "jellyfin.svg"
preferred_color = "rgb(23, 29, 37)"
command = "jellyfinmediaplayer"
```

Licensed under GPLv3.